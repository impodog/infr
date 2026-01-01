//! Stores representation of game layout, loads external lua movements, and parses movements.

#![feature(generic_atomic)]

mod layout;
pub use layout::*;

pub mod script;
