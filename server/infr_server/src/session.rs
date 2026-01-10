use crate::prelude::*;

use std::sync::atomic::{self, AtomicU32};
use std::time::Duration;

/// Stores a game session, with object layouts and other added information.
#[derive(Debug, Clone)]
pub struct Session {
    pub meta: transfer::LevelMetadata,
    pub snapshots: HashMap<transfer::SnapshotId, Snapshot>,
    pub current_snapshot_id: transfer::SnapshotId,
    pub layout: Layout,
    pub input_count: u32,
    pub round: u32,
    pub last_refresh: Duration,
}
/// This is used for countering z3 errors(z3 doesn't contain Rc). Layouts may only be used under Mutex.
unsafe impl Send for Session {}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub objects: Vec<Object>,
    pub input_count: u32,
    pub round: u32,
}

fn current_time() -> Duration {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time should't be before epoch")
}

impl Session {
    /// Converts a solver-specific error to the serializable format.
    pub fn convert_error(&self, err: InfrError) -> transfer::ServerError {
        match err {
            InfrError::Contradiction(Contradiction { rules, axioms }) => {
                let rules = rules
                    .into_iter()
                    .filter_map(|rule_index| {
                        if let Some(rule) = self.layout.map.rules.get(rule_index) {
                            let range = rule.range();
                            Some((range.0.into(), range.1.into()))
                        } else {
                            log::warn!("Solver outputed non-existent rule index: {rule_index}.");
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                transfer::ServerError::Contradiction(transfer::Contradiction { rules, axioms })
            }
            InfrError::Overlap(coord) => transfer::ServerError::Overlap(coord.into()),
            InfrError::DifferentMovements(first, second) => {
                transfer::ServerError::DifferentMovements(first.into(), second.into())
            }
            InfrError::Script(err) => transfer::ServerError::ServerSide(err.to_string()),
            InfrError::NoSuchId(id) => transfer::ServerError::ServerSide(format!(
                "Script returned unknown object id: {id}"
            )),
            InfrError::IllFormed(movement) => transfer::ServerError::ServerSide(format!(
                "Script returned ill-formed movement: {movement:?}"
            )),
        }
    }
}

pub(crate) fn route_session(state: AppState) -> Router<AppState> {
    tokio::task::spawn(remove_expired_sessions(state));
    Router::new()
        .route("/load", post(load_session))
        .route("/meta", get(get_metadata))
        .route("/step", post(step))
        .route("/map", get(get_map))
        .route("/snap", post(take_snapshot))
        .route("/remove", post(remove_snapshot))
        .route("/revert", post(revert_snapshot))
        .route("/refresh", post(refresh_session))
}

async fn remove_expired_sessions(state: AppState) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(
            crate::config::STARTUP_CONFIG.refresh_interval,
        ))
        .await;
        let mut sessions = state.sessions.write().unwrap();
        let mut pending_remove = Vec::new();
        for (session_id, session) in sessions.iter() {
            if (current_time() - session.lock().unwrap().last_refresh).as_secs()
                > crate::config::STARTUP_CONFIG.refresh_interval
            {
                pending_remove.push(*session_id);
            }
        }
        for session_id in pending_remove.into_iter() {
            sessions.remove(&session_id);
        }
    }
}

static SESSION_ID: AtomicU32 = AtomicU32::new(1);

async fn load_session(
    state: State<AppState>,
    req: Json<transfer::LoadSessionRequest>,
) -> Response<Json<transfer::LoadSessionResponse>> {
    let transfer::LoadSessionRequest { path } = req.0;
    log::info!("Request to load session from: {path:?}");
    if !path.exists() {
        return Err(transfer::ServerError::BadRequest(format!("File not found: {path:?}")).into());
    }
    let content = tokio::fs::read_to_string(&path).await.map_err(|err| {
        transfer::ServerError::BadRequest(format!("Unable to read file {path:?}: {err}"))
    })?;
    let transfer::Level {
        meta,
        requirements,
        objects,
    } = toml::from_str(&content).map_err(|err| {
        transfer::ServerError::BadRequest(format!("Unable to parse level config: {err}"))
    })?;

    let mut layout = Layout::new();
    for object in objects.into_iter() {
        layout.map.push(Object::try_from(object)?);
    }

    let session = Session {
        meta,
        snapshots: HashMap::new(),
        current_snapshot_id: 1,
        layout,
        input_count: 0,
        round: 0,
        last_refresh: current_time(),
    };
    let session_id = SESSION_ID.fetch_add(1, atomic::Ordering::SeqCst);
    state
        .0
        .sessions
        .write()
        .unwrap()
        .insert(session_id, Arc::new(Mutex::new(session)));

    log::info!("Session {session_id} is loaded");

    Ok(transfer::LoadSessionResponse {
        id: session_id,
        requirements,
    }
    .into())
}

async fn get_metadata(
    state: State<AppState>,
    req: Json<transfer::GetMetadataRequest>,
) -> Response<Json<transfer::LevelMetadata>> {
    let session_id = req.0;
    let session = state.0.get_session(session_id)?;
    let meta = session.lock().unwrap().meta.clone();
    Ok(meta.into())
}

async fn get_map(
    state: State<AppState>,
    req: Json<transfer::GetMapRequest>,
) -> Response<Json<transfer::Map>> {
    let session_id = req.0;
    let session = state.0.get_session(session_id)?;
    let map = transfer::Map::from_map(&session.lock().unwrap().layout.map);
    Ok(map.into())
}

async fn step(
    state: State<AppState>,
    req: Json<transfer::SendStepRequest>,
) -> Response<Json<transfer::SendStepResponse>> {
    let transfer::SendStepRequest {
        session_id,
        direction,
    } = req.0;

    log::info!("Stepping session {session_id}");

    let session_ref = state.0.get_session(session_id)?;
    let mut session = session_ref.lock().unwrap();

    let direction = Option::<Direction>::from(direction);
    // This movement directly responds to player input, meaning that a new set of rounds begins.
    if direction.is_some() {
        session.round = 0;
        session.input_count += 1;
    }
    let signal = Signal {
        direction,
        round: session.round,
    };
    let movements = session
        .layout
        .step(signal, &state.lua, &state.scripts.read().unwrap())
        .map_err(|err| session.convert_error(err))?;

    // Round only increments if the move is actually performed.
    session.round += 1;

    Ok(transfer::SendStepResponse {
        movements: movements.into_iter().map(Into::into).collect::<Vec<_>>(),
    }
    .into())
}

async fn take_snapshot(
    state: State<AppState>,
    req: Json<transfer::TakeSnapshotRequest>,
) -> Response<Json<transfer::SnapshotId>> {
    let session_id = req.0;
    let session_ref = state.get_session(session_id)?;
    let mut session = session_ref.lock().unwrap();
    let snapshot_id = session.current_snapshot_id;
    session.current_snapshot_id += 1;
    let objects = session.layout.map.objects.clone();
    let snapshot = Snapshot {
        objects,
        input_count: session.input_count,
        round: session.round,
    };
    session.snapshots.insert(snapshot_id, snapshot);

    log::info!("Taken snapshot {snapshot_id} of session {session_id}");

    Ok(snapshot_id.into())
}

async fn remove_snapshot(
    state: State<AppState>,
    req: Json<transfer::RemoveSnapshotRequest>,
) -> Response<String> {
    let transfer::RemoveSnapshotRequest {
        session_id,
        snapshot_id,
    } = req.0;
    let session_ref = state.get_session(session_id)?;
    let mut session = session_ref.lock().unwrap();
    session.snapshots.remove(&snapshot_id).ok_or_else(|| {
        transfer::ServerError::BadRequest(format!("Unidentified snapshot id: {snapshot_id}"))
    })?;

    log::info!("Removed snapshot {snapshot_id} of session {session_id}");

    Ok(format!("Snapshot {snapshot_id} removed"))
}

async fn revert_snapshot(
    state: State<AppState>,
    req: Json<transfer::RemoveSnapshotRequest>,
) -> Response<String> {
    let transfer::RemoveSnapshotRequest {
        session_id,
        snapshot_id,
    } = req.0;
    let session_ref = state.get_session(session_id)?;
    let mut session = session_ref.lock().unwrap();
    let Snapshot {
        objects,
        input_count,
        round,
    } = session
        .snapshots
        .get(&snapshot_id)
        .ok_or_else(|| {
            transfer::ServerError::BadRequest(format!("Unidentified snapshot id: {snapshot_id}"))
        })?
        .clone();
    session.layout.map.objects = objects;
    session.input_count = input_count;
    session.round = round;

    log::info!("Reverted to snapshot {snapshot_id} of session {session_id}");

    Ok(format!("Snapshot {snapshot_id} reverted"))
}

async fn refresh_session(
    state: State<AppState>,
    req: Json<transfer::RefreshSessionRequest>,
) -> Response<&'static str> {
    let session_id = req.0;
    let session = state.get_session(session_id)?;
    session.lock().unwrap().last_refresh = current_time();
    Ok("Session refreshed")
}
