//! This module is imported within infr_server

pub(crate) use axum::{
    Json, Router, ServiceExt,
    extract::State,
    http,
    routing::{get, post, put},
};
pub(crate) use tower::Service;

pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::Arc;
pub(crate) use tokio::sync::{Mutex, RwLock};

pub(crate) use infr_layout::*;
pub(crate) use infr_solver::*;
pub(crate) use infr_transfer as transfer;

#[derive(Clone)]
pub struct AppState {
    pub scripts: Arc<RwLock<scripts::Scripts>>,
    pub sessions: Arc<RwLock<HashMap<transfer::SessionId, Mutex<crate::session::Session>>>>,
    pub lua: mlua::Lua,
}
