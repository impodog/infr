//! Implements the infr server, handling the interfacing layer between the client and backend solvers.

mod config;

mod scripts;
mod session;

mod prelude;
use prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/test", get(test_server))
        .nest("/scripts", scripts::route_scripts())
        .with_state(AppState {
            scripts: Arc::new(RwLock::new(infr_layout::scripts::Scripts::new())),
            lua: mlua::Lua::new_with(mlua::StdLib::ALL_SAFE, Default::default())
                .expect("Unable to create lua instance"),
            sessions: Default::default(),
        });

    let listener = tokio::net::TcpListener::bind(config::STARTUP_CONFIG.address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test_server() -> &'static str {
    "The server is working!"
}
