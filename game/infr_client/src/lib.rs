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
}

pub struct InfrClientPlugin;

impl Plugin for InfrClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PositionCenter>()
            .init_resource::<CurrentSession>()
            .init_resource::<LevelMetadata>()
            .init_resource::<AnyObjectMoving>()
            .init_resource::<Map>()
            .init_resource::<CurrentMovements>()
            .init_resource::<CurrentObjects>()
            .init_resource::<CurrentRound>()
            .init_resource::<PlayerInputQueue>();
        app.init_state::<GlobalState>().init_state::<MapState>();
        app.add_message::<LoadMap>()
            .add_message::<LevelError>()
            .add_message::<PlayerDirection>();
        app.add_systems(Update, (convert_position,));
        app.add_systems(PreUpdate, (start_load_map, remove_past_objects));
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
        app.add_systems(
            FixedPostUpdate,
            (finish_load_map,).run_if(in_state(MapState::Loading)),
        );
        app.add_systems(FixedPreUpdate, (listen_keyboard_input,));
        app.add_systems(
            FixedUpdate,
            (read_player_input, queue_player_input).run_if(in_state(GlobalState::Game)),
        );
        app.add_systems(
            FixedUpdate,
            (send_animations, finish_animations)
                .chain()
                .run_if(in_state(MapState::Stepping)),
        );
        app.add_observer(observe_movement_event);
    }
}
