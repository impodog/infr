pub mod config;
mod map;
mod object;
pub mod prelude;

pub use map::*;
pub use object::*;

use bevy::{prelude::*, time::common_conditions::on_timer};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalState {
    #[default]
    Game,
    Menu,
}

pub struct InfrClientPlugin;

impl Plugin for InfrClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PositionCenter>()
            .init_resource::<CurrentSession>()
            .init_resource::<AnyObjectMoving>()
            .init_resource::<Map>();
        app.init_state::<GlobalState>();
        app.add_message::<LoadMap>().add_message::<LevelError>();
        app.add_systems(Update, (convert_position,));
        app.add_systems(PreUpdate, (start_load_map,));
        app.add_systems(
            FixedUpdate,
            (move_object,).run_if(in_state(GlobalState::Game)),
        );
        app.add_systems(
            FixedUpdate,
            refresh_session.run_if(on_timer(std::time::Duration::from_millis(
                config::CONFIG.server.refresh_interval,
            ))),
        );
    }
}
