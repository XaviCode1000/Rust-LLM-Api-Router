# LLM API Router

> Un solo comando. 34 proveedores de IA. Olvidate de configuraciones.

[![CI](https://github.com/XaviCode1000/Rust-LLM-Api-Router/actions/workflows/ci.yml/badge.svg)](https://github.com/XaviCode1000/Rust-LLM-Api-Router/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

---

## TL;DR

```bash
# Esto inicia el servidor
docker run -d -p 8080:8080 ghcr.io/xavicode1000/rust-llm-api-router:latest
```

Listo. Ya está corriendo en `http://localhost:8080`

---

## Para Qué Sirve?

| Problema | Solución |
|----------|----------|
| Tengo muchas API keys | **Una sola** para todas |
| Me cae un proveedor | **Se encarga solo** |
| Quiero pagar menos | **Elige el más barato** automáticamente |
| No quiero configurar nada | **Docker y listo** |

---

## 1. Instalar

### Opción 1: Script Automático (Recomendado)

Una línea que lo instala sola:

```bash
# Usuario (se instala en ~/.local/bin o ~/.cargo/bin)
curl -sS https://raw.githubusercontent.com/XaviCode1000/Rust-LLM-Api-Router/main/scripts/install.sh | sh

# Sistema (se instala en /usr/local/bin)
curl -sS https://raw.githubusercontent.com/XaviCode1000/Rust-LLM-Api-Router/main/scripts/install.sh | sh -s -- --bin-dir /usr/local/bin
```

El script detecta tu sistema operativo y arquitectura automáticamente.

### Opción 2: Docker (Rápido)

```bash
docker run -d -p 8080:8080 ghcr.io/xavicode1000/rust-llm-api-router:latest
```

### Opción 3: Binario Manual

```bash
# Descargá de: https://github.com/XaviCode1000/Rust-LLM-Api-Router/releases
# Linux:   llm-router-x86_64-unknown-linux.tar.gz
# macOS:   llm-router-x86_64-apple-darwin.tar.gz

tar -xzf llm-router-*.tar.gz
./llm-router --help
```

### Opción 4: Compilar (Desarrolladores)

```bash
git clone https://github.com/XaviCode1000/Rust-LLM-Api-Router.git
cd Rust-LLM-Api-Router
cargo build --release
./target/release/llm-router --help
```

---

## 2. Configuración Guía (No Técnico)

Si no sabés usar la terminal — todo es guiada:

```bash
# Agregá proveedor de forma guiada
./llm-router provider add --interactive

# Agregá tu cuenta/API key de forma segura
./llm-router account add --interactive

# Ver qué tenés configurado
./llm-router provider list
./llm-router account list
```

El programa te pregunta cada dato. Solo respondé.

---

## 2b. Configuración Manual (Técnico)

Si sabés lo que hacés, podés configurar todo desde línea de comandos:

```bash
# Agregar proveedor-directo
./llm-router provider add --id groq --name "Groq" --url "https://api.groq.com/openai/v1"

# Agregar cuenta/API key
./llm-router account add --id mi-key --provider groq --api-key $GROQ_API_KEY

# Ver el estado
./llm-router status
```

Para configuración avanzada (vault, cascading, failover), ver [docs/](docs/)

---

## 3. Usar

```bash
# Tu app usa este endpoint
http://localhost:8080/v1/chat/completions
```

---

## Ejemplo Completo

```bash
# 1. Iniciar servidor
docker run -d -p 8080:8080 ghcr.io/xavicode1000/rust-llm-api-router:latest

# 2. Agregar proveedor (modo interactivo)
./llm-router provider add --interactive
# Te pregunta: Provider ID? → groq
# Te pregunta: Nombre? → Groq
# Te pregunta: URL? → https://api.groq.com/openai/v1

# 3. Agregar tu API key (modo interactivo)
./llm-router account add --interactive
# Te pregunta: Account ID? → mi-key
# Te pregunta: Provider? → groq
# Te pregunta: API Key? → (la escribís y queda guardada)

# 4. Listo! Usar en tu app
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer mi-key" \
  -d '{"model": "groq:llama-3.3-70b-versatile", "messages": [{"role": "user", "content": "Hola"}]}'
```

---

## Proveedores

**Gratuitos:** Zhipu AI, GitHub Models, Kluster AI

**Pagos:** OpenAI, Anthropic, Groq, Mistral, Ollama, DeepSeek, +25 más

---

## Problemas?

| Problema | Solución |
|---------|----------|
| Puerto ocupado | Cambiá el puerto: `docker run -p 8081:8080 ...` |
| No puedo acceder | Verificá que Docker esté corriendo |
| Necesitás ayuda | Ir a [docs/](docs/) |

---

## Documentación

| Para Qué | Ir A |
|----------|-------|
| Referencia CLI | [docs/cli.md](docs/cli.md) |
| API Endpoint | [docs/api.md](docs/api.md) |
| Arquitectura | [docs/architecture.md](docs/architecture.md) |
| Cascading/Failover | [docs/routing.md](docs/routing.md) |
| Deploy producción | [docs/deployment.md](docs/deployment.md) |
| Desarrollo | [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) |

---

MIT — usalo como quieras.
