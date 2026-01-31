use crate::{parse_response_and_report, prelude::*};
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

/// Stores the corresponding errors from `infr_transfer::ServerError`.
#[derive(Message, Debug, Clone)]
pub struct LevelError(pub transfer::ServerError);

/// Loads a new map into the Map resource. This may take several frames.
#[derive(Message, Debug)]
pub struct LoadMap {
    pub pack: String,
    pub name: String,
}

pub(crate) fn start_load_map(
    mut reader: MessageReader<LoadMap>,
    mut commands: Commands,
) -> Result<()> {
    for message in reader.read() {
        let Some(path) = config::CONFIG.find_level(&message.pack, &message.name) else {
            warn!(
                "Unable to find level {} in pack {}",
                message.name, message.pack
            );
            continue;
        };
        let request = make_request(&transfer::LoadSessionRequest { path })?;
        commands.spawn(request).observe(observe_load_map);
    }
    Ok(())
}

fn observe_load_map(
    event: On<ResponseString>,
    mut commands: Commands,
    mut current_session: ResMut<CurrentSession>,
    mut writer: MessageWriter<LevelError>,
    mut map: ResMut<Map>,
) -> Result<()> {
    let session_id = parse_response_and_report!(SessionId, writer, event);
    **current_session = session_id;
    commands.entity(event.entity).despawn();
    map.objects.clear();
    // TODO: Read level data.
    Ok(())
}

pub(crate) fn refresh_session(mut commands: Commands, session: Res<CurrentSession>) -> Result<()> {
    if session.0 != u32::MAX {
        commands
            .spawn(make_request(&session.0)?)
            .observe(observe_discard_response);
    }
    Ok(())
}
