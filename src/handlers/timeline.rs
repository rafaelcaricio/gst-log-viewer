use axum::extract::{Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::Json;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::{ApiError, AppState, LogFilter};

// Helper function to convert ClockTime to milliseconds
fn to_milliseconds(clock_time: &gstreamer::ClockTime) -> u64 {
    // ClockTime is in nanoseconds, convert to milliseconds
    clock_time.nseconds() / 1_000_000
}

// Helper function to convert ClockTime to microseconds
fn to_microseconds(clock_time: &gstreamer::ClockTime) -> u64 {
    // ClockTime is in nanoseconds, convert to microseconds
    clock_time.nseconds() / 1_000
}

// Struct for timeline filter parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimelineFilter {
    #[serde(flatten)]
    pub log_filter: LogFilter,
    #[serde(default = "default_interval")]
    pub interval: String,
}

fn default_interval() -> String {
    "1s".to_string()
}

// Response for timeline data
#[derive(Debug, Serialize)]
pub struct TimelineBucket {
    pub timestamp: u64, // Timestamp in milliseconds
    pub count: usize,   // Number of log entries
}

#[derive(Debug, Serialize)]
pub struct TimelineResponse {
    pub buckets: Vec<TimelineBucket>,
    pub min_timestamp: u64,
    pub max_timestamp: u64,
}

// Parse interval string into microseconds
fn parse_interval(interval: &str) -> Result<u64, ApiError> {
    let re = Regex::new(r"^(\d+)(us|ms|s|m)$").unwrap();

    if let Some(captures) = re.captures(interval) {
        let value: u64 = captures
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| ApiError {
                status: StatusCode::BAD_REQUEST,
                message: format!("Invalid interval value: {}", interval),
            })?;

        let unit = captures.get(2).unwrap().as_str();

        match unit {
            "us" => Ok(value),             // Microseconds
            "ms" => Ok(value * 1_000),     // Milliseconds to microseconds
            "s" => Ok(value * 1_000_000),  // Seconds to microseconds
            "m" => Ok(value * 60_000_000), // Minutes to microseconds
            _ => Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: format!("Invalid interval unit: {}", unit),
            }),
        }
    } else {
        Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: format!("Invalid interval format: {}", interval),
        })
    }
}

// Handler for getting timeline data
pub async fn get_timeline(
    State(state): State<Arc<AppState>>,
    raw_query: RawQuery,
    query_result: Result<Query<TimelineFilter>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<TimelineResponse>, ApiError> {
    // Log the raw query string
    log::info!("Timeline raw query string: {:?}", raw_query.0);

    // Explicitly handle query parameter errors
    let filter = match query_result {
        Ok(Query(mut filter)) => {
            // Manually extract the categories from the raw query string
            if let Some(query_str) = raw_query.0.as_ref() {
                // Parse the query string to get all categories
                let pairs = url::form_urlencoded::parse(query_str.as_bytes());

                // Extract all 'categories' parameters
                for (key, value) in pairs {
                    if key == "categories" {
                        log::debug!("Found category in timeline query string: {}", value);
                        filter.log_filter.categories.push(value.to_string());
                    }
                }
            }
            filter
        }
        Err(err) => {
            log::error!("Failed to deserialize timeline query parameters: {:?}", err);
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: format!("Invalid timeline query parameters: {}", err),
            });
        }
    };

    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();
    let entries = logs.get(&filter.log_filter.session_id).ok_or_else(|| {
        let msg = format!("Session not found: {}", filter.log_filter.session_id);
        log::error!("{}", msg);
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: msg,
        }
    })?;

    // Apply filters
    let filtered_entries = entries
        .iter()
        .filter(|entry| {
            // Apply the same filtering logic as in query.rs

            // Filter by level if specified
            if let Some(ref level) = filter.log_filter.level {
                if format!("{:?}", entry.level) != *level {
                    return false;
                }
            }

            // Filter by categories if specified
            if !filter.log_filter.categories.is_empty() {
                let mut found = false;
                for cat in &filter.log_filter.categories {
                    if cat == &entry.category || cat.trim() == entry.category.trim() {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }

            // Filter by message using regex if specified
            if let Some(ref message_regex) = filter.log_filter.message_regex {
                if let Ok(regex) = Regex::new(message_regex) {
                    if !regex.is_match(&entry.message) {
                        return false;
                    }
                }
            }

            // Filter by PID if specified
            if let Some(pid) = filter.log_filter.pid {
                if entry.pid != pid {
                    return false;
                }
            }

            // Filter by thread if specified
            if let Some(ref thread) = filter.log_filter.thread {
                if entry.thread != *thread {
                    return false;
                }
            }

            // Filter by object if specified
            if let Some(ref object) = filter.log_filter.object {
                if let Some(ref entry_object) = entry.object {
                    if entry_object != object {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Filter by function using regex if specified
            if let Some(ref function_regex) = filter.log_filter.function_regex {
                if let Ok(regex) = Regex::new(function_regex) {
                    if !regex.is_match(&entry.function) {
                        return false;
                    }
                }
            }

            true
        })
        .collect::<Vec<_>>();

    // Parse the requested time interval (now in microseconds)
    let interval_us = parse_interval(&filter.interval)?;

    // Check if we need microsecond precision
    let use_microseconds = filter.interval.ends_with("us");

    // Find min and max timestamps - using microseconds or milliseconds as appropriate
    let (min_timestamp, max_timestamp) = if use_microseconds {
        let min = filtered_entries
            .iter()
            .map(|e| to_microseconds(&e.ts))
            .min()
            .unwrap_or(0);

        let max = filtered_entries
            .iter()
            .map(|e| to_microseconds(&e.ts))
            .max()
            .unwrap_or(0);

        (min, max)
    } else {
        let min = filtered_entries
            .iter()
            .map(|e| to_milliseconds(&e.ts))
            .min()
            .unwrap_or(0);

        let max = filtered_entries
            .iter()
            .map(|e| to_milliseconds(&e.ts))
            .max()
            .unwrap_or(0);

        (min, max)
    };

    // Group entries by time bucket
    let mut buckets: HashMap<u64, usize> = HashMap::new();

    for entry in &filtered_entries {
        let bucket_time = if use_microseconds {
            // Use microsecond precision
            let ts_us = to_microseconds(&entry.ts);
            ((ts_us - min_timestamp) / interval_us) * interval_us + min_timestamp
        } else {
            // Use millisecond precision - convert interval_us to milliseconds for calculation
            let ts_ms = to_milliseconds(&entry.ts);
            ((ts_ms - min_timestamp) / (interval_us / 1000)) * (interval_us / 1000) + min_timestamp
        };

        *buckets.entry(bucket_time).or_insert(0) += 1;
    }

    // Convert hashmap to sorted vector of buckets
    let mut timeline_buckets: Vec<TimelineBucket> = buckets
        .into_iter()
        .map(|(timestamp, count)| TimelineBucket { timestamp, count })
        .collect();

    // Sort by timestamp
    timeline_buckets.sort_by_key(|b| b.timestamp);

    Ok(Json(TimelineResponse {
        buckets: timeline_buckets,
        min_timestamp,
        max_timestamp,
    }))
}
