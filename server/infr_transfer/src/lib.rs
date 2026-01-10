//! Transfer layers between the server and client programs.
//!
//! This crate should be kept minimum for better efficiency.
//!
//! Please note that the names collide with those defined in server details, so you must use this crate under a module name.

#[cfg(feature = "server")]
mod conv;

mod level;
pub use level::*;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The direction which the object is facing. 1-Right 2-Up 3-Left 4-Right Others-No direction
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Direction(pub u8);

/// The coordinates where tiles are.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord(pub i32, pub i32);

/// The type for Object ids. This is globally unique even in different gameplays.
pub type ObjectId = u32;
/// The identifier for axioms, used in error reporting.
pub type AxiomId = u32;
/// The identifier for sessions that stores game states.
pub type SessionId = u32;

/// Struct used for retrieving relevant information of an object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object {
    pub id: ObjectId,
    pub direction: Direction,
    pub coord: Coord,
    /// All the groups that the object is in.
    pub groups: Vec<String>,
    /// The main group that the object is in, this is used for selecting the right sprite.
    pub nature: String,
}

/// Only used in server outputs, corresponds to `infr_layout::Manner`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoveManner {
    Swipe(Direction),
    Teleport,
    Remove,
    Add,
}
/// Only used in server outputs, corresponds to `infr_layout::Movement`.
/// Clients may use this to play animations and modify game states.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Movement {
    /// The object that performs this movement. If manner is "Add", this is a fresh object id.
    pub object: ObjectId,
    /// Manner of the movement. This is used for better animation styles.
    pub manner: MoveManner,
    /// The target of the movement.
    pub dest: Coord,
}

/// Describes all the object in a map.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map(pub Vec<Object>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contradiction {
    /// The range where the rules involved in the contradiction are.
    pub rules: Vec<(Coord, Coord)>,
    /// Axioms ids involved in the contradiction.
    pub axioms: Vec<AxiomId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerError {
    /// Reports a contradiction in the logic solver.
    Contradiction(Contradiction),
    /// Two solid objects overlap.
    Overlap(Coord),
    DifferentMovements(Movement, Movement),
    /// A server side error. Clients may directly report the error message.
    ServerSide(String),
    BadRequest(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddScriptRequest {
    /// The readable name of this script.
    pub name: String,
    /// The path to the script.
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadSessionRequest {
    /// The path to the configuration file.
    pub path: PathBuf,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadSessionResponse {
    /// The id of the newly load session.
    pub id: SessionId,
    /// The requirements that the level config demands.
    /// Clients must decide which files to load and send them via /scripts/add, before performing any actions on the session.
    pub requirements: Vec<String>,
}

pub type GetMetadataRequest = SessionId;

/// Requests the server to step forward, with or without player input.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendStepRequest {
    pub session_id: SessionId,
    /// The direction of the step. If no direction, the step is a subsequent one, or it directly responds to player input.
    pub direction: Direction,
}
/// This is returned if successful, storing the movements performed.
///
/// Please note that there might be subsequent moves, defined by the script,
/// and that the next move may return errors if this move makes it logically unsound.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendStepResponse {
    pub movements: Vec<Movement>,
}
