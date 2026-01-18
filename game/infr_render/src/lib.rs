mod camera;
pub use camera::*;

mod window;
pub use window::*;

mod particles;
pub use particles::*;

use bevy::prelude::*;

pub struct InfrRenderPlugin;
impl Plugin for InfrRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VirtualResolution>()
            .init_resource::<WindowTitle>()
            .add_systems(Startup, (setup_camera, setup_window))
            .add_systems(Update, (update_window, update_camera));
    }
}
