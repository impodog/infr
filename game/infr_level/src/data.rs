use bevy::prelude::*;
use infr_client::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

macro_rules! default_wrapper {
    ($struct: ident, $internal: ident, $default: expr) => {
        #[derive(Serialize, Deserialize, Debug, Clone, Deref, DerefMut)]
        pub struct $struct(pub $internal);
        impl Default for $struct {
            fn default() -> Self {
                Self(($default).into())
            }
        }
    };
}

/// A room is an id linked to a level from the server.
#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, Default,
)]
pub struct Room(pub String);

impl From<&Room> for infr_client::LoadMap {
    fn from(value: &Room) -> Self {
        if let Some(mid) = value.find('.') {
            Self {
                pack: value[0..mid].to_owned(),
                name: value[mid + 1..].to_owned(),
            }
        } else {
            Self {
                pack: "Default".to_owned(),
                name: value.0.clone(),
            }
        }
    }
}

/// Stores user-specific room(level) data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomData {
    /// The coordinates to the center, used for in-game maps.
    pub coord: transfer::Coord,
}

default_wrapper!(UserCurrentRoom, Room, Room("entrance.wall_path".to_owned()));

#[derive(Serialize, Deserialize, Resource, Debug, Clone, Default)]
pub struct UserData {
    #[serde(default)]
    pub rooms: HashMap<Room, RoomData>,
    #[serde(skip)]
    pub location: HashMap<transfer::Coord, Room>,
    #[serde(default)]
    pub current_room: UserCurrentRoom,
}

impl UserData {
    /// In runtime, adds a visited room to the user data. This will ensure correct mappings.
    pub fn add_room(&mut self, room: Room, coord: transfer::Coord) {
        self.rooms.insert(room.clone(), RoomData { coord });
        self.location.insert(coord, room);
    }

    /// Adds correct mapping to existing user data.
    /// This is used when reading user data from file.
    pub fn add_mappings(&mut self) {
        let mut location = HashMap::new();
        for (room, data) in self.rooms.iter() {
            location.insert(data.coord, room.clone());
        }
        self.location = location;
    }
}

pub(crate) fn init_user_data(mut commands: Commands) -> Result<()> {
    let mut data: UserData =
        match std::fs::read_to_string(&infr_client::config::CONFIG.user_data_path) {
            Ok(content) => {
                // Here the error is passed on to bevy, to quickly exit the game for corrupted save files.
                ron::from_str(&content)?
            }
            Err(err) => {
                warn!("Unable to load user data: {err}. Using default..");
                Default::default()
            }
        };
    data.add_mappings();
    commands.insert_resource(data);
    Ok(())
}

pub(crate) fn save_user_data(
    mut reader: MessageReader<AppExit>,
    user: Res<UserData>,
) -> Result<()> {
    if reader.read().next().is_some() {
        let content = ron::to_string(user.as_ref())?;
        std::fs::write(&infr_client::config::CONFIG.user_data_path, content)?;
    }
    Ok(())
}
