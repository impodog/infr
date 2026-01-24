mod object;
pub use object::*;

mod map;
pub use map::*;

mod errors;
pub use errors::*;

mod morph;
pub use morph::*;

use bevy::prelude::*;
use infr_res::when_state;

#[derive(States, Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    NotGame,
    Morph,
    Level,
}
fn erase_game_state(
    mut game_state: ResMut<NextState<GameState>>,
    state: Res<State<infr_res::InfrState>>,
) {
    if state.is_changed() && *state.get() != infr_res::InfrState::Game {
        game_state.set(GameState::NotGame);
    }
}

pub struct InfrGamePlugin;

impl Plugin for InfrGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentMapSize>();
        app.add_systems(Update, (update_object_transform,).run_if(when_state!(Game)));
        app.add_systems(Update, (on_load_map_request,));
        app.add_systems(Update, (handle_messages,));
        app.add_systems(Update, (move_camera,).run_if(when_state!(GameState, Morph)));
        app.add_systems(
            PostUpdate,
            (spawn_matching_objects, spawn_object)
                .chain()
                .run_if(when_state!(GameState, Morph)),
        );
        app.add_systems(PostUpdate, (poll_load_map_request,));
        app.add_systems(PostUpdate, (erase_game_state,).run_if(when_state!(Game)));
    }
}
