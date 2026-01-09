# Build stage
FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies (this caches them in Docker layer)
RUN cargo build --release

# Now copy the actual source code
COPY src ./src
COPY schema.sql ./

# Touch the main.rs file to ensure it's newer than the dummy
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Create necessary directories
RUN mkdir -p /app/uploads && chown -R appuser:appuser /app

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/backend /app/backend
COPY --from=builder /usr/src/app/schema.sql /app/schema.sql

# Copy .env.example as a reference (actual .env should be mounted or set via environment variables)
COPY .env .env.example

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose ports (both HTTP and HTTPS as per config)
EXPOSE 8000 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

# Environment variables (can be overridden at runtime)
ENV DATABASE_URL="postgresql://postgres:password@localhost:5432/postgres"
ENV HOST_URL="0.0.0.0"
ENV JWT_SECRET_KEY="change_this_in_production"
ENV JWT_MAXAGE="60"
ENV PORT_HTTP="8000"
ENV PORT_HTTPS="8080"
ENV UPLOAD_DIR="./uploads"
ENV MAX_FILE_SIZE="5242880"
ENV VERIFY_EMAIL="false"
ENV ENABLE_HTTPS="false"

# Run the application
CMD ["/app/backend"]

