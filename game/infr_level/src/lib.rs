mod data;
pub use data::*;

use bevy::prelude::*;

pub struct InfrLevelPlugin;

impl Plugin for InfrLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_user_data);
        app.add_systems(Last, save_user_data);
    }
}
