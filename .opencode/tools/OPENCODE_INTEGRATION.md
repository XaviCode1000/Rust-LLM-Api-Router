# Usar llm-router con OpenCode

Este documento explica cómo integrar los modelos del llm-router con OpenCode.

## Opción 1: Usar como Proxy OpenAI-Compatibile (RECOMENDADO)

### Paso 1: Iniciar el servidor llm-router

```bash
./target/release/llm-router --port 8080
```

### Paso 2: Configurar OpenCode

Editá `.opencode/opencode.json`:

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

### Modelos Disponibles

Con esta configuración podés usar cualquier modelo de los providers configurados:

```json
{
  "model": "groq:llama-3.3-70b-versatile"
}
```

**Groq Models:**
- `groq:llama-3.3-70b-versatile`
- `groq:llama-3.1-8b-instant`
- `groq:compound`
- `groq:compound-mini`

**OpenAI Models:**
- `openai:gpt-4o`
- `openai:gpt-4o-mini`
- `openai:gpt-4-turbo`

**OpenRouter Models:**
- `openrouter:anthropic/claude-3-opus`
- `openrouter:meta-llama/llama-3-70b-instruct`
- `openrouter:google/gemma-7b-it`

### Modelos Gratuitos de OpenRouter

```json
{
  "model": "openrouter:openrouter/free"
}
```

---

## Opción 2: Usar las Herramientas (sin servidor)

Si no querés iniciar el servidor, podés usar las herramientas CLI:

```bash
@llm-router_models provider="groq"
@llm_router_chat message="Hello!"
```

---

## Archivo de Configuración Completo

```json
{
  "$schema": "https://opencode.ai/config.json",

  "model": "groq:llama-3.3-70b-versatile",
  "provider": {
    "openai": {
      "options": {
        "baseURL": "http://localhost:8080/v1",
        "apiKey": "demo-key"
      }
    }
  },

  "agents": {
    "rust-expert": {
      "description": "Rust programming expert",
      "model": "groq:llama-3.3-70b-versatile",
      "tools": {
        "write": true,
        "edit": true,
        "bash": true
      }
    },
    "fast-review": {
      "description": "Quick code review with fast model",
      "model": "groq:llama-3.1-8b-instant",
      "tools": {
        "write": false,
        "edit": false,
        "bash": false
      }
    }
  }
}
```

---

## Requisitos

1. **Servidor corriendo**: `llm-router --port 8080`
2. **Al menos un provider con account configurado**:
   ```bash
   llm-router auth login --provider groq
   ```
3. **Verificar models disponibles**:
   ```bash
   llm-router provider models --provider groq
   ```

---

## Solución de Problemas

### Error: "Invalid model"

Asegurate de que el modelo exista:
```bash
llm-router provider models --provider groq
```

### Error: "Connection refused"

Iniciá el servidor:
```bash
./target/release/llm-router --port 8080
```

### Error: "No API key"

Hacé login:
```bash
llm-router auth login --provider groq
```
