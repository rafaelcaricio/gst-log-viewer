#!/bin/bash

set -eu

# Set environment variables
export HOME="/app/data"
export PORT="3000"
export RUST_BACKTRACE="1"
export RUST_LOG="info"

echo "Starting GStreamer Log Viewer..."
echo "Using binary: /app/code/gst-log-viewer"
echo "Current user: $(whoami)"
echo "Current working directory: $(pwd)"
echo "Trying to run the application..."

# Try to execute the binary without the shell 'exec' builtin
/app/code/gst-log-viewer