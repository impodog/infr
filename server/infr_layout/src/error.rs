use crate::layout::*;
use infr_solver::*;

/// Errors that occur when the layout is parsing movements.
pub enum InfrError {
    /// A logical contradiction defined by rules and axioms.
    Contradiction(infr_solver::Contradiction),
    /// Two solid objects in the same tile.
    Overlap(Coord),
    /// The external script returns an unexpected error.
    Script(mlua::Error),
    /// One object moves in different ways.
    DifferentMovements(Vec<Movement>),
    /// Ill-formed movement.
    IllFormed(Movement),
    /// No such object id.
    NoSuchId(u32),
}
impl From<infr_solver::Contradiction> for InfrError {
    fn from(value: infr_solver::Contradiction) -> Self {
        Self::Contradiction(value)
    }
}
impl From<Coord> for InfrError {
    fn from(value: Coord) -> Self {
        Self::Overlap(value)
    }
}
impl From<mlua::Error> for InfrError {
    fn from(value: mlua::Error) -> Self {
        Self::Script(value)
    }
}
