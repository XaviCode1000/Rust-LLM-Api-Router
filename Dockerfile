# ============================================================
# Stage 1: Builder — Compile with optimized cache
# ============================================================
FROM rust:1.93-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies: copy manifests and build a dummy binary
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source and build
COPY . .
RUN cargo build --release --locked && \
    strip target/release/llm-router

# ============================================================
# Stage 2: Runtime — Minimal production image
# ============================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies (SSL certs, curl for healthcheck)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false llm-router

# Create data directory
RUN mkdir -p /data && chown llm-router:llm-router /data

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/llm-router .

# Switch to non-root user
USER llm-router

# Environment variables
ENV XDG_CONFIG_HOME=/data
ENV HOST=0.0.0.0
ENV PORT=8080
ENV LOG_LEVEL=info

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Volume for persistent data
VOLUME ["/data"]

# Entry point
ENTRYPOINT ["./llm-router"]
CMD ["--host", "0.0.0.0", "--port", "8080"]