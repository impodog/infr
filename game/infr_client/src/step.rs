//! This file handles player input and send requests to step the game.

use std::collections::{HashMap, VecDeque};

use crate::{parse_response_and_report, prelude::*};
use bevy::prelude::*;
use bevy_ehttp::prelude::*;

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct PlayerDirection(pub transfer::Direction);
#[derive(Message, Debug, Clone, Copy)]
pub enum PlayerAction {
    // TODO
}

pub(crate) fn listen_keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<PlayerDirection>,
) {
    if input.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        writer.write(PlayerDirection(transfer::Direction::RIGHT));
    }
    if input.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        writer.write(PlayerDirection(transfer::Direction::UP));
    }
    if input.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        writer.write(PlayerDirection(transfer::Direction::LEFT));
    }
    if input.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        writer.write(PlayerDirection(transfer::Direction::DOWN));
    }
}

// -- Handle player inputs

/// Queues up player inputs, allowing more slick controls(by pre-sending requests) and pre-inputs.
#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct PlayerInputQueue(pub VecDeque<transfer::Direction>);

pub(crate) fn queue_player_input(
    mut reader: MessageReader<PlayerDirection>,
    mut queue: ResMut<PlayerInputQueue>,
) {
    queue.extend(reader.read().map(|message| message.0));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn read_player_input(
    mut commands: Commands,
    mut map_state: ResMut<NextState<crate::MapState>>,
    mut current_round: ResMut<crate::CurrentRound>,
    session: Res<crate::CurrentSession>,
    mut queue: ResMut<PlayerInputQueue>,
    q_request: Query<(), With<HttpRequest>>,
    mut request_level: ResMut<StepRequestLevel>,
    action_count: Res<ActionCount>,
) -> Result<()> {
    if current_round.actual_round != 0
        || q_request.iter().next().is_some()
        || action_count.actual != action_count.messages
    {
        return Ok(());
    }
    let Some(direction) = queue.pop_front() else {
        return Ok(());
    };
    info!("Player input direction: {direction:?}");
    request_level.0 += 1;
    commands
        .spawn(make_post_request(
            "session/step",
            &transfer::SendStepRequest {
                session_id: session.0,
                direction,
            },
        )?)
        .observe(observe_step);
    map_state.set(crate::MapState::Stepping);
    current_round.actual_round = 0;
    current_round.animation_round = 0;
    Ok(())
}

/// Temporarily stores all movements while in a single step.
#[derive(Resource, Debug, Clone, Default, Deref, DerefMut)]
pub struct CurrentMovements(pub HashMap<u32, transfer::Movement>);

/// Temporarily stores all objects while in a single step.
#[derive(Resource, Debug, Clone, Default, Deref, DerefMut)]
pub struct CurrentObjects(pub HashMap<u32, transfer::Object>);

/// Stores the round number of each step, this will be set to 0 each time player inputs.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentRound {
    pub actual_round: usize,
    pub animation_round: usize,
}

fn observe_step(
    event: On<ResponseString>,
    mut commands: Commands,
    mut writer: MessageWriter<LevelError>,
    mut current_movements: ResMut<CurrentMovements>,
    mut current_round: ResMut<CurrentRound>,
    session: Res<crate::CurrentSession>,
    mut request_level: ResMut<StepRequestLevel>,
) -> Result<()> {
    let transfer::SendStepResponse { movements } =
        parse_response_and_report!(transfer::SendStepResponse, writer, event);

    info!("Received movements: {movements:?}");

    request_level.0 -= 1;

    // Return to free if no more movements.
    if movements.is_empty() {
        current_movements.0.clear();
        // This is for the animation's turn to end the whole process.
        current_round.actual_round += 1;
        commands.entity(event.entity).despawn();
        return Ok(());
    }

    let requested_objects = movements
        .iter()
        .map(|movement| movement.object)
        .collect::<Vec<_>>();

    current_movements.0 = movements
        .into_iter()
        .map(|movement| (movement.object, movement))
        .collect();

    commands
        .spawn(make_get_request(
            "session/objects",
            &transfer::GetObjectsRequest {
                session_id: session.0,
                objects: requested_objects,
            },
        )?)
        .observe(observe_load_map_when_moving);
    commands.entity(event.entity).despawn();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn observe_load_map_when_moving(
    event: On<ResponseString>,
    mut commands: Commands,
    mut writer: MessageWriter<LevelError>,
    mut current_objects: ResMut<CurrentObjects>,
    mut current_round: ResMut<CurrentRound>,
) -> Result<()> {
    current_objects.0 = parse_response_and_report!(transfer::Map, writer, event)
        .0
        .into_iter()
        .map(|object| (object.id, object))
        .collect::<HashMap<_, _>>();

    // Set flag for pending animations.
    current_round.actual_round += 1;

    commands.entity(event.entity).despawn();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn send_animations(
    mut commands: Commands,
    current_objects: Res<CurrentObjects>,
    current_movements: Res<CurrentMovements>,
    session: Res<crate::CurrentSession>,
    mut map: ResMut<crate::Map>,
    mut q_object: Query<(&mut crate::ObjectState, &mut crate::ObjectGroups)>,
    mut any_moving: ResMut<crate::AnyObjectMoving>,
    mut current_round: ResMut<crate::CurrentRound>,
    mut map_state: ResMut<NextState<crate::MapState>>,
    mut action_count: ResMut<ActionCount>,
) -> Result<()> {
    use transfer::MoveManner;

    // Test if there are pending animations.
    if current_round.actual_round <= current_round.animation_round {
        return Ok(());
    }
    if current_movements.is_empty() {
        current_round.actual_round = 0;
        current_round.animation_round = 0;
        action_count.actual += 1;
        map_state.set(crate::MapState::Free);
        return Ok(());
    }

    for (id, movement) in current_movements.iter() {
        let Some(object) = current_objects.get(id) else {
            error!("Unknown object id {id} in movements list");
            continue;
        };
        match &movement.manner {
            MoveManner::Add => {
                let entity = commands
                    .spawn((
                        crate::Object {
                            id: *id,
                            session_id: session.0,
                        },
                        crate::ObjectState {
                            direction: object.direction,
                            nature: object.nature.clone(),
                        },
                        crate::Position(Vec2::new(object.coord.0 as f32, object.coord.1 as f32)),
                        crate::ObjectGroups(object.groups.clone()),
                        crate::ObjectFlags(object.flags.clone()),
                    ))
                    .id();
                map.objects.insert(*id, entity);
            }
            MoveManner::Remove => {
                // Maybe sleek remove animations?
                let Some(entity) = map.objects.get(id).copied() else {
                    error!("Unknown object id {id} in client objects list");
                    continue;
                };
                if let Ok(mut commands) = commands.get_entity(entity) {
                    commands.despawn();
                }
                map.objects.remove(id);
            }
            MoveManner::Swipe(_) | MoveManner::Teleport => {
                let Some(entity) = map.objects.get(id).copied() else {
                    error!("Unknown object id {id} in client objects list");
                    continue;
                };
                commands.trigger(crate::Movement {
                    entity,
                    dest: crate::Position(Vec2::new(
                        movement.dest.0 as f32,
                        movement.dest.1 as f32,
                    )),
                    manner: movement.manner.clone(),
                });
            }
        }
    }

    for (id, object) in current_objects.iter() {
        // Some entities may be removed or added, thus are not in this list.
        if let Some(entity) = map.objects.get(id)
            && let Ok((mut object_state, mut object_groups)) = q_object.get_mut(*entity)
        {
            object_state.direction = object.direction;
            object_state.nature = object.nature.clone();
            object_groups.0 = object.groups.clone();
        }
    }

    current_round.animation_round += 1;
    any_moving.0 = true;

    Ok(())
}

/// This is a fix to a unknown bug(probably ehttp) where HttpRequest cannot be counted.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct StepRequestLevel(usize);

pub(crate) fn finish_animations(
    mut commands: Commands,
    session: Res<crate::CurrentSession>,
    any_moving: Res<crate::AnyObjectMoving>,
    q_request: Query<(), With<HttpRequest>>,
    current_round: ResMut<CurrentRound>,
    mut request_level: ResMut<StepRequestLevel>,
) -> Result<()> {
    if !any_moving.0 && q_request.iter().next().is_none() && current_round.actual_round == current_round.animation_round && request_level.0 == 0
        // Skip the first round --- provides input to the user in another function.
        && current_round.actual_round != 0
    {
        request_level.0 += 1;
        commands
            .spawn(make_post_request(
                "session/step",
                &transfer::SendStepRequest {
                    session_id: session.0,
                    direction: Default::default(),
                },
            )?)
            .observe(observe_step);
    }
    Ok(())
}

/// Tracks the current player actions that has been done.
///
/// The actual actions is incremented by the last time the animations are sent, not when done playing.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ActionCount {
    messages: usize,
    pub actual: usize,
}

/// Message that indicates a player action has finished playing all animations.
/// You should not use OnEnter(MapState::Free), because it is changed before the last animation.
#[derive(Message, Debug, Clone, Copy)]
pub struct ActionFinished(pub usize);

pub(crate) fn test_action_finished(
    mut action_count: ResMut<ActionCount>,
    mut writer: MessageWriter<ActionFinished>,
    any_moving: Res<crate::AnyObjectMoving>,
) {
    if !any_moving.0 && action_count.messages < action_count.actual {
        action_count.messages += 1;
        writer.write(ActionFinished(action_count.messages));
    }
}
