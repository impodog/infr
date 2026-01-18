mod object;
pub use object::*;

mod map;
pub use map::*;

use bevy::prelude::*;
use infr_res::when_state;

pub struct InfrGamePlugin;

impl Plugin for InfrGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_object_transform,).run_if(when_state!(Game)));
        app.add_systems(Update, (on_load_map_request,));
    }
}
