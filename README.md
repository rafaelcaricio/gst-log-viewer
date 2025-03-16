# GStreamer Log Viewer

A web-based viewer for GStreamer log files that allows filtering and analysis of GStreamer application pipeline logs.

![GStreamer Log Viewer Screenshot - with timeline view and filtering options](screenshot-gst-log-viewer.png)

## Features

- Upload and parse GStreamer log files
- Interactive timeline view that shows log entry distribution over time:
  - Group logs by various time intervals (microseconds to minutes)
  - Select specific time ranges to filter log entries
  - Visualize busy periods or potential issues at a glance
- Comprehensive filtering by:
  - Log level
  - Category
  - Message content (regex)
  - PID
  - Thread
  - Object
  - Function name (regex)
  - Time range selection
- Pagination for efficient navigation through large log files
- Responsive UI with a modern design

## Prerequisites

- Rust (2021 edition)
- Node.js and npm/yarn

## Project Structure

- `src/`: Rust backend code
- `frontend/`: React frontend application

## Setup & Running

### Backend

1. Build the backend:

```bash
cargo build
```

### Frontend

1. Install dependencies:

```bash
cd frontend
npm install
# or
yarn install
```

2. Build the frontend:

```bash
npm run build
# or
yarn build
```

### Running the Application

1. Start the server:

```bash
cargo run
```

2. Open your browser and navigate to http://localhost:3000

## Deployment

### Cloudron Deployment

This application can be deployed to [Cloudron](https://cloudron.io/), a self-hosted platform for running web applications.

1. Build the Docker image:

```bash
./build-cloudron.sh
```

2. To build and push to a Docker registry:

```bash
./build-cloudron.sh yourusername/gst-log-viewer:latest
```

3. Install on Cloudron:
   - Go to your Cloudron dashboard
   - Click "App Store"
   - Click "Install from Manifest"
   - Either provide the URL to your Git repository or upload the CloudronManifest.json file

#### Cloudron Files

- `CloudronManifest.json`: Defines the application for Cloudron
- `Dockerfile`: Multi-stage build file for creating the Docker image
- `start.sh`: Script to start the application in Cloudron environment
- `supervisord.conf`: Configuration for managing multiple processes
- `nginx.conf`: Configuration for the Nginx web server

#### Architecture

The Cloudron deployment uses:
- **Nginx**: Serves the static frontend files and proxies API requests to the backend
- **Supervisord**: Manages both the Nginx and backend processes
- **Rust Backend**: Runs on port 3000 (internal)
- **Web Interface**: Accessible via port 8000 as specified in CloudronManifest.json

## Usage

1. Upload a GStreamer log file using the upload interface
2. Explore the timeline chart to see log entry distribution over time:
   - Change the time interval grouping (from microseconds to minutes) using the dropdown
   - Click or drag on the timeline to select a specific time range
   - Use the brush below the timeline to zoom in on specific regions
   - Clear time range selection using the "Clear" button
3. Use the filters panel to narrow down the log entries by level, category, etc.
4. View and paginate through the filtered log entries
5. Adjust the entries per page as needed

## Technical Details

- Backend: Rust with Axum web framework
- Frontend: React with Tailwind CSS and shadcn/ui components
- Parser: Uses the gst-log-parser crate (integrated as a dependency) for parsing GStreamer logs

## License

This project is licensed under the same terms as the underlying gst-log-parser crate (MIT/Apache-2.0).
