use std::{collections::HashMap, path::PathBuf};

use crate::prelude::*;

/// Client form of an game object, more focused on rendering the right sprite.
#[derive(Debug, Clone)]
pub struct Object {
    pub id: transfer::ObjectId,
    pub coord: transfer::Coord,
    pub direction: transfer::Direction,
    /// The key to current sprite atlas, if existing.
    pub sprite: Option<String>,
    /// The nature of the object, given when the object is created.
    pub nature: String,
}

pub type MovementMap = HashMap<ObjectId, transfer::Movement>;

/// While the snapshots from the server stores objects, the client should store movements, for better object sprite selection.
#[derive(Debug, Clone)]
struct Snapshot {
    movements: MovementMap,
}

#[derive(Debug, Clone)]
pub struct Map {
    /// The connection to the server from this map(session in server speech).
    pub client: Client,
    pub session_id: SessionId,
    /// All objects stored in a convenient id map.
    pub objects: HashMap<ObjectId, Object>,
    /// Movements from the previous step.
    pub movements: MovementMap,
    snapshots: HashMap<SnapshotId, Snapshot>,
}

impl Map {
    /// Loads a level from a given level path.
    pub async fn load(path: PathBuf) -> Result<Result<Self, transfer::ServerError>> {
        let client = Client::new();

        let response = client
            .post(crate::config::STARTUP_CONFIG.url("session/load"))
            .json(&transfer::LoadSessionRequest { path })
            .send()
            .await?;
        let id: SessionId = match response.status() {
            StatusCode::OK => response.json().await?,
            _ => {
                let error: transfer::ServerError = response.json().await?;
                return Ok(Err(error));
            }
        };
        let mut map = Map {
            client,
            session_id: id,
            objects: HashMap::new(),
            movements: HashMap::new(),
            snapshots: HashMap::new(),
        };
        if let Err(server_error) = map.reload_objects().await? {
            return Ok(Err(server_error));
        }
        Ok(Ok(map))
    }

    const fn direction_name(direction: transfer::Direction) -> &'static str {
        match direction.0 {
            1 => "Right",
            2 => "Up",
            3 => "Left",
            4 => "Down",
            _ => "Nodir",
        }
    }

    pub fn decide_sprite(
        direction: transfer::Direction,
        group: &str,
        movement: Option<&transfer::Movement>,
    ) -> Option<String> {
        use infr_transfer::MoveManner;

        macro_rules! try_return {
            ($name: expr) => {
                if crate::config::SPRITE_CONFIG
                    .map
                    .get($name)
                    .is_some_and(|list| !list.is_empty())
                {
                    return Some(group.into());
                }
            };
        }

        let direction = Self::direction_name(direction);

        match movement {
            Some(movement) => match movement.manner {
                MoveManner::Add => {
                    try_return!(&format!("{group}{direction}Add"));
                    try_return!(&format!("{group}Add"));
                }
                MoveManner::Swipe(direction) => {
                    let direction = Self::direction_name(direction);
                    try_return!(&format!("{group}{direction}Swipe"));
                    try_return!(&format!("{group}Swipe"));
                }
                MoveManner::Remove => {
                    try_return!(&format!("{group}{direction}Remove"));
                    try_return!(&format!("{group}Remove"));
                }
                MoveManner::Teleport => {
                    try_return!(&format!("{group}{direction}Teleport"));
                    try_return!(&format!("{group}Teleport"));
                }
            },
            None => {}
        }
        try_return!(&format!("{group}{direction}"));
        try_return!(group);
        None
    }

    /// Gets all objects from the server, and refresh the whole setup.
    ///
    /// This will read `Self::movements` and if a snapshot is reverted, you must change movements first.
    pub async fn reload_objects(&mut self) -> Result<Result<(), ServerError>> {
        let response = self
            .client
            .get(crate::config::STARTUP_CONFIG.url("session/map"))
            .json(&self.session_id)
            .send()
            .await?;
        let map = if response.status() == StatusCode::OK {
            let map: transfer::Map = response.json().await?;
            map
        } else {
            let error: transfer::ServerError = response.json().await?;
            return Ok(Err(error));
        };
        self.objects.clear();
        for object in map.0.into_iter() {
            let transfer::Object {
                id,
                coord,
                direction,
                groups: _,
                nature,
            } = object;
            let sprite = Self::decide_sprite(direction, &nature, self.movements.get(&id));
            self.objects.insert(
                id,
                Object {
                    id,
                    coord,
                    direction,
                    nature,
                    sprite,
                },
            );
        }
        Ok(Ok(()))
    }

    /// Stores a snapshot to be later reverted to, returning its id.
    pub async fn take_snapshot(&mut self) -> Result<Result<SnapshotId, ServerError>> {
        let response = self
            .client
            .post(crate::config::STARTUP_CONFIG.url("session/snap"))
            .json(&self.session_id)
            .send()
            .await?;
        if response.status() == StatusCode::OK {
            let id: SnapshotId = response.json().await?;
            let snapshot = Snapshot {
                movements: self.movements.clone(),
            };
            self.snapshots.insert(id, snapshot);
            Ok(Ok(id))
        } else {
            Ok(Err(response.json().await?))
        }
    }

    /// Reverts to a stored snapshot. If the snapshot doesn't exist, a server error is returned(though it happens in the client).
    pub async fn revert_snapshot(
        &mut self,
        snapshot_id: SnapshotId,
    ) -> Result<Result<(), ServerError>> {
        let movements = if let Some(snapshot) = self.snapshots.get(&snapshot_id) {
            snapshot.movements.clone()
        } else {
            return Ok(Err(ServerError::BadRequest(format!(
                "Unknown snapshot id {snapshot_id} for session {}",
                self.session_id
            ))));
        };
        let response = self
            .client
            .post(crate::config::STARTUP_CONFIG.url("/session/revert"))
            .json(&transfer::RevertSnapshotRequest {
                session_id: self.session_id,
                snapshot_id,
            })
            .send()
            .await?;
        if response.status() == StatusCode::OK {
            self.movements = movements;
            self.reload_objects().await
        } else {
            Ok(Err(response.json().await?))
        }
    }

    /// Removes a stored snapshot to save some resources. Note that this doesn't revert to that snapshot.
    /// If the snapshot doesn't exist, a server error is returned(though it happens in the client).
    pub async fn remove_snapshot(
        &mut self,
        snapshot_id: SnapshotId,
    ) -> Result<Result<(), ServerError>> {
        if self.snapshots.remove(&snapshot_id).is_none() {
            return Ok(Err(ServerError::BadRequest(format!(
                "Unknown snapshot id {snapshot_id} for session {}",
                self.session_id
            ))));
        }
        let response = self
            .client
            .post(crate::config::STARTUP_CONFIG.url("/session/remove"))
            .json(&transfer::RemoveSnapshotRequest {
                session_id: self.session_id,
                snapshot_id,
            })
            .send()
            .await?;
        if response.status() == StatusCode::OK {
            Ok(Ok(()))
        } else {
            Ok(Err(response.json().await?))
        }
    }

    /// Refreshes session time countdown, to keep the session alive.
    /// Note that this request should not fail if the user properly refreshed the session on given interval.
    /// Thus, `ServerError` is treated as any other error here.
    pub async fn refresh_session(&self) -> Result<()> {
        self.client
            .post(crate::config::STARTUP_CONFIG.url("/session/refresh"))
            .json(&self.session_id)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
