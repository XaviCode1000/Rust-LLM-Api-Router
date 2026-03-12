# CLI Reference

Referencia completa de la interfaz de línea de comandos.

## Uso General

```bash
llm-router [OPTIONS] [COMMAND]
```

### Opciones Globales

| Opción | Descripción | Default |
|--------|-------------|---------|
| `--host <HOST>` | Host para el servidor | `0.0.0.0` |
| `--port <PORT>` | Puerto para el servidor | `8080` |
| `--log-level <LEVEL>` | Nivel de log (trace, debug, info, warn, error) | `info` |
| `-h, --help` | Mostrar ayuda | - |

## Comandos de Proveedores

Gestión de proveedores LLM.

### `llm-router provider add`

Agregar un nuevo proveedor.

```bash
llm-router provider add \
  --id <ID> \
  --name <NOMBRE> \
  --base-url <URL> \
  [--api-key <KEY>] \
  [--disabled] \
  [--interactive]
```

**Opciones:**

| Opción | Descripción | Requerido |
|--------|-------------|-----------|
| `--id` | Identificador único del proveedor | Sí |
| `--name` | Nombre legible del proveedor | Sí |
| `--base-url` | URL base de la API | Sí |
| `--api-key` | API key (o usar --interactive) | No |
| `--disabled` | Iniciar deshabilitado | No |
| `--interactive` | Pedir API key interactivamente | No |

**Ejemplos:**

```bash
# Agregar Groq
llm-router provider add \
  --id groq \
  --name "Groq" \
  --base-url "https://api.groq.com/openai/v1"

# Agregar con API key
llm-router provider add \
  --id openrouter \
  --name "OpenRouter" \
  --base-url "https://openrouter.ai/api/v1" \
  --api-key "sk-or-v1-xxx"

# Agregar interactivo (pide API key)
llm-router provider add \
  --id mistral \
  --name "Mistral AI" \
  --base-url "https://api.mistral.ai/v1" \
  --interactive
```

---

### `llm-router provider list`

Listar todos los proveedores registrados.

```bash
llm-router provider list
```

**Salida:**

```
ID                   Name                           Base URL                                 Status
----------------------------------------------------------------------------------------------------
groq                 Groq                           https://api.groq.com/openai/v1           ✓ Enabled
openrouter           OpenRouter                     https://openrouter.ai/api/v1             ✓ Enabled
mistral              Mistral AI                     https://api.mistral.ai/v1                ✗ Disabled
```

---

### `llm-router provider enable`

Habilitar un proveedor.

```bash
llm-router provider enable --id <ID>
```

**Ejemplo:**

```bash
llm-router provider enable --id mistral
```

---

### `llm-router provider disable`

Deshabilitar un proveedor.

```bash
llm-router provider disable --id <ID>
```

**Ejemplo:**

```bash
llm-router provider disable --id mistral
```

---

### `llm-router provider remove`

Eliminar un proveedor.

```bash
llm-router provider remove --id <ID>
```

**Ejemplo:**

```bash
llm-router provider remove --id mistral
```

---

### `llm-router provider validate`

Validar credenciales de un proveedor.

```bash
llm-router provider validate --id <ID>
```

**Ejemplo:**

```bash
llm-router provider validate --id groq
```

---

## Comandos de Cuentas

Gestión de cuentas (API keys) por proveedor.

### `llm-router account add`

Agregar una nueva cuenta.

```bash
llm-router account add \
  --id <ID> \
  --provider <PROVIDER_ID> \
  --api-key <KEY> \
  [--priority <N>] \
  [--inactive] \
  [--interactive]
```

**Opciones:**

| Opción | Descripción | Default |
|--------|-------------|---------|
| `--id` | Identificador único de la cuenta | - |
| `--provider` | ID del proveedor | - |
| `--api-key` | API key (o usar --interactive) | - |
| `--priority` | Prioridad (menor = mayor prioridad) | `0` |
| `--inactive` | Iniciar inactiva | - |
| `--interactive` | Pedir API key interactivamente | - |

**Ejemplos:**

```bash
# Agregar cuenta con API key
llm-router account add \
  --id groq-1 \
  --provider groq \
  --api-key "gsk_xxx" \
  --priority 0

# Agregar interactivo
llm-router account add \
  --id groq-2 \
  --provider groq \
  --priority 1 \
  --interactive
```

---

### `llm-router account list`

Listar todas las cuentas.

```bash
llm-router account list
```

**Salida:**

```
ID                   Provider             Priority   Status   API Key
------------------------------------------------------------------------------------------
groq-1               groq                 0          ✓ Active gsk_DVyb...
groq-2               groq                 1          ✓ Active gsk_ABC...
openrouter-1         openrouter           0          ✓ Active sk-or-v1...
```

---

### `llm-router account set-priority`

Cambiar prioridad de una cuenta.

```bash
llm-router account set-priority --id <ID> --priority <N>
```

**Ejemplo:**

```bash
llm-router account set-priority --id groq-1 --priority 10
```

---

### `llm-router account remove`

Eliminar una cuenta.

```bash
llm-router account remove --id <ID>
```

**Ejemplo:**

```bash
llm-router account remove --id groq-1
```

---

### `llm-router account validate`

Validar API key de una cuenta.

```bash
llm-router account validate --id <ID>
```

**Ejemplo:**

```bash
llm-router account validate --id groq-1
```

---

## Scripts de Bootstrap

### register-providers.sh

Registra automáticamente 12+ proveedores:

```bash
./scripts/register-providers.sh
```

**Opciones:**

```bash
./scripts/register-providers.sh [--api-key <KEY>] [--interactive]
```

---

### register-accounts.sh

Registra proveedores y cuentas usando API keys de copyq:

```bash
./scripts/register-accounts.sh
```

**Requisitos:**

- Tener `copyq` instalado
- API keys en pestaña `SECRETS` de copyq

---

## Ejemplos de Uso

### Flujo Completo

```bash
# 1. Registrar proveedores
llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1"
llm-router provider add --id openrouter --name "OpenRouter" --base-url "https://openrouter.ai/api/v1"

# 2. Registrar cuentas
llm-router account add --id groq-1 --provider groq --api-key "gsk_xxx" --priority 0
llm-router account add --id or-1 --provider openrouter --api-key "sk-or-v1_xxx" --priority 0

# 3. Habilitar proveedores
llm-router provider enable --id groq
llm-router provider enable --id openrouter

# 4. Verificar
llm-router provider list
llm-router account list

# 5. Iniciar servidor
llm-router --port 8080
```

### Usar con OpenAI SDK

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="no-needed"  # La API key está en el sistema
)

response = client.chat.completions.create(
    model="groq:llama-3.1-8b-instant",
    messages=[
        {"role": "user", "content": "Hola!"}
    ]
)

print(response.choices[0].message.content)
```

---

## Solución de Problemas

### "No API key provided"

Usá `--api-key` o `--interactive`:

```bash
llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1" --api-key "gsk_xxx"
```

### "Provider not found"

Verificá el ID con `provider list`:

```bash
llm-router provider list
```

### "Invalid API key"

Validá la key con el proveedor:

```bash
llm-router account validate --id groq-1
```

### Cuentas no se rotan

Verificá que haya múltiples cuentas activas:

```bash
llm-router account list
```

Si todas tienen la misma prioridad, el sistema usa round-robin.
