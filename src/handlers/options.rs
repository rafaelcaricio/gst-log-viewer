use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;

use crate::models::{ApiError, AppState, FilterOptionsResponse};

// Handler for getting available filter options
pub async fn get_filter_options(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<HashMap<String, String>>,
) -> Result<Json<FilterOptionsResponse>, ApiError> {
    let session_id = filter.get("session_id").ok_or_else(|| {
        let msg = "Missing session_id parameter".to_string();
        log::error!("{}", msg);
        ApiError {
            status: StatusCode::BAD_REQUEST,
            message: msg,
        }
    })?;

    log::info!("Fetching filter options for session: {}", session_id);

    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();

    // Check if we have logs for this session
    let session_exists = logs.contains_key(session_id);
    log::debug!("Session exists in state: {}", session_exists);

    // List all sessions for debugging
    log::debug!("Available sessions: {:?}", logs.keys().collect::<Vec<_>>());

    let entries = logs.get(session_id).ok_or_else(|| {
        let msg = format!("Session not found: {}. This may occur if the log file is still being processed or if parsing failed.", session_id);
        log::error!("{}", msg);
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: msg,
        }
    })?;

    log::debug!("Found session with {} entries", entries.len());

    // Check if we have entries
    if entries.is_empty() {
        let msg = format!("No log entries found for session: {}. The log file may be empty or in an incorrect format.", session_id);
        log::error!("{}", msg);
        return Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: msg,
        });
    }

    // Extract unique values for each filter field
    let start_time = Instant::now();
    let mut categories = HashSet::new();
    let mut levels = HashSet::new();
    let mut pids = HashSet::new();
    let mut threads = HashSet::new();
    let mut objects = HashSet::new();

    for entry in entries {
        categories.insert(entry.category.clone());
        levels.insert(format!("{:?}", entry.level));
        pids.insert(entry.pid);
        threads.insert(entry.thread.clone());
        if let Some(ref object) = entry.object {
            objects.insert(object.clone());
        }
    }

    let elapsed = start_time.elapsed();
    log::debug!("Extracted filter options in {:.2?}: {} categories, {} levels, {} PIDs, {} threads, {} objects",
        elapsed, categories.len(), levels.len(), pids.len(), threads.len(), objects.len());

    let response = FilterOptionsResponse {
        categories: categories.into_iter().collect(),
        levels: levels.into_iter().collect(),
        pids: pids.into_iter().collect(),
        threads: threads.into_iter().collect(),
        objects: objects.into_iter().collect(),
    };

    Ok(Json(response))
}
