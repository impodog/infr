use std::path::PathBuf;

use crate::prelude::*;

pub(crate) fn route_scripts() -> Router<AppState> {
    Router::new()
        .route("/add", post(add_script))
        .route("/reload", post(reload_script))
        .route("/run", post(run_lua))
}

/// Calls to add a script with a name handle, if not already.
async fn add_script(
    state: State<AppState>,
    req: Json<transfer::AddScriptRequest>,
) -> Result<(), Json<transfer::ServerError>> {
    let transfer::AddScriptRequest { name, path } = req.0;
    log::info!("Request to add script: {name} at {path:?}");
    if !path.exists() {
        return Err(transfer::ServerError::ServerSide(format!(
            "Script at path {path:?} not found"
        ))
        .into());
    }
    state.0.scripts.write().await.add(name, PathBuf::from(path));
    reload_script(state).await?;
    Ok(())
}

/// Forces to reload all scripts that were changed.
async fn reload_script(state: State<AppState>) -> Result<(), Json<transfer::ServerError>> {
    log::info!("Reloading all scripts...");
    state
        .0
        .scripts
        .write()
        .await
        .reload(&state.lua)
        .map_err(|err| transfer::ServerError::ServerSide(err.to_string()))?;
    Ok(())
}

/// Runs a lua command.
async fn run_lua(state: State<AppState>, chunk: Json<String>) -> Result<(), String> {
    state
        .lua
        .load(chunk.0)
        .exec()
        .map_err(|err| err.to_string())
}
