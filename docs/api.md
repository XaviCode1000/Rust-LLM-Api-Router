# API Reference

Referencia completa de la API HTTP.

## Base URL

```
http://localhost:8080
```

## Autenticación

No requiere autenticación externa. Las API keys están almacenadas internamente y se rotan automáticamente.

---

## Endpoints

### Health Checks

#### GET /health

Health check básico.

**Request:**

```bash
GET /health
```

**Response 200:**

```json
{
  "status": "healthy",
  "timestamp": 1773335508,
  "version": "0.1.0"
}
```

---

#### GET /health/detail

Estado detallado del sistema con estadísticas.

**Request:**

```bash
GET /health/detail
```

**Response 200:**

```json
{
  "status": "healthy",
  "timestamp": 1773335515,
  "version": "0.1.0",
  "providers": {
    "total": 3,
    "enabled": 3,
    "disabled": 0
  },
  "accounts": {
    "total": 5,
    "active": 5,
    "inactive": 0
  }
}
```

---

#### GET /accounts

Lista de cuentas registradas (API keys enmascaradas).

**Request:**

```bash
GET /accounts
```

**Response 200:**

```json
[
  {
    "id": "groq-1",
    "provider_id": "groq",
    "is_active": true,
    "priority": 0,
    "api_key_prefix": "gsk_DVyb"
  },
  {
    "id": "groq-2",
    "provider_id": "groq",
    "is_active": true,
    "priority": 1,
    "api_key_prefix": "gsk_ABC"
  }
]
```

---

### API OpenAI-Compatible

#### POST /v1/chat/completions

Generar completado de chat compatible con OpenAI.

**Request:**

```bash
POST /v1/chat/completions
Content-Type: application/json
```

**Body:**

```json
{
  "model": "groq:llama-3.1-8b-instant",
  "messages": [
    {
      "role": "user",
      "content": "Hola!"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 1024,
  "stream": false
}
```

**Parámetros:**

| Campo | Tipo | Descripción | Default |
|-------|------|-------------|---------|
| `model` | string | Modelo en formato `provider:model` | - |
| `messages` | array[] | Lista de mensajes | - |
| `messages[].role` | string | Rol: "system", "user", "assistant" | - |
| `messages[].content` | string | Contenido del mensaje | - |
| `temperature` | number | Temperatura (0.0 - 2.0) | 0.7 |
| `max_tokens` | integer | Máximo de tokens a generar | 1024 |
| `stream` | boolean | Habilitar streaming (SSE) | false |

**Response 200:**

```json
{
  "id": "chatcmpl-653e9220-350f-4c66-8522-cda8c95a0bb8",
  "object": "chat.completion",
  "created": 1773335238,
  "model": "groq:llama-3.1-8b-instant",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "¡Hola! ¿Cómo puedo ayudarte hoy?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 8,
    "total_tokens": 18
  }
}
```

**Response 400 (Error):**

```json
{
  "error": {
    "message": "No active accounts found for provider 'groq'",
    "type": "no_accounts"
  }
}
```

**Response 502 (Provider Error):**

```json
{
  "error": {
    "message": "Request to provider failed: HTTP request failed",
    "type": "provider_error"
  }
}
```

---

#### GET /v1/models

Listar modelos disponibles.

**Request:**

```bash
GET /v1/models
```

**Response 200:**

```json
{
  "object": "list",
  "data": []
}
```

> **Nota:** Este endpoint está en desarrollo. Devuelve lista vacía actualmente.

---

### Métricas

#### GET /metrics

Métricas en formato Prometheus.

**Request:**

```bash
GET /metrics
```

**Response 200:**

```
# HELP llm_proxy_requests_total Total number of requests
# TYPE llm_proxy_requests_total counter
llm_proxy_requests_total 42

# HELP llm_proxy_request_duration_seconds Request duration in seconds
# TYPE llm_proxy_request_duration_seconds histogram
llm_proxy_request_duration_seconds_bucket{le="0.1"} 30
llm_proxy_request_duration_seconds_bucket{le="0.5"} 40
llm_proxy_request_duration_seconds_bucket{le="1.0"} 42
llm_proxy_request_duration_seconds_bucket{le="+Inf"} 42
```

---

## Formatos de Modelo

El campo `model` acepta estos formatos:

### `provider:model`

```json
{"model": "groq:llama-3.1-8b-instant"}
{"model": "openrouter:meta-llama/llama-3.2-3b-instruct:free"}
```

### `provider/model`

```json
{"model": "mistral/mistral-small-latest"}
{"model": "cerebras/llama3.1-8b"}
```

### Modelos por Proveedor

#### Groq

```
groq:llama-3.1-8b-instant
groq:llama-3.3-70b-versatile
groq:mixtral-8x7b-32768
```

#### OpenRouter

```
openrouter:meta-llama/llama-3.2-3b-instruct:free
openrouter:google/gemma-7b-it:free
openrouter:mistralai/mistral-7b-instruct:free
```

#### Mistral AI

```
mistral:mistral-small-latest
mistral:mistral-medium-latest
mistral:mistral-large-latest
```

---

## Códigos de Error

| Código | Tipo | Descripción |
|--------|------|-------------|
| 400 | `no_accounts` | No hay cuentas activas para el proveedor |
| 400 | `invalid_request` | Request inválido |
| 401 | `authentication_error` | API key inválida |
| 429 | `rate_limit` | Límite de tasa excedido |
| 502 | `provider_error` | Error del proveedor |
| 501 | `not_implemented` | Feature no implementada (ej: streaming) |

---

## Ejemplos

### cURL

```bash
# Chat simple
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.1-8b-instant",
    "messages": [{"role": "user", "content": "Hola!"}]
  }'

# Con temperatura y max_tokens
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.1-8b-instant",
    "messages": [{"role": "user", "content": "Explicá Rust en 1 oración"}],
    "temperature": 0.5,
    "max_tokens": 50
  }'
```

### Python (OpenAI SDK)

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="groq:llama-3.1-8b-instant",
    messages=[
        {"role": "system", "content": "Sos un asistente útil."},
        {"role": "user", "content": "Hola!"}
    ],
    temperature=0.7
)

print(response.choices[0].message.content)
```

### Node.js (OpenAI SDK)

```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
  baseURL: 'http://localhost:8080/v1',
  apiKey: 'not-needed'
});

const response = await openai.chat.completions.create({
  model: 'groq:llama-3.1-8b-instant',
  messages: [
    { role: 'user', content: 'Hola!' }
  ]
});

console.log(response.choices[0].message.content);
```

---

## Rotación y Failover

El sistema automáticamente:

1. **Selecciona cuenta** usando round-robin o prioridad
2. **Intenta request** con la cuenta seleccionada
3. **Reintenta** con otra cuenta si falla (máx 3 intentos)
4. **Abre circuit breaker** tras 5 fallos consecutivos
5. **Cierra circuit breaker** después de 30 segundos

### Ejemplo de Failover

```bash
# Si groq-1 falla, automáticamente usa groq-2
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "groq:llama-3.1-8b-instant", "messages": [{"role": "user", "content": "Hola!"}]}'
```

---

## Streaming (SSE)

> **Nota:** Streaming no implementado aún. Ver Issue #10.

Cuando esté disponible:

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.1-8b-instant",
    "messages": [{"role": "user", "content": "Hola!"}],
    "stream": true
  }'
```

Response será Server-Sent Events:

```
data: {"choices":[{"delta":{"content":"Hola"}}]}

data: {"choices":[{"delta":{"content":"!"}}]}

data: [DONE]
```
