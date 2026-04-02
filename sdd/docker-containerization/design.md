# Technical Design: Docker & Containerization (Issue #15)

## Architecture

### Dockerfile (Multi-stage)

```dockerfile
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
```

### .dockerignore

```
# Build artifacts
target/
**/*.rs.bk
*.pdb

# Git
.git/
.gitignore

# IDE
.vscode/
.idea/
*.swp
*.swo

# Documentation
docs/
*.md
!README.md

# SDD artifacts
sdd/
openspec/

# Coverage
coverage*/
*.lcov

# CI/CD
.github/

# OS files
.DS_Store
Thumbs.db

# Env files (don't leak secrets)
.env
.env.*
!.env.example
```

### docker-compose.yml

```yaml
services:
  llm-router:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: llm-router
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
    environment:
      - HOST=0.0.0.0
      - PORT=8080
      - LOG_LEVEL=info
      - ROUTING_STRATEGY=auto
      - CASCADING_ENABLED=false
      - CASCADING_MIN_QUALITY=0.75
      - BUDGET_MODE=false
      - MAX_RETRIES=3
      - REQUEST_TIMEOUT_SECONDS=60
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
```

### GitHub Actions (`.github/workflows/docker.yml`)

```yaml
name: Docker

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  docker:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=raw,value=latest,enable=${{ github.ref == 'refs/heads/main' }}
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha,format=short

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### docs/deployment.md

Documentation covering:
1. Prerequisites (Docker, Docker Compose)
2. Building locally (`docker build -t llm-router .`)
3. Running with docker-compose
4. Environment variables reference
5. Data persistence with volumes
6. Health check configuration
7. Pulling from GHCR

## Files Created

| File | Lines | Description |
|------|-------|-------------|
| `Dockerfile` | ~45 | Multi-stage optimized build |
| `.dockerignore` | ~30 | Exclude build artifacts |
| `docker-compose.yml` | ~25 | Local development setup |
| `.github/workflows/docker.yml` | ~40 | CI/CD Docker build and push |
| `docs/deployment.md` | ~80 | Deployment documentation |

## Image Size Estimate

| Stage | Size |
|-------|------|
| Builder image | ~1.5GB (rust:1.93-slim + build deps) |
| Runtime image | ~50-80MB (debian:bookworm-slim + binary + certs) |
| Binary (stripped) | ~8-12MB |
