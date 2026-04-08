# LLM API Router

<p align="center">
  <a href="https://github.com/XaviCode1000/Rust-LLM-Api-Router/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/XaviCode1000/Rust-LLM-Api-Router/ci.yml?branch=main&label=CI" alt="CI Status">
  </a>
  <a href="https://codecov.io/gh/XaviCode1000/Rust-LLM-Api-Router">
    <img src="https://img.shields.io/codecov/c/github/XaviCode1000/Rust-LLM-Api-Router?label=Coverage" alt="Coverage">
  </a>
  <a href="https://crates.io/crates/rust-llm-api-router">
    <img src="https://img.shields.io/crates/v/rust-llm-api-router" alt="Crate">
  </a>
  <a href="https://docs.rs/rust-llm-api-router">
    <img src="https://img.shields.io/docsrs/rust-llm-api-router" alt="Documentation">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT">
  </a>
</p>

**Un solo endpoint para 34 proveedores de IA.** Ahorrá costos, evitá caídas, y mantené el control de tus APIs — todo con una interfaz compatible con OpenAI.

---

## ¿Qué Hace?

LLM API Router es un **proxy inteligente** que se sienta entre tus aplicaciones y los proveedores de IA (OpenAI, Anthropic, Groq, etc.) y hace tres cosas:

1. **Balancea costos** — Usa el modelo más barato que pueda manejar tu consulta
2. **Previene caídas** — Si un proveedor falla, rota automáticamente a otro
3. **Consolida APIs** — Todos los proveedores hablan el mismo idioma (OpenAI-compatible)

> **¿Para quién es?** Equipos que usan múltiples proveedores de IA, quieren reducir costos de API, necesitan alta disponibilidad, o ambos.

---

## Empezar en 5 Minutos

### 1. Instalá

```bash
# Docker (recomendado)
docker pull ghcr.io/xavicode1000/rust-llm-api-router:latest
docker run -d -p 8080:8080 ghcr.io/xavicode1000/rust-llm-api-router:latest

# O compilá desde fuente
cargo build --release
./target/release/llm-router --port 8080
```

### 2. Registrá un Proveedor

```bash
./target/release/llm-router provider add \
  --id groq \
  --name "Groq" \
  --base-url "https://api.groq.com/openai/v1"
```

### 3. Agregá tu API Key

```bash
./target/release/llm-router account add \
  --id mi-groq \
  --provider groq \
  --api-key "tu-api-key"
```

### 4. Usalo

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hola!"}]
  }'
```

**Listo.** Tu app ahora habla con Groq a través del router.

---

## Funcionalidades Principales

| Función | Qué Hace |
|---------|----------|
| **34 Proveedores** | OpenAI, Anthropic, Groq, Mistral, Ollama, y 29 más (incluyendo 5 con free tier permanente) |
| **Multi-Cuenta** | Múltiples API keys por proveedor con rotación automática |
| **Failover Automático** | Si un proveedor cae, rota a otro sin que lo notes |
| **Streaming (SSE)** | Respuestas token por token en tiempo real |
| **Ahorro de Costos** | Elige el modelo más barato capaz de manejar tu consulta |
| **CLI Interactiva** | Gestioná todo desde la terminal con colores y tablas |
| **API Compatible OpenAI** | Drop-in replacement — tu app no necesita cambios |

---

## Estrategias de Enrutamiento Inteligente

El router puede **pensar antes de enviar** tu request:

| Estrategia | Cuándo Usarla |
|------------|---------------|
| **Cost-Aware** | Querés el modelo más barato posible para cada tipo de consulta |
| **Cascading** | Empezá barato, escalá si la calidad no alcanza |
| **Task-Based** | Diferentes modelos para código, chat, razonamiento, etc. |
| **Failover** | Alta disponibilidad — si uno falla, otro responde |

**Ejemplo rápido:**
```bash
# Activar enrutamiento cascading (empieza barato, escala si hace falta)
./target/release/llm-router --routing-strategy cascading --quality-threshold 0.85
```

> 📖 **Guía completa de enrutamiento:** [docs/routing.md](docs/routing.md)

---

## Proveedores Soportados

### Principales (Testeados ✅)

| Proveedor | Estado |
|-----------|--------|
| OpenAI | ✅ |
| Anthropic | ✅ |
| Groq | ✅ |
| Mistral AI | ✅ |
| OpenRouter | ✅ |
| Cerebras | ✅ |
| Cloudflare Workers AI | ✅ |

### Otros 27 Proveedores

**Locales:** Ollama, LM Studio, vLLM
**Enterprise:** Azure OpenAI, AWS Bedrock, Google Vertex AI
**Plataformas:** DeepSeek, Together, Fireworks AI, xAI/Grok, Perplexity, Replicate, Anyscale, DeepInfra, Novita AI, SambaNova, HuggingFace, AI21 Labs, Aleph Alpha, NVIDIA NIM, Cohere, Google AI Studio
**Free Tier Permanente:** Zhipu AI (GLM-4 Flash), GitHub Models (GPT-4o gratis), Kluster AI, LLM7.io, SiliconFlow

> 📋 **Lista completa con URLs:** [docs/architecture.md](docs/architecture.md)

---

## Modelos Verificados

### Groq
- `llama-3.3-70b-versatile` ✅
- `llama-3.1-8b-instant` ✅
- `groq/compound` ✅
- `groq/compound-mini` ✅

> ⚠️ Modelos viejos de Groq como `llama3-8b-8192` fueron descontinuados.

---

## Uso con Tu App

### OpenCode

```json
{
  "model": "groq:llama-3.3-70b-versatile",
  "provider": {
    "openai": {
      "options": {
        "baseURL": "http://localhost:8080/v1",
        "apiKey": "demo-key"
      }
    }
  }
}
```

### Cualquier Cliente OpenAI

Solo cambiá la `baseURL` a `http://localhost:8080/v1` y listo.

---

## Comandos CLI Esenciales

### Proveedores

```bash
./target/release/llm-router provider list          # Ver proveedores
./target/release/llm-router provider add ...       # Agregar
./target/release/llm-router provider enable ...    # Habilitar
./target/release/llm-router provider models ...    # Ver modelos disponibles
```

### Cuentas

```bash
./target/release/llm-router account list           # Ver cuentas
./target/release/llm-router account add ...        # Agregar API key
./target/release/llm-router account validate ...   # Verificar que funcione
```

### Autenticación Segura

```bash
./target/release/llm-router auth login --provider groq     # OAuth (abre navegador)
./target/release/llm-router auth login --provider groq --device-flow  # Headless
./target/release/llm-router logout --all                   # Cerrar sesión
```

> 📖 **Referencia completa de CLI:** [docs/cli.md](docs/cli.md)

---

## Endpoints de API

| Endpoint | Qué Hace |
|----------|----------|
| `POST /v1/chat/completions` | Enviar consulta (compatible OpenAI) |
| `GET /v1/models` | Listar modelos disponibles |
| `GET /health` | Health check básico |
| `GET /health/detail` | Estado detallado del sistema |
| `GET /accounts` | Ver cuentas registradas |
| `GET /metrics` | Métricas Prometheus |

> 📖 **Referencia completa de API:** [docs/api.md](docs/api.md)

---

## Configuración

### Variables de Entorno

```bash
PORT=8080                           # Puerto del servidor
HOST=0.0.0.0                        # Host
LOG_LEVEL=info                      # trace, debug, info, warn, error
ROUTING_STRATEGY=auto               # auto, cost-optimized, cascading, failover
CASCADING_MIN_QUALITY=0.75         # Umbral de calidad mínimo
SECURE_STORAGE=auto                 # auto, keyring, encrypted, disabled
```

### Archivos de Configuración

```
~/.config/rust-llm-api-router/
├── providers.json    # Proveedores registrados
└── accounts.json     # Cuentas con API keys
```

---

## Seguridad

- **API Keys:** Guardadas en el llavero del sistema (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **OAuth 2.1 / PKCE:** Autenticación segura con navegador
- **Device Flow:** Para entornos sin navegador (servidores, CI/CD)
- **Cifrado:** AES-256-GCM con Argon2id como fallback

> 📖 **Detalles de seguridad:** [docs/security.md](docs/security.md)

---

## Calidad del Proyecto

| Métrica | Valor |
|---------|-------|
| **Tests** | 492 pasando |
| **Cobertura** | 80.35% |
| **Proveedores** | 29 soportados |
| **Licencia** | MIT |

---

## Para Desarrolladores

¿Querés contribuir, entender la arquitectura, o ver cómo funciona por dentro?

| Tema | Dónde |
|------|-------|
| **Arquitectura** | [docs/architecture.md](docs/architecture.md) |
| **Guía de Desarrollo** | [DEVELOPMENT.md](DEVELOPMENT.md) |
| **Estrategias de Routing** | [docs/routing.md](docs/routing.md) |
| **Testing** | [docs/TESTING_GUIDE.md](docs/TESTING_GUIDE.md) |
| **Deployment** | [docs/deployment.md](docs/deployment.md) |
| **Seguridad** | [docs/security.md](docs/security.md) |
| **CLI Reference** | [docs/cli.md](docs/cli.md) |
| **API Reference** | [docs/api.md](docs/api.md) |

---

## Contribuir

1. Fork el repo
2. Creá una rama (`git checkout -b feature/mi-feature`)
3. Commiteá (`git commit -m 'feat: agregué X'`)
4. Pusheá (`git push origin feature/mi-feature`)
5. Abrí un PR

---

## Roadmap

### ✅ Completado

- Streaming SSE
- 34 proveedores soportados (incluyendo 5 free tier)
- Cost-Aware Routing (#23)
- Cascading Routing (#24)
- Task-Based Routing (#26)
- CLI moderna con colores y tablas (#19)
- Almacenamiento seguro de API keys (#22)
- Docker y GHCR (#15)
- 80%+ cobertura de tests

### 🔄 Próximo

- Kubernetes manifests
- Testing completo de los 34 proveedores
- Guías por proveedor

---

## Licencia

MIT — ver [LICENSE](LICENSE) para detalles.
