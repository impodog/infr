use bevy::prelude::*;
use infr_res::Animation;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            infr_render::InfrRenderPlugin,
            infr_res::InfrResPlugin,
        ))
        .add_systems(Update, (spawn_empty, move_animation))
        .run();
}

fn spawn_empty(mut commands: Commands, mut flag: Local<bool>) {
    if !*flag {
        commands.spawn(Animation {
            name: "Empty".to_owned(),
            size: Vec2::new(32.0, 32.0),
        });
        *flag = true;
    }
}

fn move_animation(mut query: Query<(&mut Transform), With<Animation>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 0.1;
    }
}
