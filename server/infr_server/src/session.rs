use crate::prelude::*;

use std::sync::atomic::{self, AtomicU32};
use std::time::Duration;

/// Stores a game session, with object layouts and other added information.
#[derive(Debug, Clone)]
pub struct Session {
    pub meta: transfer::LevelMetadata,
    /// The script requirements provided by the level config.
    pub requirements: Vec<String>,

    pub snapshots: HashMap<transfer::SnapshotId, Snapshot>,
    pub current_snapshot_id: transfer::SnapshotId,

    pub layout: Layout,
    pub input_count: u32,
    pub round: u32,
    /// Whether the session can accept input from the player. This is set to false when a input is provided,
    /// and set to true again if empty movement vec is returned(all movements are parsed).
    pub can_input: bool,
    /// Whether the `Layout::init_map` has already been called and no longer needs to be called again.
    pub initialized: bool,
    /// Used for detecting expired sessions.
    pub last_refresh: Duration,
}
/// This is used for countering z3 errors(z3 doesn't contain Rc). Layouts may only be used under Mutex.
unsafe impl Send for Session {}

/// A simplified session, containing fields that change by step.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub objects: Vec<Object>,
    pub input_count: u32,
    pub round: u32,
    pub can_input: bool,
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

    /// Attempts to initialize the map, if not already.
    pub fn initialize(&mut self) -> Result<(), transfer::ServerError> {
        if self.initialized {
            return Ok(());
        }
        // As the map is initialized, groups will be proven, allowing /map action.
        self.layout
            .init_map()
            .map_err(|err| self.convert_error(err))?;
        self.layout.map.revert();
        self.initialized = true;
        Ok(())
    }
}

pub(crate) fn route_session(state: AppState) -> Router<AppState> {
    tokio::task::spawn(remove_expired_sessions(state));
    Router::new()
        .route("/load", post(load_session))
        .route("/import", post(import_scripts))
        .route("/meta", get(get_metadata))
        .route("/status", get(get_session_status))
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
            log::info!("Removing expired session {session_id}");
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
        objects,
        requirements,
    } = toml::from_str(&content).map_err(|err| {
        transfer::ServerError::BadRequest(format!("Unable to parse level config: {err}"))
    })?;

    let mut layout = Layout::new();
    for object in objects.into_iter() {
        layout.map.push(Object::try_from(object)?);
    }

    let session = Session {
        meta,
        requirements: requirements.clone(),
        snapshots: HashMap::new(),
        current_snapshot_id: 1,
        layout,
        input_count: 0,
        round: 0,
        can_input: true,
        initialized: false,
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

/// This should be called after loading the session and calling /scripts/add, to ensure that all required scripts are
/// actually loaded to the lua vm.
async fn import_scripts(
    state: State<AppState>,
    req: Json<transfer::ImportScriptsRequest>,
) -> Response<&'static str> {
    let session_id = req.0;
    let session_ref = state.0.get_session(session_id)?;

    log::info!("Importing scripts for session {session_id}");

    // This clone prevents sharing requirements across await, causing future to be not Send
    let requirements = session_ref.lock().unwrap().requirements.clone();
    let mut join_set = tokio::task::JoinSet::new();
    for requirement in requirements.into_iter() {
        join_set.spawn(
            state
                .0
                .lua
                .load(format!("require(\"{requirement}\")"))
                .exec_async(),
        );
    }
    let results = join_set.join_all().await;

    let session = session_ref.lock().unwrap();
    for (result, requirement) in results.into_iter().zip(session.requirements.iter()) {
        match result {
            Ok(_) => {
                log::info!("Session {session_id}: Script {requirement} loaded successfully")
            }
            Err(err) => {
                log::error!("Session {session_id}: Failed to load script {requirement}: {err}");
                return Err(transfer::ServerError::ServerSide(err.to_string()).into());
            }
        }
    }

    Ok("All requirements imported without errors")
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

async fn get_session_status(
    state: State<AppState>,
    req: Json<transfer::GetSessionStatusRequest>,
) -> Response<Json<transfer::SessionStatus>> {
    let session_id = req.0;
    let session_ref = state.0.get_session(session_id)?;
    let session = session_ref.lock().unwrap();
    let status = transfer::SessionStatus {
        snapshot_count: session.snapshots.len(),
        input_count: session.input_count,
        round: session.round,
        can_input: session.can_input,
    };
    Ok(status.into())
}

async fn get_map(
    state: State<AppState>,
    req: Json<transfer::GetMapRequest>,
) -> Response<Json<transfer::Map>> {
    let session_id = req.0;
    let session_ref = state.0.get_session(session_id)?;
    let mut session = session_ref.lock().unwrap();

    session.initialize()?;
    let map = transfer::Map::from_map(&session.layout.map);

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
        if !session.can_input {
            return Err(transfer::ServerError::InputRefused.into());
        }
        // round clearing is done when can_input is set to true
        session.input_count += 1;
    } else {
        // If the map should take an input, stepping call is fused to return empty movements.
        if session.can_input {
            return Ok(transfer::SendStepResponse {
                movements: Vec::new(),
            }
            .into());
        }
    }
    let signal = Signal {
        direction,
        round: session.round,
    };
    let movements = session
        .layout
        .step(signal, &state.lua, &state.scripts.read().unwrap())
        .map_err(|err| session.convert_error(err))?;

    // The session state is altered and needs to reinitialize.
    session.initialized = false;
    if movements.is_empty() {
        session.round = 0;
        session.can_input = true;
    } else {
        session.round += 1;
        session.can_input = false;
    }

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
        can_input: session.can_input,
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
        can_input,
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
    session.can_input = can_input;
    // Since only the objects were copied, we need to refresh map state
    session.initialized = false;

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
