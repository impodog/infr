//! Defines types for parsing level config.

use crate::*;

/// Used only in `Level` for constructing a object, opposed to retrieving it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelObject {
    pub direction: Direction,
    pub coord: Coord,
    pub group: String,
}

/// The entry struct for parsing level configuration files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Level {
    pub title: String,
    pub objects: Vec<LevelObject>,
    pub requirements: Vec<String>,
    // TODO: Add transition config
}
