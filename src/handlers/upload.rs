use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::Json;
use uuid::Uuid;

use crate::models::{ApiError, AppState};
use crate::parser;
use crate::parser::Entry;

// Handler for log file uploads
pub async fn upload_log(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<HashMap<String, String>>, ApiError> {
    // Generate a unique session ID for this upload
    let session_id = Uuid::new_v4().to_string();
    let temp_path = state.temp_dir.path().join(&session_id);

    log::info!("Starting upload for session: {}", session_id);

    // Extract and save the uploaded file
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        let msg = format!("Failed to read multipart form: {}", e);
        log::error!("{}", msg);
        ApiError {
            status: StatusCode::BAD_REQUEST,
            message: msg,
        }
    })? {
        let field_name = field.name().unwrap_or("unnamed").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        log::debug!(
            "Processing field '{}' with type '{}'",
            field_name, content_type
        );

        // Get file data
        let data = field.bytes().await.map_err(|e| {
            let msg = format!("Failed to read field data: {}", e);
            log::error!("{}", msg);
            ApiError {
                status: StatusCode::BAD_REQUEST,
                message: msg,
            }
        })?;

        log::debug!("Received file data of size: {} bytes", data.len());

        // Create and write to temporary file
        let mut file = File::create(&temp_path).map_err(|e| {
            let msg = format!("Failed to create temporary file: {}", e);
            log::error!("{}", msg);
            ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: msg,
            }
        })?;

        file.write_all(&data).map_err(|e| {
            let msg = format!("Failed to write data to temporary file: {}", e);
            log::error!("{}", msg);
            ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: msg,
            }
        })?;

        log::debug!("File written to temporary path: {}", temp_path.display());

        // Parse log file in a blocking task to avoid blocking the async runtime
        let session_id_clone = session_id.clone();
        let temp_path_clone = temp_path.clone();
        let state_clone = state.clone();

        tokio::task::spawn_blocking(move || {
            let result = parse_log_file(temp_path_clone, session_id_clone, state_clone);
            if let Err(e) = &result {
                log::error!("Error parsing log file: {}", e);
            }
            result
        });

        // For simplicity, we only process the first field
        break;
    }

    Ok(Json(HashMap::from([(
        "session_id".to_string(),
        session_id,
    )])))
}

// Parse the log file and store the entries in the app state
pub fn parse_log_file(
    path: impl AsRef<Path>,
    session_id: String,
    state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    log::info!("Parsing log file for session {}", session_id);
    let start_time = Instant::now();

    // Open the file and parse it
    let file = File::open(&path)?;
    let file_size = fs::metadata(&path)?.len();
    log::debug!("Opened file with size: {} bytes", file_size);

    let entries: Vec<Entry> = parser::parse(file).collect();
    let elapsed = start_time.elapsed();

    log::info!(
        "Parsed {} entries for session {} in {:.2?}",
        entries.len(),
        session_id,
        elapsed
    );

    if entries.is_empty() {
        log::warn!("No entries were parsed from the log file. This might indicate an incorrect format.");
    } else {
        // Sample the first few entries to help with debugging
        log::debug!("Sample entries (up to 3):");
        for (i, entry) in entries.iter().take(3).enumerate() {
            log::debug!(
                "  Entry {}: {} | {}:{} | {} | {:?}",
                i + 1,
                entry.ts,
                entry.file,
                entry.line,
                entry.category,
                entry.level
            );
        }
    }

    // Store the parsed entries
    {
        let mut logs = state.parsed_logs.write().unwrap();
        logs.insert(session_id.clone(), entries);
        log::debug!("Stored parsed entries in state for session: {}", session_id);
        log::debug!("Current sessions in state: {}", logs.len());
    }

    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&path) {
        log::error!("Error removing temporary file: {}", e);
    } else {
        log::debug!("Removed temporary file: {}", path.as_ref().display());
    }

    Ok(())
}
