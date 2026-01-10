use crate::prelude::*;

use std::sync::atomic::{self, AtomicU32};

/// Stores a game session, with object layouts and other added information.
#[derive(Debug, Clone)]
pub struct Session {
    pub meta: transfer::LevelMetadata,
    pub layout: Layout,
    pub input_count: u32,
    pub round: u32,
}
/// This is used for countering z3 errors(z3 doesn't contain Rc). Layouts may only be used under Mutex.
unsafe impl Send for Session {}

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

pub(crate) fn route_session() -> Router<AppState> {
    Router::new()
        .route("/load", post(load_session))
        .route("/meta", get(get_metadata))
        .route("/step", post(step))
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
        layout,
        input_count: 0,
        round: 0,
    };
    let session_id = SESSION_ID.fetch_add(1, atomic::Ordering::SeqCst);
    state
        .0
        .sessions
        .write()
        .unwrap()
        .insert(session_id, Arc::new(Mutex::new(session)));
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

async fn step(
    state: State<AppState>,
    req: Json<transfer::SendStepRequest>,
) -> Response<Json<transfer::SendStepResponse>> {
    let transfer::SendStepRequest {
        session_id,
        direction,
    } = req.0;

    let session_ref = state.0.get_session(session_id)?;
    let mut session = session_ref.lock().unwrap();

    let direction = Option::<Direction>::from(direction);
    // This movement directly responds to player input, meaning that a new set of rounds begins.
    if direction.is_some() {
        session.round = 0;
        session.input_count += 1;
    };
    let signal = Signal {
        direction,
        round: session.round,
    };
    let movements = session
        .layout
        .step(signal, &state.lua, &state.scripts.read().unwrap())
        .map_err(|err| session.convert_error(err))?;

    Ok(transfer::SendStepResponse {
        movements: movements.into_iter().map(Into::into).collect::<Vec<_>>(),
    }
    .into())
}
