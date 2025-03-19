mod handlers;
mod models;
mod parser;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use axum::routing::{get, post};
use axum::extract::DefaultBodyLimit;
use axum::Router;
use tempfile::TempDir;
use tower_http::{cors::CorsLayer, services::ServeDir};

use handlers::options::get_filter_options;
use handlers::query::get_logs;
use handlers::timeline::get_timeline;
use handlers::upload::upload_log;
use models::AppState;

fn get_storage_dir() -> Result<TempDir> {
    // Check if running in Cloudron environment
    if let Ok(data_dir) = env::var("CLOUDRON_APP_DATA_DIR") {
        let logs_dir = PathBuf::from(data_dir).join("logs");
        std::fs::create_dir_all(&logs_dir)?;
        log::info!(
            "Using Cloudron data directory for logs: {}",
            logs_dir.display()
        );
        // Create a TempDir in the persistent location
        let dir = TempDir::new_in(&logs_dir)?;
        Ok(dir)
    } else {
        // Fallback to temporary directory for local development
        let dir = tempfile::tempdir()?;
        log::info!("Using temporary directory: {}", dir.path().display());
        Ok(dir)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Initialize GStreamer (required by the parser)
    gstreamer::init().expect("Failed to initialize GStreamer");

    // Get storage directory
    let temp_dir = get_storage_dir().expect("Failed to create storage directory");

    // Create the shared application state
    let state = Arc::new(AppState {
        parsed_logs: RwLock::new(HashMap::new()),
        temp_dir,
    });

    // Build our application with routes
    let app = Router::new()
        .route("/api/upload", post(upload_log))
        .route("/api/logs", get(get_logs))
        .route("/api/timeline", get(get_timeline))
        .route("/api/filter-options", get(get_filter_options))
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024)) // Set max body limit to 500MB
        .with_state(state);

    // Run our application with hyper
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
