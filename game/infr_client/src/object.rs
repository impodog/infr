use std::sync::OnceLock;

use crate::prelude::*;
use bevy::prelude::*;

/// Entry struct for object entities.
#[derive(Component, Default, Debug, Clone, Copy)]
#[require(ObjectState, Position, Sprite, MovementState)]
pub struct Object {
    pub id: ObjectId,
    pub session_id: SessionId,
}

/// The position of the object in fraction.
#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

/// Stores states that decide which animation this object should play.
#[derive(Component, Default, Debug, Clone)]
pub struct ObjectState {
    pub direction: transfer::Direction,
    pub nature: String,
}

/// Stores all the groups that the object is in, provided by the server.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut)]
pub struct ObjectGroups(pub Vec<String>);

/// Stores the center and extent of all positions
#[derive(Resource, Default, Debug, Clone)]
pub struct PositionCenter {
    pub center: Vec2,
    /// This is half of the length and width of all objects.
    pub extent: Vec2,
}

pub(crate) fn convert_position(
    mut q_position: Query<(&mut Transform, &Object, &Position)>,
    session_id: Res<crate::CurrentSession>,
    mut position_center: ResMut<PositionCenter>,
) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (_, object, position) in q_position.iter() {
        if object.session_id == **session_id {
            min_x = min_x.min(position.x);
            max_x = max_x.max(position.x);
            min_y = min_y.min(position.y);
            max_y = max_y.max(position.y);
        }
    }
    let center = Vec2::new((max_x + min_x) * 0.5, (max_y + min_y) * 0.5);
    let extent = Vec2::new((max_x - min_x) * 0.5, (max_y - min_y) * 0.5);
    position_center.center = center;
    position_center.extent = extent;

    q_position
        .par_iter_mut()
        .for_each(|(mut transform, _, position)| {
            let rel_position = position.0 - center;
            transform.translation.x = rel_position.x * config::CONFIG.display.tile_size.0 as f32;
            transform.translation.y = rel_position.y * config::CONFIG.display.tile_size.1 as f32;
        });
}

/// Message that inform objects to do a specific animation.
#[derive(EntityEvent, Debug, Clone)]
pub struct Movement {
    pub entity: Entity,
    pub dest: Position,
    pub manner: transfer::MoveManner,
}

/// Stores current movement animation of the object.
#[derive(Component, Default, Debug, Clone)]
pub(crate) struct MovementState {
    pub moving: bool,
    pub dest: Vec2,
    pub speed: f32,
}
/// Stores if any object has animation playing.
#[derive(Resource, Default)]
pub(crate) struct AnyObjectMoving(pub bool);

pub(crate) fn move_object(
    mut q_object: Query<(&mut Position, &mut MovementState)>,
    mut any_object_moving: ResMut<AnyObjectMoving>,
    time: Res<Time>,
) {
    let any_moving = OnceLock::<bool>::new();
    q_object
        .par_iter_mut()
        .for_each(|(mut position, mut movement_state)| {
            if movement_state.moving {
                let direction = (movement_state.dest - **position).normalize_or_zero();
                let target = **position + direction * movement_state.speed * time.delta_secs();
                let (min, max) = if target.x < position.x {
                    (target.x, position.x)
                } else {
                    (position.x, target.x)
                };
                position.0 = target;
                // Update moving status
                if (min..=max).contains(&movement_state.dest.x) {
                    movement_state.moving = false;
                } else {
                    any_moving.set(true).ok();
                }
            }
        });
    any_object_moving.0 = *any_moving.get_or_init(|| false);
}
