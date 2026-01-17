mod camera;
pub use camera::*;

use bevy::prelude::*;

pub struct InfrRenderPlugin;
impl Plugin for InfrRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VirtualResolution>()
            .add_systems(Startup, (setup_camera,));
    }
}
