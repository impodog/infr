use crate::prelude::*;
use bevy::prelude::*;
use bevy_ehttp::prelude::*;

use std::collections::HashMap;

/// Stores the current map provided by the server.
#[derive(Resource, Debug)]
pub struct Map {
    pub objects: HashMap<u32, Entity>,
}

#[derive(Resource, Debug, Clone, Copy, Deref, DerefMut)]
pub struct CurrentSession(pub SessionId);
impl Default for CurrentSession {
    fn default() -> Self {
        Self(SessionId::MAX)
    }
}

/// Whether the map has completed communicating with the server, playing animation, and therefore
/// can accept user input.
#[derive(Resource, Default, Debug, Clone, Copy, Deref, DerefMut)]
pub struct MapAcceptsInput(pub bool);

/// Loads a new map into the Map resource. This may take several frames.
#[derive(Message, Debug)]
pub struct LoadMap {
    pub pack: String,
    pub name: String,
}

pub(crate) fn start_load_map(mut reader: MessageReader<LoadMap>) {
    for message in reader.read() {
        let Some(level_path) = config::CONFIG.find_level(&message.pack, &message.name) else {
            warn!(
                "Unable to find level {} in pack {}",
                message.name, message.pack
            );
            continue;
        };
    }
}
