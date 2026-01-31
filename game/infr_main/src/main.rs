use bevy::prelude::*;
use bevy_ehttp::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HttpPlugin, infr_client::InfrClientPlugin))
        .run();
}
