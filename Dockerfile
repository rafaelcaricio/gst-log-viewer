## Builder stage for frontend
FROM node:20-slim AS frontend-builder

WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

## Builder stage for backend
FROM rust:1-slim-bullseye AS backend-builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml .
COPY src/ ./src/

# Build the application
RUN cargo build --release
# Ensure the binary is executable
RUN chmod +x /app/target/release/gst-log-viewer

## Builder stage for C wrapper
FROM gcc:latest AS wrapper-builder

WORKDIR /app
COPY cloudron-wrapper.c .
RUN gcc -static -o wrapper cloudron-wrapper.c

## Final stage
FROM cloudron/base:4.2.0

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libgstreamer1.0-0 \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    nginx \
    supervisor \
    file \
    && rm -rf /var/lib/apt/lists/*

# Set up directories
RUN mkdir -p /app/code/frontend/dist /app/data/logs /app/data/nginx

# Copy built assets from builder stages
COPY --from=backend-builder /app/target/release/gst-log-viewer /app/code/
COPY --from=frontend-builder /app/dist/ /app/code/frontend/dist/
COPY --from=wrapper-builder /app/wrapper /app/code/wrapper
RUN chmod +x /app/code/wrapper

# Copy configuration files
COPY supervisord.conf /app/pkg/
COPY nginx.conf /etc/nginx/nginx.conf
COPY run-app.sh /app/pkg/

# Remove default nginx site and logs directory
RUN rm -rf /etc/nginx/sites-enabled/default /var/log/nginx

# Copy startup script
COPY start.sh /app/pkg/
RUN chmod +x /app/pkg/start.sh
RUN chmod +x /app/pkg/run-app.sh

# Set proper permissions
RUN chown -R cloudron:cloudron /app/data
RUN mkdir -p /run/nginx
RUN chown -R cloudron:cloudron /run/nginx

# Expose the Nginx port
EXPOSE 8000

CMD ["/app/pkg/start.sh"]