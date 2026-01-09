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
        .nest("/session", session::route_session())
        .with_state(AppState::new().expect("App initialization failed"));

    let listener = tokio::net::TcpListener::bind(config::STARTUP_CONFIG.address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test_server() -> &'static str {
    "The server is working!"
}
