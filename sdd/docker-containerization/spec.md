# Specification: Docker & Containerization (Issue #15)

## Requirements

### REQ-1: Multi-stage Dockerfile
The Dockerfile SHALL use at least 2 stages:
1. **Builder** — Compiles the Rust binary with release optimizations
2. **Runtime** — Minimal image containing only the binary and runtime dependencies

### REQ-2: Optimized Build Cache
The Dockerfile SHALL copy `Cargo.toml` and `Cargo.lock` first and build a dummy binary to cache dependencies before copying source code.

### REQ-3: Non-root User
The runtime container SHALL run as a non-root user (`llm-router`).

### REQ-4: Health Check
The Dockerfile SHALL include a `HEALTHCHECK` instruction that calls `GET /health` on port 8080.

### REQ-5: Data Volume
The container SHALL expose a `/data` volume mapped to `XDG_CONFIG_HOME` for persistent storage of `accounts.json` and `providers.json`.

### REQ-6: Environment Variables
The container SHALL support all existing environment variables:
- `PORT`, `HOST`, `LOG_LEVEL`
- `ROUTING_STRATEGY`, `CASCADING_ENABLED`, `CASCADING_MIN_QUALITY`, etc.

### REQ-7: .dockerignore
The `.dockerignore` file SHALL exclude: `target/`, `.git/`, `.github/`, `coverage*/`, `sdd/`, `docs/`, `*.md` (except README)

### REQ-8: docker-compose.yml
The `docker-compose.yml` SHALL:
- Build from local Dockerfile
- Expose port 8080
- Mount `./data:/data` volume
- Include healthcheck configuration
- Set default environment variables

### REQ-9: GitHub Actions Docker Workflow
A GitHub Actions workflow SHALL:
- Trigger on push to `main` and tags
- Build and push to `ghcr.io/xavicode1000/rust-llm-api-router`
- Tag with `latest` and git tag version

### REQ-10: Documentation
`docs/deployment.md` SHALL document:
- How to build the Docker image locally
- How to run with docker-compose
- How to configure via environment variables
- How to persist data with volumes
- How to deploy to GHCR

## Scenarios

### Scenario 1: Build Docker Image Locally
**Given** user runs `docker build -t llm-router .`
**When** build completes
**Then** image is created with size < 100MB

### Scenario 2: Run with docker-compose
**Given** user runs `docker compose up -d`
**When** container starts
**Then** server is accessible at `http://localhost:8080/health`

### Scenario 3: Persistent Data
**Given** user mounts `./data:/data` volume
**When** providers and accounts are added
**And** container is restarted
**Then** data persists across restarts

### Scenario 4: Health Check
**Given** container is running
**When** Docker checks health
**Then** `GET /health` returns 200 with `{"status":"healthy"}`

### Scenario 5: CI/CD Auto-publish
**Given** code is pushed to `main`
**When** GitHub Actions runs
**Then** Docker image is built and pushed to GHCR

### Scenario 6: Non-root Security
**Given** container is running
**When** checking process user
**Then** process runs as `llm-router` user, not root
