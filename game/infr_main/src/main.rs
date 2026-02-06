use bevy::prelude::*;
use bevy_ehttp::prelude::*;

fn main() {
    App::new()
        .add_plugins((HttpPlugin, DefaultPlugins, infr_client::InfrClientPlugin))
        .add_systems(PreStartup, send_load_map)
        .run();
}

fn send_load_map(mut writer: MessageWriter<infr_client::LoadMap>, mut flag: Local<bool>) {
    if !*flag {
        *flag = true;
        writer.write(infr_client::LoadMap {
            pack: "test".into(),
            name: "hello".into(),
        });
    }
}
