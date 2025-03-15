use anyhow::Result;
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use gst_log_parser::Entry;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Arc, RwLock},
    time::Instant,
};
use tempfile::tempdir;
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

// Temporary storage for uploaded log files and parsed entries
struct AppState {
    // Map of session ID to parsed log entries
    parsed_logs: RwLock<HashMap<String, Vec<Entry>>>,
    // Directory for temporary log file storage
    temp_dir: tempfile::TempDir,
}

// Custom error type for API errors with better logging
#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiError({:?}): {}", self.status, self.message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        // Log the error
        eprintln!("{}", self);

        // Convert to HTTP response
        (
            self.status,
            Json(HashMap::from([("error".to_string(), self.message)])),
        )
            .into_response()
    }
}

// Filter parameters for log query
#[derive(Debug, Deserialize)]
struct LogFilter {
    session_id: String,
    level: Option<String>,
    category: Option<String>,
    message_regex: Option<String>,
    pid: Option<u32>,
    thread: Option<String>,
    object: Option<String>,
    function_regex: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
}

// Response with log entries
#[derive(Debug, Serialize)]
struct LogResponse {
    entries: Vec<SerializableEntry>,
    total: usize,
    page: usize,
    total_pages: usize,
}

// Response with available filter options
#[derive(Debug, Serialize)]
struct FilterOptionsResponse {
    categories: Vec<String>,
    levels: Vec<String>,
    pids: Vec<u32>,
    threads: Vec<String>,
    objects: Vec<String>,
}

// Make Entry serializable for JSON responses
#[derive(Debug, Serialize)]
struct SerializableEntry {
    ts: String,
    pid: u32,
    thread: String,
    level: String,
    category: String,
    file: String,
    line: u32,
    function: String,
    message: String,
    object: Option<String>,
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

#[tokio::main]
async fn main() {
    // Initialize GStreamer (required by the parser)
    gstreamer::init().expect("Failed to initialize GStreamer");

    // Create a temporary directory for storing uploaded log files
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    println!("Using temporary directory: {}", temp_dir.path().display());

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
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// Handler for log file uploads
async fn upload_log(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<HashMap<String, String>>, ApiError> {
    // Generate a unique session ID for this upload
    let session_id = Uuid::new_v4().to_string();
    let temp_path = state.temp_dir.path().join(&session_id);

    println!("Starting upload for session: {}", session_id);

    // Extract and save the uploaded file
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        let msg = format!("Failed to read multipart form: {}", e);
        eprintln!("{}", msg);
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

        println!(
            "Processing field '{}' with type '{}'",
            field_name, content_type
        );

        // Get file data
        let data = field.bytes().await.map_err(|e| {
            let msg = format!("Failed to read field data: {}", e);
            eprintln!("{}", msg);
            ApiError {
                status: StatusCode::BAD_REQUEST,
                message: msg,
            }
        })?;

        println!("Received file data of size: {} bytes", data.len());

        // Create and write to temporary file
        let mut file = File::create(&temp_path).map_err(|e| {
            let msg = format!("Failed to create temporary file: {}", e);
            eprintln!("{}", msg);
            ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: msg,
            }
        })?;

        file.write_all(&data).map_err(|e| {
            let msg = format!("Failed to write data to temporary file: {}", e);
            eprintln!("{}", msg);
            ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: msg,
            }
        })?;

        println!("File written to temporary path: {}", temp_path.display());

        // Parse log file in a blocking task to avoid blocking the async runtime
        let session_id_clone = session_id.clone();
        let temp_path_clone = temp_path.clone();
        let state_clone = state.clone();

        tokio::task::spawn_blocking(move || {
            let result = parse_log_file(temp_path_clone, session_id_clone, state_clone);
            if let Err(e) = &result {
                eprintln!("Error parsing log file: {}", e);
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
fn parse_log_file(
    path: impl AsRef<Path>,
    session_id: String,
    state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    println!("Parsing log file for session {}", session_id);
    let start_time = Instant::now();

    // Open the file and parse it
    let file = File::open(&path)?;
    let file_size = fs::metadata(&path)?.len();
    println!("Opened file with size: {} bytes", file_size);

    let entries: Vec<Entry> = gst_log_parser::parse(file).collect();
    let elapsed = start_time.elapsed();

    println!(
        "Parsed {} entries for session {} in {:.2?}",
        entries.len(),
        session_id,
        elapsed
    );

    if entries.is_empty() {
        println!("WARNING: No entries were parsed from the log file. This might indicate an incorrect format.");
    } else {
        // Sample the first few entries to help with debugging
        println!("Sample entries (up to 3):");
        for (i, entry) in entries.iter().take(3).enumerate() {
            println!(
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
        println!("Stored parsed entries in state for session: {}", session_id);
        println!("Current sessions in state: {}", logs.len());
    }

    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&path) {
        eprintln!("Error removing temporary file: {}", e);
    } else {
        println!("Removed temporary file: {}", path.as_ref().display());
    }

    Ok(())
}

// Handler for getting log entries with filtering and pagination
async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<LogFilter>,
) -> Result<Json<LogResponse>, ApiError> {
    println!("Fetching logs with filters: {:?}", filter);

    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();
    let entries = logs.get(&filter.session_id).ok_or_else(|| {
        let msg = format!("Session not found: {}", filter.session_id);
        eprintln!("{}", msg);
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: msg,
        }
    })?;

    println!("Found session with {} entries", entries.len());

    // Apply filters
    let start_time = Instant::now();
    let filtered_entries: Vec<&Entry> = entries
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
                if let Ok(regex) = regex::Regex::new(message_regex) {
                    if !regex.is_match(&entry.message) {
                        return false;
                    }
                } else {
                    // Log invalid regex but don't filter out entries
                    eprintln!("Invalid message regex: {}", message_regex);
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
                if let Ok(regex) = regex::Regex::new(function_regex) {
                    if !regex.is_match(&entry.function) {
                        return false;
                    }
                } else {
                    // Log invalid regex but don't filter out entries
                    eprintln!("Invalid function regex: {}", function_regex);
                }
            }

            true
        })
        .collect();

    let filter_time = start_time.elapsed();
    println!(
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

    println!(
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

// Handler for getting available filter options
async fn get_filter_options(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<HashMap<String, String>>,
) -> Result<Json<FilterOptionsResponse>, ApiError> {
    let session_id = filter.get("session_id").ok_or_else(|| {
        let msg = "Missing session_id parameter".to_string();
        eprintln!("{}", msg);
        ApiError {
            status: StatusCode::BAD_REQUEST,
            message: msg,
        }
    })?;

    println!("Fetching filter options for session: {}", session_id);

    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();

    // Check if we have logs for this session
    let session_exists = logs.contains_key(session_id);
    println!("Session exists in state: {}", session_exists);

    // List all sessions for debugging
    println!("Available sessions: {:?}", logs.keys().collect::<Vec<_>>());

    let entries = logs.get(session_id).ok_or_else(|| {
        let msg = format!("Session not found: {}. This may occur if the log file is still being processed or if parsing failed.", session_id);
        eprintln!("{}", msg);
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: msg,
        }
    })?;

    println!("Found session with {} entries", entries.len());

    // Check if we have entries
    if entries.is_empty() {
        let msg = format!("No log entries found for session: {}. The log file may be empty or in an incorrect format.", session_id);
        eprintln!("{}", msg);
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
    println!("Extracted filter options in {:.2?}: {} categories, {} levels, {} PIDs, {} threads, {} objects",
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
