//! This module is imported within infr_server

pub(crate) use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};

pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::{Arc, Mutex, RwLock};

pub(crate) use infr_layout::*;
pub(crate) use infr_solver::*;
pub(crate) use infr_transfer as transfer;

pub type Response<T> = axum::response::Result<T>;
pub type TextResponse = Response<&'static str>;
pub type SessionHashMap = HashMap<transfer::SessionId, Arc<Mutex<crate::session::Session>>>;

#[derive(Clone)]
pub struct AppState {
    pub scripts: Arc<RwLock<scripts::Scripts>>,
    pub sessions: Arc<RwLock<SessionHashMap>>,
    pub lua: mlua::Lua,
}

impl AppState {
    /// Initializes AppState with proper calls.
    pub fn new() -> mlua::Result<Self> {
        let scripts = scripts::Scripts::new();
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let lua = mlua::Lua::new();
        scripts.add_global_functions(&lua)?;

        Ok(Self {
            scripts: Arc::new(RwLock::new(scripts)),
            sessions,
            lua,
        })
    }

    /// Retrieves an Arc to session hash map, and returns error if the desired session does not exist.
    pub fn get_session<'s>(
        &'s self,
        session_id: transfer::SessionId,
    ) -> Result<Arc<Mutex<crate::session::Session>>, transfer::ServerError> {
        let read_guard = self.sessions.read().unwrap();
        if let Some(session) = read_guard.get(&session_id) {
            Ok(session.clone())
        } else {
            Err(transfer::ServerError::BadRequest(format!(
                "Unknown session id: {session_id}"
            )))
        }
    }
}
