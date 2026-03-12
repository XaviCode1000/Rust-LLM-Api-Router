# LLM API Router

Proxy/router de API LLM con arquitectura limpia en Rust. Permite usar múltiples proveedores (Groq, OpenRouter, Mistral, etc.) con rotación automática de cuentas y failover.

## Características

- ✅ **Multi-proveedor**: Soporta 12+ proveedores LLM gratuitos
- ✅ **Multi-cuenta**: Múltiples API keys por proveedor con rotación automática
- ✅ **Failover automático**: Circuit breaker y reintentos inteligentes
- ✅ **API OpenAI-compatible**: Usa clientes OpenAI sin cambios
- ✅ **Health checks**: Monitoreo completo del sistema
- ✅ **CLI integrada**: Gestión de proveedores y cuentas desde terminal

## Inicio Rápido

### 1. Compilar

```bash
cargo build --release
```

### 2. Registrar Proveedores

```bash
# Registrar un proveedor
./target/release/llm-router provider add \
  --id groq \
  --name "Groq" \
  --base-url "https://api.groq.com/openai/v1"

# Listar proveedores
./target/release/llm-router provider list
```

### 3. Registrar Cuentas con API Keys

```bash
# Registrar cuenta con API key
./target/release/llm-router account add \
  --id groq-1 \
  --provider groq \
  --api-key "tu-api-key-aqui" \
  --priority 0

# Listar cuentas
./target/release/llm-router account list
```

### 4. Iniciar Servidor

```bash
./target/release/llm-router --port 8080
```

### 5. Probar API

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.1-8b-instant",
    "messages": [{"role": "user", "content": "Hola!"}]
  }'
```

## API Reference

### POST /v1/chat/completions

Endpoint compatible con OpenAI.

**Request:**
```json
{
  "model": "groq:llama-3.1-8b-instant",
  "messages": [
    {"role": "user", "content": "Hola!"}
  ],
  "temperature": 0.7,
  "max_tokens": 1024
}
```

**Response:**
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1773335238,
  "model": "groq:llama-3.1-8b-instant",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "¡Hola! ¿Cómo puedo ayudarte?"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 8,
    "total_tokens": 18
  }
}
```

### GET /health

Health check básico.

```bash
curl http://localhost:8080/health
# {"status":"healthy","timestamp":1773335508,"version":"0.1.0"}
```

### GET /health/detail

Estado detallado del sistema.

```bash
curl http://localhost:8080/health/detail
```

### GET /accounts

Lista de cuentas registradas (API keys enmascaradas).

```bash
curl http://localhost:8080/accounts
```

### GET /metrics

Métricas Prometheus.

```bash
curl http://localhost:8080/metrics
```

## CLI Reference

### Proveedores

```bash
# Agregar proveedor
llm-router provider add --id <id> --name <nombre> --base-url <url> [--disabled]

# Listar proveedores
llm-router provider list

# Habilitar proveedor
llm-router provider enable --id <id>

# Deshabilitar proveedor
llm-router provider disable --id <id>

# Eliminar proveedor
llm-router provider remove --id <id>

# Validar proveedor
llm-router provider validate --id <id>
```

### Cuentas

```bash
# Agregar cuenta
llm-router account add \
  --id <id> \
  --provider <provider_id> \
  --api-key <key> \
  [--priority <n>] \
  [--interactive]

# Listar cuentas
llm-router account list

# Cambiar prioridad
llm-router account set-priority --id <id> --priority <n>

# Eliminar cuenta
llm-router account remove --id <id>

# Validar cuenta
llm-router account validate --id <id>
```

## Proveedores Soportados

| Proveedor | Base URL | Estado |
|-----------|----------|--------|
| Groq | https://api.groq.com/openai/v1 | ✅ |
| OpenRouter | https://openrouter.ai/api/v1 | ✅ |
| Mistral AI | https://api.mistral.ai/v1 | ✅ |
| Cerebras | https://api.cerebras.ai/v1 | ✅ |
| Cloudflare Workers AI | https://api.cloudflare.com/client/v4/accounts | ✅ |
| NVIDIA NIM | https://integrate.api.nvidia.com/v1 | 🔄 |
| Hugging Face | https://api-inference.huggingface.co/models | 🔄 |
| DeepSeek | https://api.deepseek.com/v1 | 🔄 |
| xAI (Grok) | https://api.x.ai/v1 | 🔄 |
| Cohere | https://api.cohere.ai/v1 | 🔄 |
| AI21 | https://api.ai21.com/studio/v1 | 🔄 |
| Google AI Studio | https://generativelanguage.googleapis.com/v1beta | 🔄 |

✅ = Probado | 🔄 = Soportado (pendiente test)

## Estrategias de Rotación

El sistema soporta 4 estrategias de rotación de cuentas:

1. **Round-Robin**: Rotación secuencial entre cuentas
2. **Weighted**: Basado en prioridad (menor número = mayor prioridad)
3. **Latency**: Selecciona cuenta con menor latencia (WIP)
4. **User-Affinity**: Misma cuenta por usuario (WIP)

## Failover Automático

- **Circuit Breaker**: Se abre tras 5 fallos consecutivos
- **Auto-cierre**: Reintenta después de 30 segundos
- **Health Scoring**: Puntuación 0-100 basada en éxito/latencia

## Configuración

### Variables de Entorno

```bash
# Puerto del servidor (default: 8080)
PORT=8080

# Host (default: 0.0.0.0)
HOST=0.0.0.0

# Nivel de log (trace, debug, info, warn, error)
LOG_LEVEL=info
```

### Archivos de Configuración

Los datos se guardan en el directorio de configuración XDG:

```
~/.config/rust-llm-api-router/
├── providers.json    # Proveedores registrados
└── accounts.json     # Cuentas con API keys
```

## Scripts de Bootstrap

### register-providers.sh

Registra 12+ proveedores automáticamente:

```bash
./scripts/register-providers.sh
```

### register-accounts.sh

Registra proveedores y cuentas usando API keys de copyq:

```bash
./scripts/register-accounts.sh
```

## Arquitectura

El sistema sigue **Clean Architecture**:

```
Domain Layer        (Entidades, Traits, Errores)
    ↓
Application Layer   (Servicios, Rotación, Failover)
    ↓
Infrastructure      (HTTP Client, Persistencia JSON)
    ↓
Presentation        (Handlers HTTP, Routes)
```

### Dependencias

- **Web Framework**: Axum 0.7
- **HTTP Client**: reqwest 0.11
- **Async Runtime**: tokio 1.x
- **Serialización**: serde + serde_json
- **CLI**: clap 4.4
- **Métricas**: prometheus 0.13
- **Logs**: tracing + tracing-subscriber

## Desarrollo

### Build

```bash
cargo build
```

### Tests

```bash
cargo test
```

### Lint

```bash
cargo clippy
cargo fmt
```

## Roadmap

- [ ] Streaming (SSE) para /v1/chat/completions
- [ ] Endpoint /v1/models con lista real
- [ ] Adapters para Anthropic, Google, Cohere
- [ ] Docker + Kubernetes manifests
- [ ] CI/CD pipeline

## Issues

- [Issue #6](https://github.com/XaviCode1000/Rust-LLM-Api-Router/issues/11) - Deployment (Docker, K8s)
- [Issue #10](https://github.com/XaviCode1000/Rust-LLM-Api-Router/issues/10) - Future Improvements

## Licencia

MIT

## Autores

Desarrollado con Arquitectura Limpia y patrones DDD en Rust.
