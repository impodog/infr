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

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
pub struct CurrentSession {
    #[deref]
    pub id: SessionId,
    /// Saves the exact message used to load this map.
    pub name: LoadMap,
}
impl Default for CurrentSession {
    fn default() -> Self {
        Self {
            id: SessionId::MAX,
            name: Default::default(),
        }
    }
}

/// Stores the corresponding errors from `infr_transfer::ServerError`.
#[derive(Message, Debug, Clone)]
pub struct LevelError(pub transfer::ServerError);

/// Loads a new map into the Map resource. This may take several frames.
#[derive(Message, Component, Debug, Clone, Default)]
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
        commands
            .spawn((request, message.clone()))
            .observe(observe_load_map);
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
    q_message: Query<&LoadMap>,
) -> Result<()> {
    let session_id = parse_response_and_report!(SessionId, writer, event);
    info!("Loaded session id: {session_id}");
    current_session.id = session_id;
    if let Ok(message) = q_message.get(event.entity) {
        current_session.name = message.clone();
    }
    map.objects.clear();

    // Read level data.
    commands
        .spawn(make_get_request("session/meta", &session_id)?)
        .observe(observe_level_meta);
    commands
        .spawn(make_get_request("session/map", &session_id)?)
        .observe(observe_level_map);
    commands
        .spawn(make_get_request("session/rules", &session_id)?)
        .observe(crate::step::observe_get_rules);

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
                    session_id: session.id,
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
    if session.id != u32::MAX {
        commands
            .spawn(make_post_request("session/refresh", &session.id)?)
            .observe(observe_discard_response);
    }
    Ok(())
}

/// Message to completely reload all objects of the map.
#[derive(Message, Default)]
pub struct ReloadMap;

pub(crate) fn handle_reload_map(
    mut reader: MessageReader<ReloadMap>,
    mut commands: Commands,
    q_request: Query<Entity, With<RequestMarker>>,
    mut state: ResMut<NextState<crate::MapState>>,
    session: Res<CurrentSession>,
) -> Result<()> {
    // Filters multiple requests.
    if reader.read().next().is_none() {
        return Ok(());
    }
    state.set(crate::MapState::Reloading);
    q_request.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
    commands
        .spawn(make_post_request("session/map", &session.id)?)
        .observe(observe_reload_map);
    Ok(())
}

fn observe_reload_map(
    event: On<ResponseString>,
    mut commands: Commands,
    mut state: ResMut<NextState<crate::MapState>>,
    mut writer: MessageWriter<LevelError>,
    mut map: ResMut<Map>,
    mut q_object: Query<(&mut crate::ObjectState, &mut crate::ObjectGroups)>,
    session: Res<CurrentSession>,
) -> Result<()> {
    let objects = parse_response_and_report!(transfer::Map, writer, event)
        .0
        .into_iter()
        .map(|object| (object.id, object))
        .collect::<HashMap<_, _>>();
    for (id, object) in objects.into_iter() {
        if let Some(entity) = map.objects.get(&id)
            && let Ok((mut object_state, mut object_groups)) = q_object.get_mut(*entity)
        {
            object_state.direction = object.direction;
            object_state.nature = object.nature.clone();
            object_groups.0 = object.groups.clone();
        } else {
            let entity = commands
                .spawn((
                    crate::Object {
                        id: object.id,
                        session_id: session.id,
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
            map.objects.insert(object.id, entity);
        }
    }

    state.set(crate::MapState::Free);
    commands.entity(event.entity).despawn();

    Ok(())
}

pub(crate) fn reload_on_error(
    mut reader: MessageReader<LevelError>,
    mut writer: MessageWriter<ReloadMap>,
) {
    if reader.read().next().is_some() {
        writer.write(ReloadMap);
    }
}

pub(crate) fn restart_on_input(
    mut reader: MessageReader<crate::PlayerAction>,
    mut writer: MessageWriter<LoadMap>,
    session: Res<CurrentSession>,
) {
    for action in reader.read() {
        if matches!(action, crate::PlayerAction::Restart) {
            writer.write(session.name.clone());
        }
    }
}
