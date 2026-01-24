use std::path::PathBuf;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};

#[derive(Resource, Debug)]
pub struct Map {
    pub client: infr_client::Map,
}

#[derive(Message, Debug, Clone)]
pub struct LoadMap {
    pub path: PathBuf,
}

#[derive(Component, Debug)]
pub(crate) struct LoadMapTask {
    pub task: Option<Task<Result<Result<infr_client::Map, infr_transfer::ServerError>>>>,
}

pub(crate) fn on_load_map_request(mut reader: MessageReader<LoadMap>, mut commands: Commands) {
    let task_pool = AsyncComputeTaskPool::get();
    for request in reader.read() {
        let path = request.path.clone();
        let task = task_pool.spawn(async move { infr_client::Map::load(path).await });
        commands.spawn(LoadMapTask { task: Some(task) });
    }
}

pub(crate) fn poll_load_map_request(
    mut tasks: Query<(Entity, &mut LoadMapTask)>,
    mut commands: Commands,
    mut writer: crate::ErrorWriter,
    mut morph_writer: MessageWriter<crate::MorphSessionMessage>,
    mut game_state: ResMut<NextState<crate::GameState>>,
    mut state: ResMut<NextState<infr_res::InfrState>>,
    map_size: Res<crate::CurrentMapSize>,
) -> Result<()> {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(task) = &mut task.task
            && let Some(result) = check_ready(task)
        {
            let result = crate::try_report!(result?, writer);

            // Sends the morph message to be parsed next frame.
            let mut x_min = f32::INFINITY;
            let mut y_min = f32::INFINITY;
            let mut x_max = f32::NEG_INFINITY;
            let mut y_max = f32::NEG_INFINITY;
            for (_id, object) in result.objects.iter() {
                x_min = x_min.min(object.coord.0 as f32);
                x_max = x_max.max(object.coord.0 as f32);
                y_min = y_min.min(object.coord.1 as f32);
                y_max = y_max.max(object.coord.1 as f32);
            }
            let x_len = (x_max - x_min + map_size.0.max.x - map_size.0.min.x) / 2.0;
            let y_len = (y_max - y_min + map_size.0.max.y - map_size.0.min.y) / 2.0;
            morph_writer.write(crate::MorphSessionMessage {
                offset: Vec2::new(x_len, y_len),
            });

            // Set states.
            game_state.set(crate::GameState::Morph);
            state.set(infr_res::InfrState::Game);
            // Update bevy world.
            commands.insert_resource(Map { client: result });
            commands.entity(entity).despawn();
        }
    }
    Ok(())
}
