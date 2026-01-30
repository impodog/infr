use crate::prelude::*;
use bevy::prelude::*;

/// Entry struct for object entities.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Object(pub ObjectId);

/// The position of the object in fraction.
pub struct Position(pub Vec2);

/// Stores states that decide which animation this object should play.
#[derive(Component, Default, Debug, Clone)]
pub struct ObjectState {
    pub direction: transfer::Direction,
    pub nature: String,
}

/// Message to spawn an object entity.
#[derive(Message, Debug, Clone)]
pub struct AddObject {
    pub desc: transfer::Object,
}
