use crate::prelude::*;

use std::sync::atomic::{self, AtomicU32};

/// Stores a game session, with object layouts and other added information.
#[derive(Debug, Clone)]
pub struct Session {
    pub title: String,
    pub layout: Layout,
    pub steps: usize,
}
/// This is used for countering z3 errors(z3 doesn't contain Rc). Layouts may only be used under Mutex.
unsafe impl Send for Session {}

pub(crate) fn route_session() -> Router<AppState> {
    Router::new().route("/load", post(load_session))
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
        title,
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
        title,
        layout,
        steps: 0,
    };
    let session_id = SESSION_ID.fetch_add(1, atomic::Ordering::SeqCst);
    state
        .0
        .sessions
        .write()
        .unwrap()
        .insert(session_id, Mutex::new(session));
    Ok(transfer::LoadSessionResponse {
        id: session_id,
        requirements,
    }
    .into())
}
