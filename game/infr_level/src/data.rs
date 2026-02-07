use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A room is an id linked to a level from the server.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
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
    /// The delta to the center, used for in-game maps.
    pub delta: (i32, i32),
}

#[derive(Serialize, Deserialize, Resource, Debug, Clone, Default)]
pub struct UserData {
    pub rooms: HashMap<Room, RoomData>,
}

pub(crate) fn init_user_data(mut commands: Commands) -> Result<()> {
    match std::fs::read_to_string(&infr_client::config::CONFIG.user_data_path) {
        Ok(content) => {
            // Here the error is passed on to bevy, to quickly exit the game for corrupted save files.
            let data: UserData = ron::from_str(&content)?;
            commands.insert_resource(data);
        }
        Err(err) => {
            warn!("Unable to load user data: {err}. Using default..");
            commands.init_resource::<UserData>();
        }
    }
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
