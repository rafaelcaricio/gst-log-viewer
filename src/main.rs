use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use gst_log_parser::Entry;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Arc, RwLock},
};
use tempfile::tempdir;
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;
use anyhow::Result;

// Temporary storage for uploaded log files and parsed entries
struct AppState {
    // Map of session ID to parsed log entries
    parsed_logs: RwLock<HashMap<String, Vec<Entry>>>,
    // Directory for temporary log file storage
    temp_dir: tempfile::TempDir,
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
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    // Generate a unique session ID for this upload
    let session_id = Uuid::new_v4().to_string();
    let temp_path = state.temp_dir.path().join(&session_id);
    
    // Extract and save the uploaded file
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        // Get file data (we don't need the filename)
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Create and write to temporary file
        let mut file = File::create(&temp_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Parse log file in a blocking task to avoid blocking the async runtime
        let session_id_clone = session_id.clone();
        let temp_path_clone = temp_path.clone();
        let state_clone = state.clone();
        
        tokio::task::spawn_blocking(move || -> Result<(), anyhow::Error> {
            parse_log_file(temp_path_clone, session_id_clone, state_clone)
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
fn parse_log_file(path: impl AsRef<Path>, session_id: String, state: Arc<AppState>) -> Result<(), anyhow::Error> {
    println!("Parsing log file for session {}", session_id);
    
    // Open the file and parse it
    let file = File::open(&path)?;
    let entries: Vec<Entry> = gst_log_parser::parse(file).collect();
    
    println!("Parsed {} entries for session {}", entries.len(), session_id);
    
    // Store the parsed entries
    state.parsed_logs.write().unwrap().insert(session_id, entries);
    
    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&path) {
        eprintln!("Error removing temporary file: {}", e);
    }
    
    Ok(())
}

// Handler for getting log entries with filtering and pagination
async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<LogFilter>,
) -> Result<Json<LogResponse>, StatusCode> {
    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();
    let entries = logs.get(&filter.session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    // Apply filters
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
                }
            }
            
            true
        })
        .collect();
    
    // Apply pagination
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(100).min(1000);
    let total = filtered_entries.len();
    let total_pages = (total + per_page - 1) / per_page;
    
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);
    
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
) -> Result<Json<FilterOptionsResponse>, StatusCode> {
    let session_id = filter.get("session_id").ok_or(StatusCode::BAD_REQUEST)?;
    
    // Get the parsed logs for the session
    let logs = state.parsed_logs.read().unwrap();
    let entries = logs.get(session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    // Extract unique values for each filter field
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
    
    Ok(Json(FilterOptionsResponse {
        categories: categories.into_iter().collect(),
        levels: levels.into_iter().collect(),
        pids: pids.into_iter().collect(),
        threads: threads.into_iter().collect(),
        objects: objects.into_iter().collect(),
    }))
}