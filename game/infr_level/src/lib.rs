mod data;
pub use data::*;

mod rooms;
pub use rooms::*;

use bevy::prelude::*;

pub struct InfrLevelPlugin;

impl Plugin for InfrLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadRoomMessage>();
        app.init_resource::<CurrentRoom>();
        app.add_systems(PreStartup, init_user_data);
        app.add_systems(Last, save_user_data);
        app.add_systems(Startup, startup_load_user);

        app.add_systems(PreUpdate, load_room);
        app.add_systems(OnExit(infr_client::MapState::Loading), finish_load_room);
        app.add_systems(OnEnter(infr_client::MapState::Free), switch_to_next_room);
    }
}
