use std::collections::{BTreeSet, HashMap};

use bevy::prelude::*;
use infr_client::prelude::*;

/// Informs the game to switch to a new room and update save file.
#[derive(Message, Debug, Clone)]
pub struct LoadRoomMessage {
    pub room: crate::Room,
    pub direction: transfer::Direction,
}
#[derive(Resource, Debug, Clone, Default, Deref)]
pub struct CurrentRoom(crate::Room);

pub(crate) fn load_room(
    mut reader: MessageReader<LoadRoomMessage>,
    mut writer: MessageWriter<infr_client::LoadMap>,
    mut current_room: ResMut<CurrentRoom>,
    mut user: ResMut<crate::UserData>,
) {
    for LoadRoomMessage { room, direction } in reader.read() {
        writer.write(room.into());
        let prev_coord = user
            .rooms
            .get(room)
            .map(|data| data.coord)
            .unwrap_or_default();
        let new_coord = prev_coord + direction.delta();
        user.add_room(room.clone(), new_coord);
        user.current_room.0 = room.clone();
        current_room.0 = room.clone();
    }
}

pub(crate) fn finish_load_room() {
    // TODO
}

pub(crate) fn startup_load_user(
    user: Res<crate::UserData>,
    mut writer: MessageWriter<LoadRoomMessage>,
) {
    writer.write(LoadRoomMessage {
        room: user.current_room.0.clone(),
        direction: transfer::Direction::default(),
    });
}

pub(crate) fn switch_to_next_room(
    mut reader: MessageReader<infr_client::ActionFinished>,
    mut writer: MessageWriter<LoadRoomMessage>,
    q_object: Query<(
        &infr_client::Position,
        &infr_client::ObjectFlags,
        &infr_client::ObjectGroups,
    )>,
) {
    if reader.read().next().is_none() {
        return;
    }
    let you: String = "You".into();
    let mut finish_tiles = HashMap::<Coord, String>::new();
    // BTreeSet ensure top left finish tiles are tested first.
    let mut players = BTreeSet::<Coord>::new();
    for (position, flags, groups) in q_object.iter() {
        let Some(coord) = position.try_round() else {
            return;
        };
        if let Some(finish_flag) = flags.search_flags("C:Finish:").next() {
            finish_tiles
                .entry(coord)
                .or_insert_with(|| finish_flag.to_owned());
        }
        if groups.binary_search(&you).is_ok() {
            players.insert(coord);
        }
    }
    info!("players: {players:?}; finish_tiles: {finish_tiles:?}");
    for player in players.iter() {
        if let Some(finish_flag) = finish_tiles.get(player) {
            let Some(mid) = finish_flag.find('/') else {
                warn!(
                    "'C:Finish:{finish_flag}' flag should contain '/' character in the form of 'Direction/Pack.Name'"
                );
                continue;
            };
            let Ok(direction) = finish_flag[..mid].parse::<transfer::Direction>();
            if !direction.is_some() {
                warn!("'C:Finish:{finish_flag}' flag should contain a valid direction.");
                continue;
            }
            writer.write(LoadRoomMessage {
                room: crate::Room(finish_flag[mid + 1..].into()),
                direction,
            });
            return;
        }
    }
}
