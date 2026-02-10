//! This module loads resources and display them accordingly.

mod sprite;
pub use sprite::*;

mod framerate;

pub mod background;
pub mod camera;
pub mod window;

mod info;

use bevy::prelude::*;

pub struct InfrResPlugin;

impl Plugin for InfrResPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationAtlasHandles>()
            .init_resource::<camera::VirtualResolution>()
            .init_resource::<window::WindowTitle>()
            .init_resource::<background::BackgroundRange>();
        app.add_systems(Update, (modify_animation, tick_animation));
        app.add_systems(Update, (select_object_animation,));
        app.add_systems(Last, framerate::control_framerate);
        app.add_systems(
            Startup,
            (
                camera::setup_camera,
                window::setup_window,
                background::setup_background,
            ),
        );
        app.add_systems(
            Update,
            (
                window::update_window,
                camera::update_camera,
                camera::update_resolution,
                camera::update_color_tinting,
                background::modify_background,
            ),
        );
        app.add_systems(
            Update,
            (
                background::update_background_tile,
                background::reset_background_range,
            )
                .run_if(in_state(infr_client::GlobalState::Game)),
        );
        app.add_systems(
            Update,
            (
                info::play_morph_effect,
                info::accelerate_loading_when_loaded,
            ),
        );
        app.add_systems(OnExit(infr_client::MapState::Loading), info::show_title);
        app.add_systems(OnEnter(infr_client::MapState::Loading), info::show_loading);
    }
}
