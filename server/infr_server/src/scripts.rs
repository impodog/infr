use std::path::PathBuf;

use crate::prelude::*;

pub(crate) fn route_scripts() -> Router<AppState> {
    Router::new()
        .route("/add", post(add_script))
        .route("/reload", post(reload_script))
        .route("/run", post(run_lua))
}

/// Calls to add a script with a name handle, if not already.
async fn add_script(state: State<AppState>, req: Json<transfer::AddScriptRequest>) -> TextResponse {
    let transfer::AddScriptRequest { name, path } = req.0;
    log::info!("Request to manually add script: {name} at {path:?}");
    if !path.exists() {
        return Err(transfer::ServerError::BadRequest(format!("File not found: {path:?}")).into());
    }
    state
        .0
        .scripts
        .write()
        .unwrap()
        .add(name, PathBuf::from(path));
    Ok("Script added successfully")
}

/// Forces to reload all scripts that were changed.
pub(crate) async fn reload_script(state: State<AppState>) -> TextResponse {
    log::info!("Reloading all scripts...");
    state
        .0
        .scripts
        .write()
        .unwrap()
        .reload(&state.lua)
        .map_err(|err| transfer::ServerError::ServerSide(err.to_string()))?;
    Ok("Scripts reloaded without error")
}

/// Runs a lua command.
async fn run_lua(state: State<AppState>, chunk: Json<String>) -> Result<(), String> {
    state
        .lua
        .load(chunk.0)
        .exec()
        .map_err(|err| err.to_string())
}
