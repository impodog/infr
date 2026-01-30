mod map;
pub use map::*;

mod object;
pub use object::*;

pub mod prelude;

pub mod config;

use bevy::prelude::*;

pub struct InfrClientPlugin;

impl Plugin for InfrClientPlugin {
    fn build(&self, app: &mut App) {}
}
