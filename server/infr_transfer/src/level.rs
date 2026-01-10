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
    pub meta: LevelMetadata,
    pub objects: Vec<LevelObject>,
    pub requirements: Vec<String>,
    // TODO: Add transition config
}

/// The data that needs to be transferred throughout the program unmodified.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelMetadata {
    pub title: String,
}
