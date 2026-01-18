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
) -> Result<()> {
    let task_pool = AsyncComputeTaskPool::get();
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(task) = &mut task.task
            && let Some(result) = check_ready(task)
        {
            let result = result?;
            // TODO: Add server error handling.
        }
    }
    Ok(())
}
