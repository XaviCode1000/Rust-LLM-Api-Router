# Deployment

## Docker (Issue #15)

### Prerequisites
- Docker 20.10+
- Docker Compose v2+

### Build Locally
```bash
docker build -t llm-router .
```

### Run with Docker Compose
```bash
docker compose up -d
```

### Run Manually
```bash
docker run -d \
  --name llm-router \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e LOG_LEVEL=info \
  -e ROUTING_STRATEGY=auto \
  llm-router
```

### Pull from GHCR
```bash
docker pull ghcr.io/xavicode1000/rust-llm-api-router:latest
docker run -d -p 8080:8080 ghcr.io/xavicode1000/rust-llm-api-router:latest
```

### Data Persistence

Mount a volume to `/data` (mapped to `XDG_CONFIG_HOME`):
```bash
docker run -v $(pwd)/data:/data llm-router
```

This persists `accounts.json`, `providers.json`, and secure storage across container restarts.

### Health Check
The container includes a health check at `GET /health`:
```bash
curl http://localhost:8080/health
# {"status":"healthy","timestamp":1234567890,"version":"0.1.0"}
```

## Environment Variables

### Server Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| HOST | 0.0.0.0 | Host to bind |
| PORT | 8080 | Port to bind |
| LOG_LEVEL | info | trace, debug, info, warn, error |

### Routing Configuration (Issue #29)
| Variable | Default | Description |
|----------|---------|-------------|
| ROUTING_STRATEGY | auto | auto, cost-optimized, cascading, failover, load-balanced |
| CASCADING_ENABLED | false | Enable cascading routing |
| CASCADING_MIN_QUALITY | 0.75 | Minimum quality score (0.0-1.0) |
| CASCADING_MAX_TIERS | 3 | Maximum tiers to try |
| CASCADING_PER_TIER_TIMEOUT_MS | 5000 | Timeout per tier |
| BUDGET_MODE | false | Enable budget mode |
| MAX_RETRIES | 3 | Maximum retries per request |
| REQUEST_TIMEOUT_SECONDS | 60 | Request timeout |

### Secure Storage (Issue #22)
| Variable | Default | Description |
|----------|---------|-------------|
| SECURE_STORAGE | auto | auto, keyring, encrypted, disabled |

## Security Considerations for Production

### 1. Use System Keyring

In production, ensure the system keyring is available:
- **Linux**: Install `libsecret` or `libdbus` for Secret Service support
- **Docker**: May need to mount socket or use `SECURE_STORAGE=encrypted` fallback

```bash
# Force encrypted storage in Docker (if keyring unavailable)
docker run -e SECURE_STORAGE=encrypted ...
```

### 2. Volume Mounting for Secure Storage

If using encrypted file storage, mount a persistent volume:

```yaml
# docker-compose.yml
services:
  llm-router:
    image: ghcr.io/xavicode1000/rust-llm-api-router:latest
    volumes:
      - llm-router-data:/data
      - llm-router-secrets:/secrets  # Encrypted storage

volumes:
  llm-router-data:
  llm-router-secrets:
```

### 3. Network Security

```bash
# Run with restricted network
docker run --network=none llm-router

# Or with specific network
docker network create llm-net
docker run --network=llm-net llm-router
```

### 4. Resource Limits

```bash
# Limit CPU and memory
docker run --cpus=1 --memory=512m llm-router
```

### 5. Read-Only Root Filesystem

```bash
# For additional security
docker run --read-only --tmpfs /tmp llm-router
```

## Kubernetes (Basic)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-router
spec:
  replicas: 1
  selector:
    matchLabels:
      app: llm-router
  template:
    metadata:
      labels:
        app: llm-router
    spec:
      containers:
      - name: llm-router
        image: ghcr.io/xavicode1000/rust-llm-api-router:latest
        ports:
        - containerPort: 8080
        env:
        - name: ROUTING_STRATEGY
          value: "cascading"
        - name: SECURE_STORAGE
          value: "keyring"
        resources:
          limits:
            cpu: "1"
            memory: "512Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: llm-router
spec:
  selector:
    app: llm-router
  ports:
  - port: 80
    targetPort: 8080
```

## Monitoring

### Health Endpoint
```bash
curl http://localhost:8080/health
```

### Detailed Health
```bash
curl http://localhost:8080/health/detail
```

### Metrics (Prometheus)
```bash
curl http://localhost:8080/metrics
```