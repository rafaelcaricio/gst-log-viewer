#!/bin/bash

set -eu

echo "=> Ensure directories"
mkdir -p /app/data/logs
mkdir -p /app/data/nginx/body /app/data/nginx/proxy /app/data/nginx/fastcgi /app/data/nginx/uwsgi /app/data/nginx/scgi

echo "=> Setting permissions"
chown -R cloudron:cloudron /app/data
chmod -R 755 /app/data/nginx

echo "=> Checking backend binary"
ls -la /app/code/gst-log-viewer
file /app/code/gst-log-viewer
# Don't try to chmod the binary, it's in a read-only filesystem
# chmod +x /app/code/gst-log-viewer

echo "=> Starting supervisord to manage processes"
exec /usr/bin/supervisord -c /app/pkg/supervisord.conf