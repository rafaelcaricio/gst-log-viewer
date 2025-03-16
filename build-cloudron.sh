#!/bin/bash

set -eu

echo "Building GStreamer Log Viewer for Cloudron..."

# Build the Docker image
echo "Building Docker image..."
docker build --platform linux/amd64 -t gst-log-viewer .

# Check if a tag was provided
if [ $# -eq 1 ]; then
    TAG=$1
    echo "Tagging image as ${TAG}..."
    docker tag gst-log-viewer ${TAG}
    
    echo "Do you want to push the image to Docker Hub? (y/n)"
    read -r PUSH
    
    if [ "$PUSH" = "y" ]; then
        echo "Pushing image to Docker Hub..."
        docker push ${TAG}
        echo "Image pushed successfully!"
    fi
fi

echo "Build completed successfully!"
echo "To deploy to Cloudron, use the CloudronManifest.json file."
echo "Now the app will be accessible via Nginx on port 8000!"