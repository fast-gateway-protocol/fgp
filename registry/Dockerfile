# Build stage
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source for dependency caching
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p src/api && \
    echo "fn main() {}" > src/api/main.rs && \
    echo "pub mod api;" > src/lib.rs && \
    echo "pub mod handlers; pub mod responses; pub mod routes; pub use handlers::AppState; pub use routes::create_router;" > src/api/mod.rs && \
    echo "" > src/api/handlers.rs && \
    echo "" > src/api/responses.rs && \
    echo "" > src/api/routes.rs

# Build dependencies only
RUN cargo build --release --features api --bin fgp-registry-api 2>/dev/null || true

# Copy actual source
COPY src ./src
COPY migrations ./migrations

# Build the actual binary
RUN touch src/lib.rs src/api/main.rs && \
    cargo build --release --features api --bin fgp-registry-api

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/fgp-registry-api /usr/local/bin/

# Create non-root user
RUN useradd -m -u 1000 appuser
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the server
CMD ["fgp-registry-api", "--host", "0.0.0.0", "--port", "8080"]
