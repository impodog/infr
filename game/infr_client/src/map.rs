use crate::{parse_response_and_report, prelude::*};
use bevy::prelude::*;
use bevy_ehttp::prelude::*;

use std::collections::HashMap;

/// Stores the current map provided by the server.
#[derive(Resource, Debug, Default)]
pub struct Map {
    pub objects: HashMap<u32, Entity>,
}
impl Map {
    /// Useful method for clearing the map when reverting or loading a new map.
    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

#[derive(Resource, Debug, Clone, Copy, Deref, DerefMut)]
pub struct CurrentSession(pub SessionId);
impl Default for CurrentSession {
    fn default() -> Self {
        Self(SessionId::MAX)
    }
}

/// Stores the corresponding errors from `infr_transfer::ServerError`.
#[derive(Message, Debug, Clone)]
pub struct LevelError(pub transfer::ServerError);

/// Loads a new map into the Map resource. This may take several frames.
#[derive(Message, Debug)]
pub struct LoadMap {
    pub pack: String,
    pub name: String,
}

/// Marks an entity to be removed when switching sessions.
#[derive(Component, Default)]
pub struct SessionOnly;

pub(crate) fn start_load_map(
    mut reader: MessageReader<LoadMap>,
    mut commands: Commands,
    mut map_state: ResMut<NextState<crate::MapState>>,
    q_session_only: Query<Entity, With<SessionOnly>>,
) -> Result<()> {
    for message in reader.read() {
        let Some(path) = config::CONFIG.find_level(&message.pack, &message.name) else {
            warn!(
                "Unable to find level {} in pack {}",
                message.name, message.pack
            );
            continue;
        };
        let request = make_post_request("session/load", &transfer::LoadSessionRequest { path })?;
        commands.spawn(request).observe(observe_load_map);
        map_state.set(crate::MapState::Loading);
        for entity in q_session_only.iter() {
            commands.entity(entity).despawn();
        }
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
    info!("Loaded session id: {session_id}");
    **current_session = session_id;
    map.objects.clear();

    // Read level data.
    commands
        .spawn(make_get_request("session/meta", &session_id)?)
        .observe(observe_level_meta);
    commands
        .spawn(make_get_request("session/map", &session_id)?)
        .observe(observe_level_map);

    commands.entity(event.entity).despawn();

    Ok(())
}

/// Stores the metadata that the level provides.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub struct LevelMetadata(pub transfer::LevelMetadata);

fn observe_level_meta(
    event: On<ResponseString>,
    mut commands: Commands,
    mut writer: MessageWriter<LevelError>,
) -> Result<()> {
    let meta = parse_response_and_report!(transfer::LevelMetadata, writer, event);
    info!("Acquired level metadata: {meta:?}");
    commands.insert_resource(LevelMetadata(meta));
    commands.entity(event.entity).despawn();
    Ok(())
}

fn observe_level_map(
    event: On<ResponseString>,
    mut commands: Commands,
    mut writer: MessageWriter<LevelError>,
    mut client_map: ResMut<crate::Map>,
    mut action_count: ResMut<crate::ActionCount>,
    session: Res<CurrentSession>,
) -> Result<()> {
    let map = parse_response_and_report!(transfer::Map, writer, event);
    client_map.clear();
    *action_count = crate::ActionCount::default();
    info!("Acquired level map: {map:?}");
    for object in map.0.into_iter() {
        let entity = commands
            .spawn((
                crate::Object {
                    id: object.id,
                    session_id: session.0,
                },
                crate::ObjectState {
                    direction: object.direction,
                    nature: object.nature,
                },
                crate::Position(Vec2::new(object.coord.0 as f32, object.coord.1 as f32)),
                crate::ObjectGroups(object.groups),
                crate::ObjectFlags(object.flags),
            ))
            .id();
        client_map.objects.insert(object.id, entity);
    }
    commands.entity(event.entity).despawn();
    Ok(())
}

pub(crate) fn finish_load_map(
    q_request: Query<(), With<RequestMarker>>,
    mut map_state: ResMut<NextState<crate::MapState>>,
) {
    if q_request.iter().next().is_none() {
        map_state.set(crate::MapState::Free);
    }
}

pub(crate) fn refresh_session(mut commands: Commands, session: Res<CurrentSession>) -> Result<()> {
    if session.0 != u32::MAX {
        commands
            .spawn(make_post_request("session/refresh", &session.0)?)
            .observe(observe_discard_response);
    }
    Ok(())
}
