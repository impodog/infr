//! Defines types for parsing level config.

use std::collections::HashSet;

use crate::*;

/// Used only in `Level` for constructing a object, opposed to retrieving it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelObject {
    pub direction: Direction,
    pub coord: Coord,
    pub group: String,
    #[serde(default)]
    pub flags: Vec<String>,
}

/// The entry struct for parsing level configuration files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Level {
    pub meta: LevelMetadata,
    pub objects: Vec<LevelObject>,
    #[serde(default)]
    pub requirements: Vec<String>,
    // TODO: Add transition config
}

/// The data that needs to be transferred throughout the program unmodified.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LevelMetadata {
    #[serde(default = "default_title")]
    pub title: String,
    /// The background sprite to use.
    #[serde(default)]
    pub background: Option<String>,
    /// Chooses a color tinting style for the level.
    #[serde(default)]
    pub tinting: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub flags: HashSet<String>,
    #[serde(default)]
    pub axioms: Vec<String>,
}

fn default_title() -> String {
    "Untitled".into()
}
