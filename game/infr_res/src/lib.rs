//! This module loads resources and display them accordingly.

mod sprite;
pub use sprite::*;

mod framerate;

pub mod camera;

pub mod window;

use bevy::prelude::*;

pub struct InfrResPlugin;

impl Plugin for InfrResPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationAtlasHandles>()
            .init_resource::<camera::VirtualResolution>()
            .init_resource::<window::WindowTitle>();
        app.add_systems(Update, (modify_animation, tick_animation));
        app.add_systems(FixedUpdate, (select_object_animation,));
        app.add_systems(FixedLast, framerate::control_framerate);
        app.add_systems(Startup, (camera::setup_camera, window::setup_window));
        app.add_systems(
            Update,
            (
                window::update_window,
                camera::update_camera,
                camera::update_resolution,
            ),
        );
    }
}
