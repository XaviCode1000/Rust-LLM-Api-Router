# LLM Router Tool for OpenCode

This directory contains custom tools for interacting with `llm-router` CLI from OpenCode agents.

## Herramientas Disponibles

| Herramienta | Descripción |
|-------------|-------------|
| `llm-router_provider_list` | Lista todos los providers y su estado |
| `llm-router_account_list` | Lista todas las cuentas configuradas |
| `llm-router_models` | Lista modelos disponibles de un provider |
| `llm-router_auth_status` | Verifica estado de autenticación |
| `llm-router_server_status` | Verifica si el servidor está corriendo |
| `llm_router_chat` | Envia un mensaje al LLM (requiere servidor) |

## Uso

Las herramientas se cargan automáticamente desde `.opencode/tools/` cuando inicies OpenCode.

### Ejemplos en OpenCode:

```
@llm-router_provider_list

@llm-router_account_list

@llm-router_models provider="groq"

@llm-router_auth_status

@llm-router_server_status

@llm_router_chat message="Hello, how are you?"
```

## Configuración

El path al binario está hardcodeado en `llm-router.ts`:

```typescript
const LLM_ROUTER_BIN = "/home/gazadev/Dev/my_apps/Rust-LLM-Api-Router/target/release/llm-router"
```

Si movés el binario, actualizá esta ruta.

## Requisitos

- OpenCode instalado
- Bun (para ejecutar las herramientas)
- llm-router buildueado en release mode
