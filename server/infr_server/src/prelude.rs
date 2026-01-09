//! This module is imported within infr_server

pub(crate) use axum::{
    Json, Router,
    extract::State,
    http,
    routing::{get, post, put},
};

pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::{Arc, Mutex, RwLock};

pub(crate) use infr_layout::*;
pub(crate) use infr_solver::*;
pub(crate) use infr_transfer as transfer;

pub type Response<T> = axum::response::Result<T>;
pub type TextResponse = Response<&'static str>;

#[derive(Clone)]
pub struct AppState {
    pub scripts: Arc<RwLock<scripts::Scripts>>,
    pub sessions: Arc<RwLock<HashMap<transfer::SessionId, Mutex<crate::session::Session>>>>,
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
}
