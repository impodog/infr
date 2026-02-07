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

pub(crate) fn read_player_input(
    mut commands: Commands,
    mut map_state: ResMut<NextState<crate::MapState>>,
    mut current_round: ResMut<crate::CurrentRound>,
    session: Res<crate::CurrentSession>,
    mut queue: ResMut<PlayerInputQueue>,
) -> Result<()> {
    if current_round.actual_round != 0 {
        return Ok(());
    }
    let Some(direction) = queue.pop_front() else {
        return Ok(());
    };
    info!("Player input direction: {direction:?}");
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
) -> Result<()> {
    let transfer::SendStepResponse { movements } =
        parse_response_and_report!(transfer::SendStepResponse, writer, event);

    info!("Received movements: {:?}", movements);

    // Return to free if no more movements.
    if movements.is_empty() {
        current_movements.0.clear();
        // This is for the animation's turn to end the whole process.
        current_round.actual_round += 1;
        return Ok(());
    }

    current_movements.0 = movements
        .into_iter()
        .map(|movement| (movement.object, movement))
        .collect();

    commands
        .spawn(make_get_request("session/map", &session.0)?)
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
) -> Result<()> {
    use transfer::MoveManner;

    // Test if there are pending animations.
    if current_round.actual_round == current_round.animation_round {
        return Ok(());
    }
    if current_movements.is_empty() {
        map_state.set(crate::MapState::Free);
        current_round.actual_round = 0;
        current_round.animation_round = 0xfeedd095;
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

pub(crate) fn finish_animations(
    mut commands: Commands,
    session: Res<crate::CurrentSession>,
    any_moving: Res<crate::AnyObjectMoving>,
    q_request: Query<(), With<HttpRequest>>,
    current_round: Res<CurrentRound>,
) -> Result<()> {
    if !any_moving.0
        && q_request.iter().next().is_none()
        && current_round.actual_round == current_round.animation_round
        // Skip the first round --- provides input to the user in another function.
        && current_round.actual_round != 0
    {
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
