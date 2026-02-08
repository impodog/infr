use bevy::prelude::*;
use bevy_ehttp::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            HttpPlugin,
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            infr_client::InfrClientPlugin,
            infr_res::InfrResPlugin,
            infr_level::InfrLevelPlugin,
        ))
        .run();
}
