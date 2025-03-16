use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::extract::{Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::Json;
use regex::Regex;

use crate::models::{ApiError, AppState, LogFilter, LogResponse, SerializableEntry};

// Handler for getting log entries with filtering and pagination
pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    raw_query: RawQuery,
    // Use an extractor to capture deserialization errors
    query_result: Result<Query<LogFilter>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<LogResponse>, ApiError> {
    // Log the raw query string first to see exactly what's being received
    log::info!("Raw query string: {:?}", raw_query.0);

    // Explicitly handle query parameter errors
    let filter = match query_result {
        Ok(Query(mut filter)) => {
            // We've successfully deserialized the basic parameters
            // Now manually extract the categories from the raw query string
            if let Some(query_str) = raw_query.0.as_ref() {
                // Parse the query string to get all categories
                let pairs = url::form_urlencoded::parse(query_str.as_bytes());

                // Extract all 'categories' parameters
                for (key, value) in pairs {
                    if key == "categories" {
                        log::debug!("Found category in query string: {}", value);
                        filter.categories.push(value.to_string());
                    }
                }

                log::info!("Manually extracted categories: {:?}", filter.categories);
            }
            filter
        }
        Err(err) => {
            log::error!("Failed to deserialize query parameters: {:?}", err);
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: format!("Invalid query parameters: {}", err),
            });
        }
    };

    log::info!("Deserialized filters: {:?}", filter);

    // Log categories specifically for debugging
    if !filter.categories.is_empty() {
        log::info!("Categories filter: {:?}", filter.categories);
    } else {
        log::info!("No categories filter applied");
    }

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

            // Filter by categories if specified
            if !filter.categories.is_empty() {
                log::debug!(
                    "Filtering by categories: {:?}, entry category: {}",
                    filter.categories,
                    entry.category
                );
                // For debugging purposes
                let entry_bytes = entry.category.as_bytes();
                log::debug!("Entry category as bytes: {:?}", entry_bytes);

                let mut found = false;
                for cat in &filter.categories {
                    let cat_bytes = cat.as_bytes();
                    log::debug!("Filter category as bytes: {:?}", cat_bytes);

                    // Do various equality checks to help debug
                    let string_eq = cat == &entry.category;
                    let bytes_eq = cat_bytes == entry_bytes;
                    let trim_eq = cat.trim() == entry.category.trim();

                    log::debug!(
                        "'{}' == '{}': string_eq={}, bytes_eq={}, trim_eq={}",
                        cat,
                        entry.category,
                        string_eq,
                        bytes_eq,
                        trim_eq
                    );

                    if string_eq || bytes_eq || trim_eq {
                        found = true;
                        break;
                    }
                }

                if !found {
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
    let page = filter.page.max(1);
    let per_page = filter.per_page.min(1000);
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
