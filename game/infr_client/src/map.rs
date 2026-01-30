use crate::prelude::*;
use bevy::prelude::*;
use bevy_ehttp::prelude::*;

use std::collections::HashMap;

/// Stores the current map provided by the server.
#[derive(Resource, Debug)]
pub struct Map {
    pub session_id: transfer::SessionId,
    pub objects: HashMap<u32, Entity>,
}
