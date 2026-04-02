# Deployment

## Docker

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

### Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| HOST | 0.0.0.0 | Host to bind |
| PORT | 8080 | Port to bind |
| LOG_LEVEL | info | trace, debug, info, warn, error |
| ROUTING_STRATEGY | auto | auto, cost-optimized, cascading, failover, load-balanced |
| CASCADING_ENABLED | false | Enable cascading routing |
| CASCADING_MIN_QUALITY | 0.75 | Minimum quality score (0.0-1.0) |
| BUDGET_MODE | false | Enable budget mode |
| MAX_RETRIES | 3 | Maximum retries per request |
| REQUEST_TIMEOUT_SECONDS | 60 | Request timeout |

### Data Persistence
Mount a volume to `/data` (mapped to `XDG_CONFIG_HOME`):
```bash
docker run -v $(pwd)/data:/data llm-router
```

This persists `accounts.json` and `providers.json` across container restarts.

### Health Check
The container includes a health check at `GET /health`:
```bash
curl http://localhost:8080/health
# {"status":"healthy","timestamp":1234567890,"version":"0.1.0"}
```

### Pull from GHCR
```bash
docker pull ghcr.io/xavicode1000/rust-llm-api-router:latest
```