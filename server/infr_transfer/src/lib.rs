//! Transfer layers between the server and client programs.
//!
//! This crate should be kept minimum for better efficiency.
//!
//! Please note that the names collide with those defined in server details, so you must use this crate under a module name.

#![feature(never_type)]

#[cfg(feature = "server")]
mod conv;

mod level;
pub use level::*;

use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

/// The direction which the object is facing. 1-Right 2-Up 3-Left 4-Down Others-No direction
#[derive(
    Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Direction(pub u8);
impl Direction {
    pub const UNKNOWN: Self = Self(0);
    pub const RIGHT: Self = Self(1);
    pub const UP: Self = Self(2);
    pub const LEFT: Self = Self(3);
    pub const DOWN: Self = Self(4);
    /// Gets the coordinates of unit vector in that direction.
    pub fn delta(&self) -> Coord {
        match self.0 {
            1 => Coord(1, 0),
            2 => Coord(0, 1),
            3 => Coord(-1, 0),
            4 => Coord(0, -1),
            _ => Coord(0, 0),
        }
    }

    /// Returns if the direction is normal (0, 1, 2, 3, 4). This will include UNKNOWN.
    pub fn is_normal(&self) -> bool {
        self.0 <= 4
    }

    /// Returns if the direction is one of the four directions (1, 2, 3, 4). This will not include UNKNOWN.
    pub fn is_some(&self) -> bool {
        self.0 <= 4 && self.0 != 0
    }
}
impl FromStr for Direction {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let direction = match s {
            "Right" => Self::RIGHT,
            "Up" => Self::UP,
            "Left" => Self::LEFT,
            "Down" => Self::DOWN,
            _ => Self::default(),
        };
        Ok(direction)
    }
}

/// The coordinates where tiles are.
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Coord(pub i32, pub i32);
impl std::ops::Add for Coord {
    type Output = Coord;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

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
    /// Corresponds to flags stored in the server.
    pub flags: Vec<String>,
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
    /// The client didn't step until empty movements is returned before giving a new input.
    InputRefused,
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

pub type GetMetadataRequest = SessionId;

pub type GetSessionStatusRequest = SessionId;
/// Returned by the server, the internal status the session stores.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionStatus {
    pub snapshot_count: usize,
    pub input_count: u32,
    pub round: u32,
    pub can_input: bool,
}

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
    /// When there is a player input, a snapshot is taken before any movements performed.
    pub snapshot_id: Option<SnapshotId>,
}

pub type GetMapRequest = SessionId;

/// Gets some of the objects requested to improve efficiency.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetObjectsRequest {
    pub session_id: SessionId,
    pub objects: Vec<ObjectId>,
    /// When set to true, the server will add extra objects whose features changed.
    pub add_changed: bool,
}

pub type SnapshotId = u32;
pub type TakeSnapshotRequest = SessionId;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveSnapshotRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevertSnapshotRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
}

pub type RefreshSessionRequest = SessionId;
