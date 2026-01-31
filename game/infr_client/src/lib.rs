pub mod config;
mod map;
mod object;
pub mod prelude;

pub use map::*;
pub use object::*;

use bevy::prelude::*;

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
            .init_resource::<AnyObjectMoving>();
        app.init_state::<GlobalState>();
        app.add_systems(Update, (convert_position,));
        app.add_systems(FixedPreUpdate, (start_load_map,));
        app.add_systems(
            FixedUpdate,
            (move_object,).run_if(in_state(GlobalState::Game)),
        );
    }
}
