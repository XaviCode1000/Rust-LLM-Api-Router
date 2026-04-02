# Change Proposal: Docker & Containerization (Issue #15)

## Intent

Agregar containerización con Docker al proyecto para facilitar deployment, desarrollo local y distribución.

## Problem

Actualmente no hay forma de ejecutar el LLM API Router en un contenedor Docker. Esto significa:

- **No hay reproducibilidad** — cada entorno necesita instalar Rust y compilar
- **No hay aislamiento** — dependencias del sistema afectan la app
- **No hay facilidad de deployment** — no se puede desplegar en Kubernetes, ECS, etc.
- **Desarrollo local complejo** — necesita toolchain de Rust instalado
- **No hay imagen publicada** — no hay Docker Hub ni GHCR

## Scope

### Incluido
1. **Multi-stage Dockerfile** optimizado con build cache y imagen final mínima (scratch o distroless)
2. **`.dockerignore`** para excluir archivos innecesarios del build context
3. **`docker-compose.yml`** para desarrollo local con volume mounting
4. **GitHub Actions workflow** para build y push automático a GHCR
5. **Documentación** de deployment en `docs/deployment.md`

### NO Incluido
- Docker Swarm o Kubernetes manifests (fuera de scope)
- Multi-arch builds (amd64 + arm64) — se puede agregar después
- Docker secrets management — se usa env vars

## Approach

### Multi-stage Dockerfile

**Stage 1: Builder** — Compila con cache optimizado
```dockerfile
FROM rust:1.93-slim AS builder
WORKDIR /app
# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src
# Copy actual source
COPY . .
RUN cargo build --release --locked
```

**Stage 2: Runtime** — Imagen mínima con solo el binario
```dockerfile
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /bin/false llm-router
USER llm-router
WORKDIR /app
COPY --from=builder /app/target/release/llm-router .
ENV XDG_CONFIG_HOME=/data
VOLUME ["/data"]
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
ENTRYPOINT ["./llm-router"]
CMD ["--host", "0.0.0.0", "--port", "8080"]
```

### docker-compose.yml

```yaml
services:
  llm-router:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
    environment:
      - LOG_LEVEL=info
      - ROUTING_STRATEGY=auto
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
```

### GitHub Actions

```yaml
jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          push: true
          tags: ghcr.io/xavicode1000/rust-llm-api-router:latest
```

## Impact

### Files Created
| File | Description |
|------|-------------|
| `Dockerfile` | Multi-stage optimized build |
| `.dockerignore` | Exclude build artifacts, git, etc. |
| `docker-compose.yml` | Local development setup |
| `.github/workflows/docker.yml` | Automated build and push to GHCR |
| `docs/deployment.md` | Deployment documentation |

### Risks
- **Build time**: First build takes ~10-15 min, but subsequent builds use cache
- **Image size**: ~50-80MB final image (slim debian + static binary)
- **Data persistence**: Users must mount `/data` volume or lose config on restart

## Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| **scratch image** | No SSL certs, no curl for healthcheck |
| **alpine** | musl vs glibc issues with some Rust crates |
| **distroless** | No shell for debugging, harder to maintain |
| **debian:bookworm-slim** | ✅ Best balance: small, glibc, has apt for certs |
