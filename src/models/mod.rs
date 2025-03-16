use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::RwLock;
use tempfile::TempDir;

use crate::parser::Entry;

// Temporary storage for uploaded log files and parsed entries
pub struct AppState {
    // Map of session ID to parsed log entries
    pub parsed_logs: RwLock<HashMap<String, Vec<Entry>>>,
    // Directory for temporary log file storage
    pub temp_dir: TempDir,
}

// Custom error type for API errors with better logging
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiError({:?}): {}", self.status, self.message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        // Log the error with full context
        log::error!("{}", self);

        // Convert to HTTP response
        (
            self.status,
            Json(HashMap::from([("error".to_string(), self.message)])),
        )
            .into_response()
    }
}

// Filter parameters for log query
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LogFilter {
    pub session_id: String,
    pub level: Option<String>,
    // Instead of trying to deserialize directly, we'll handle this field manually
    #[serde(skip)]
    pub categories: Vec<String>,
    pub message_regex: Option<String>,
    pub pid: Option<u32>,
    pub thread: Option<String>,
    pub object: Option<String>,
    pub function_regex: Option<String>,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    // Time range filtering
    pub min_timestamp: Option<u64>,
    pub max_timestamp: Option<u64>,
    // Explicit time unit flag
    #[serde(default)]
    pub use_microseconds: bool,
}

// Helper functions for default values
fn default_page() -> usize {
    1
}
fn default_per_page() -> usize {
    100
}

// Response with log entries
#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub entries: Vec<SerializableEntry>,
    pub total: usize,
    pub page: usize,
    pub total_pages: usize,
}

// Response with available filter options
#[derive(Debug, Serialize)]
pub struct FilterOptionsResponse {
    pub categories: Vec<String>,
    pub levels: Vec<String>,
    pub pids: Vec<u32>,
    pub threads: Vec<String>,
    pub objects: Vec<String>,
}

// Make Entry serializable for JSON responses
#[derive(Debug, Serialize)]
pub struct SerializableEntry {
    pub ts: String,
    pub pid: u32,
    pub thread: String,
    pub level: String,
    pub category: String,
    pub file: String,
    pub line: u32,
    pub function: String,
    pub message: String,
    pub object: Option<String>,
}

impl From<&Entry> for SerializableEntry {
    fn from(entry: &Entry) -> Self {
        SerializableEntry {
            ts: format!("{}", entry.ts),
            pid: entry.pid,
            thread: entry.thread.clone(),
            level: format!("{:?}", entry.level),
            category: entry.category.clone(),
            file: entry.file.clone(),
            line: entry.line,
            function: entry.function.clone(),
            message: entry.message.clone(),
            object: entry.object.clone(),
        }
    }
}
