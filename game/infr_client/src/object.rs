use std::sync::OnceLock;

use crate::prelude::*;
use bevy::prelude::*;

/// Entry struct for object entities.
#[derive(Component, Default, Debug, Clone, Copy)]
#[require(ObjectState, Position, Sprite, MovementState, ObjectFlags)]
pub struct Object {
    pub id: ObjectId,
    pub session_id: SessionId,
}

/// The position of the object in fraction.
#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

impl Position {
    /// Attempts to round the position to nearest integer coordinates, if the error is acceptable.
    pub fn try_round(&self) -> Option<Coord> {
        fn try_round_f32(value: f32) -> Option<i32> {
            let rounded = value.round();
            if (value - rounded).abs() < 1e-2 {
                Some(rounded as i32)
            } else {
                None
            }
        }
        let x = try_round_f32(self.x)?;
        let y = try_round_f32(self.y)?;
        Some(Coord(x, y))
    }
}

/// Stores states that decide which animation this object should play.
#[derive(Component, Default, Debug, Clone)]
pub struct ObjectState {
    pub direction: transfer::Direction,
    pub nature: String,
}

/// Stores all the groups that the object is in, provided by the server.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut)]
pub struct ObjectGroups(pub Vec<String>);

/// Object flags acquired from the level configuration.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut)]
pub struct ObjectFlags(pub Vec<String>);

impl ObjectFlags {
    /// Searches for all flags that starts with this prefix.
    /// This will automatically strip the prefix.
    ///
    /// Note: The flags must be sorted(auto done by the object).
    pub fn search_flags<'f>(&'f self, pat: &str) -> impl std::iter::Iterator<Item = &'f str> {
        fn take_slice(s: &str, len: usize) -> &str {
            if len < s.len() { &s[..len] } else { s }
        }

        let lower_bound = {
            let mut l = 0;
            let mut r = self.len();
            while l < r {
                let mid = (l + r) >> 1;
                if self[mid].as_str() < pat {
                    l = mid + 1;
                } else {
                    r = mid;
                }
            }
            l
        };
        let upper_bound = {
            let mut l = 0;
            let mut r = self.len();
            while l < r {
                let mid = (l + r) >> 1;
                if take_slice(self[mid].as_str(), pat.len()) <= pat {
                    l = mid + 1;
                } else {
                    r = mid;
                }
            }
            l
        };
        if lower_bound < upper_bound {
            self[lower_bound..upper_bound].iter()
        } else {
            self[0..0].iter()
        }
        .map(|s| &s[pat.len()..])
    }
}

/// Stores the center and extent of all positions
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct PositionCenter {
    pub center: Vec2,
    /// This is half of the length and width of all objects.
    pub extent: Vec2,
}

pub(crate) fn convert_position(
    mut q_position: Query<(&mut Transform, Option<&Object>, &Position)>,
    session_id: Res<crate::CurrentSession>,
    mut position_center: ResMut<PositionCenter>,
) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (_, object, position) in q_position.iter() {
        if object.is_some_and(|object| object.session_id == **session_id) {
            min_x = min_x.min(position.x);
            max_x = max_x.max(position.x);
            min_y = min_y.min(position.y);
            max_y = max_y.max(position.y);
        }
    }
    let center = Vec2::new((max_x + min_x) * 0.5, (max_y + min_y) * 0.5);
    let extent = Vec2::new((max_x - min_x + 1.0) * 0.5, (max_y - min_y + 1.0) * 0.5);
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
pub(crate) fn observe_movement_event(
    event: On<Movement>,
    mut q_move_state: Query<&mut MovementState>,
) {
    let Ok(mut move_state) = q_move_state.get_mut(event.entity) else {
        warn!("Sending movement event to entity without MovementState");
        return;
    };
    let speed: f32 = match &event.manner {
        transfer::MoveManner::Swipe(_) => config::CONFIG.client.movement_velocity,
        _ => config::CONFIG.client.teleport_velocity,
    };
    move_state.moving = true;
    move_state.dest = event.dest.0;
    move_state.speed = speed;
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
    mut q_object: Query<(&mut Position, &mut MovementState, &mut Transform)>,
    mut any_object_moving: ResMut<AnyObjectMoving>,
    round: Res<crate::CurrentRound>,
    time: Res<Time>,
) {
    fn min_max(a: f32, b: f32) -> (f32, f32) {
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        // This prevents floating point precision causing animation not to end.
        (a - 1e-5, b + 1e-5)
    }

    let any_moving = OnceLock::<bool>::new();
    q_object
        .par_iter_mut()
        .for_each(|(mut position, mut movement_state, mut transform)| {
            if movement_state.moving {
                let delta = movement_state.dest - **position;
                let direction = (movement_state.dest - **position).normalize_or_zero();
                let speed = movement_state.speed
                    * if direction.length() > 0.0 {
                        (delta.length() / direction.length() * 1.5).clamp(1.0, 1.5)
                    } else {
                        1.0
                    };
                let target = **position + direction * speed * time.delta_secs();
                let (min_x, max_x) = min_max(target.x, position.x);
                let (min_y, max_y) = min_max(target.y, position.y);
                position.0 = target;
                // Update moving status
                if (min_x..=max_x).contains(&movement_state.dest.x)
                    && (min_y..max_y).contains(&movement_state.dest.y)
                {
                    movement_state.moving = false;
                    position.0 = movement_state.dest;
                } else {
                    any_moving.set(true).ok();
                }
                // The ensures objects that move later the display on top.
                transform.translation.z = round.animation_round as f32;
            }
        });
    any_object_moving.0 = *any_moving.get_or_init(|| false);
}

pub(crate) fn remove_past_objects(
    q_object: Query<(Entity, &Object)>,
    session: Res<crate::CurrentSession>,
    mut commands: Commands,
) {
    for (entity, object) in q_object.iter() {
        if object.session_id != session.0 {
            commands.entity(entity).despawn();
        }
    }
}
