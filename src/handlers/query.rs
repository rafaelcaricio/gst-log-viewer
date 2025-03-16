use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use regex::Regex;

use crate::models::{ApiError, AppState, LogFilter, LogResponse, SerializableEntry};

// Handler for getting log entries with filtering and pagination
pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<LogFilter>,
) -> Result<Json<LogResponse>, ApiError> {
    log::info!("Fetching logs with filters: {:?}", filter);

    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();
    let entries = logs.get(&filter.session_id).ok_or_else(|| {
        let msg = format!("Session not found: {}", filter.session_id);
        log::error!("{}", msg);
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: msg,
        }
    })?;

    log::debug!("Found session with {} entries", entries.len());

    // Apply filters
    let start_time = Instant::now();
    let filtered_entries = entries
        .iter()
        .filter(|entry| {
            // Filter by level if specified
            if let Some(ref level) = filter.level {
                if format!("{:?}", entry.level) != *level {
                    return false;
                }
            }

            // Filter by category if specified
            if let Some(ref category) = filter.category {
                if entry.category != *category {
                    return false;
                }
            }

            // Filter by message using regex if specified
            if let Some(ref message_regex) = filter.message_regex {
                if let Ok(regex) = Regex::new(message_regex) {
                    if !regex.is_match(&entry.message) {
                        return false;
                    }
                } else {
                    // Log invalid regex but don't filter out entries
                    log::error!("Invalid message regex: {}", message_regex);
                }
            }

            // Filter by PID if specified
            if let Some(pid) = filter.pid {
                if entry.pid != pid {
                    return false;
                }
            }

            // Filter by thread if specified
            if let Some(ref thread) = filter.thread {
                if entry.thread != *thread {
                    return false;
                }
            }

            // Filter by object if specified
            if let Some(ref object) = filter.object {
                if let Some(ref entry_object) = entry.object {
                    if entry_object != object {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Filter by function using regex if specified
            if let Some(ref function_regex) = filter.function_regex {
                if let Ok(regex) = Regex::new(function_regex) {
                    if !regex.is_match(&entry.function) {
                        return false;
                    }
                } else {
                    // Log invalid regex but don't filter out entries
                    log::error!("Invalid function regex: {}", function_regex);
                }
            }

            true
        })
        .collect::<Vec<_>>();

    let filter_time = start_time.elapsed();
    log::debug!(
        "Filtered to {} entries in {:.2?}",
        filtered_entries.len(),
        filter_time
    );

    // Apply pagination
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(100).min(1000);
    let total = filtered_entries.len();
    let total_pages = (total + per_page - 1) / per_page;

    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);

    log::debug!(
        "Pagination: page {}/{}, showing entries {}-{} of {}",
        page,
        total_pages,
        start + 1,
        end,
        total
    );

    let paginated_entries = filtered_entries
        .into_iter()
        .skip(start)
        .take(end - start)
        .map(SerializableEntry::from)
        .collect();

    Ok(Json(LogResponse {
        entries: paginated_entries,
        total,
        page,
        total_pages,
    }))
}
