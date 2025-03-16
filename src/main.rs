mod handlers;
mod models;
mod parser;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::routing::{get, post};
use axum::Router;
use tempfile::tempdir;
use tower_http::{cors::CorsLayer, services::ServeDir};

use handlers::options::get_filter_options;
use handlers::query::get_logs;
use handlers::upload::upload_log;
use models::AppState;

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    // Initialize GStreamer (required by the parser)
    gstreamer::init().expect("Failed to initialize GStreamer");

    // Create a temporary directory for storing uploaded log files
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    log::info!("Using temporary directory: {}", temp_dir.path().display());

    // Create the shared application state
    let state = Arc::new(AppState {
        parsed_logs: RwLock::new(HashMap::new()),
        temp_dir,
    });

    // Build our application with routes
    let app = Router::new()
        .route("/api/upload", post(upload_log))
        .route("/api/logs", get(get_logs))
        .route("/api/filter-options", get(get_filter_options))
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Run our application with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
