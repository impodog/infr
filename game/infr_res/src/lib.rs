//! Loads external config/resources using Bevy interface.

mod images;
pub use images::*;

mod client;
pub use client::*;

use bevy::prelude::*;
pub struct InfrResPlugin;
impl Plugin for InfrResPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationAtlasHandles>()
            .add_systems(PreUpdate, (modify_animation,))
            .add_systems(Update, (tick_animation,));
    }
}
