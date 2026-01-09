//! Transfer layers between the server and client programs.
//!
//! This crate should be kept minimum for better efficiency.
//!
//! Please note that the names collide with those defined in server details, so you must use this crate under a module name.

#[cfg(feature = "server")]
mod conv;

use serde::{Deserialize, Serialize};

/// The direction which the object is facing. 1-Right 2-Up 3-Left 4-Right Others-No direction
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Direction(pub u8);

/// The coordinates where tiles are.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord(pub i32, pub i32);

/// The type for Object ids. This is globally unique even in different gameplays.
pub type Id = u32;
/// The identifier for axioms, used in error reporting.
pub type AxiomId = u32;

/// Struct used for retrieving relevant information of an object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object {
    pub id: Id,
    pub direction: Direction,
    pub coord: Coord,
    /// All the groups that the object is in.
    pub groups: Vec<String>,
    /// The main group that the object is in, this is used for selecting the right sprite.
    pub nature: String,
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
    /// A server side error. Clients may directly report the error message.
    ServerSide(String),
}
