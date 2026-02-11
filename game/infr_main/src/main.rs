use bevy::prelude::*;
use bevy_ehttp::prelude::*;

fn main() {
    let run_path = std::path::Path::new(".").canonicalize().unwrap();
    App::new()
        .add_plugins((
            HttpPlugin,
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    file_path: run_path.to_string_lossy().into_owned(),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            infr_client::InfrClientPlugin,
            infr_res::InfrResPlugin,
            infr_level::InfrLevelPlugin,
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.insert_resource(Time::<Fixed>::from_duration(
                infr_client::config::CONFIG.client.frame_duration,
            ));
            commands.insert_resource(ClearColor(Color::BLACK));
        })
        .run();
}
