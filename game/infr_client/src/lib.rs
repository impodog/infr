pub mod config;
pub mod prelude;

mod map;
pub use map::*;

mod object;
pub use object::*;

mod step;
pub use step::*;

use bevy::{prelude::*, time::common_conditions::on_timer};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalState {
    #[default]
    Game,
    Menu,
}
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapState {
    #[default]
    /// Loading map contents.
    Loading,
    /// When requesting and animating object steps.
    Stepping,
    /// Player can only input in this state.
    Free,
    /// The map has faced an error / player requested revert to snapshot, and requires reload.
    Reloading,
}

pub struct InfrClientPlugin;

impl Plugin for InfrClientPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<bevy_ehttp::HttpRequest, prelude::RequestMarker>();
        app.init_resource::<PositionCenter>()
            .init_resource::<CurrentSession>()
            .init_resource::<LevelMetadata>()
            .init_resource::<AnyObjectMoving>()
            .init_resource::<Map>()
            .init_resource::<CurrentMovements>()
            .init_resource::<CurrentObjects>()
            .init_resource::<CurrentRound>()
            .init_resource::<PlayerInputQueue>()
            .init_resource::<ActionCount>()
            .init_resource::<PreInputSnapshot>()
            .init_resource::<RuleRanges>();
        app.init_state::<GlobalState>().init_state::<MapState>();
        app.add_message::<LoadMap>()
            .add_message::<ReloadMap>()
            .add_message::<LevelError>()
            .add_message::<PlayerDirection>()
            .add_message::<PlayerAction>()
            .add_message::<ActionFinished>();
        app.add_systems(FixedUpdate, (convert_position,));
        app.add_systems(PreUpdate, (start_load_map, remove_past_objects));
        app.add_systems(
            FixedUpdate,
            (move_object,).run_if(in_state(GlobalState::Game)),
        );
        app.add_systems(
            FixedUpdate,
            refresh_session.run_if(on_timer(std::time::Duration::from_secs(
                config::CONFIG.server.refresh_interval,
            ))),
        );
        app.add_systems(
            PostUpdate,
            (finish_load_map,).run_if(in_state(MapState::Loading)),
        );
        app.add_systems(PreUpdate, (listen_keyboard_input,));
        app.add_systems(
            PreUpdate,
            (
                (test_action_finished, read_player_input)
                    .chain()
                    .run_if(in_state(MapState::Free)),
                queue_player_input,
            )
                .run_if(in_state(GlobalState::Game)),
        );
        app.add_systems(
            Update,
            (send_animations, finish_animations)
                .chain()
                .run_if(in_state(MapState::Stepping)),
        );
        app.add_observer(observe_movement_event);
        app.add_systems(Update, (reload_on_error, handle_reload_map).chain());
        app.add_systems(Last, restart_on_input);
    }
}
