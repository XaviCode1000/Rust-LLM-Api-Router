---
description: Start new SDD change — initiates full DAG workflow for a feature
---

# /sdd-new

Start a new Spec-Driven Development change for a feature.

---

## Usage

```
/sdd-new <feature-name>
```

Example:
```
/sdd-new add-csv-export-endpoint
```

---

## Process

### Step 1: Announce

```
Using sdd-new to start change for: <feature-name>
```

### Step 2: Delegate to @sdd-orchestrator

```
Delegating to @sdd-orchestrator:
 - Project: ${project_name}
 - Directory: ${current_directory}
 - Task: Start SDD change for <feature-name>
```

### Step 3: SDD DAG Execution

The orchestrator will execute:

```
1. sdd-explore   → exploration.md
2. sdd-propose   → proposal.md
3. sdd-spec      → spec.md
4. sdd-design    → design.md
5. sdd-tasks     → tasks.md
6. sdd-apply     → Code implementation
7. sdd-verify    → verification.md
8. sdd-archive   → Change closed
```

---

## Output

Change artifacts in `.qwen/artifacts/<feature>/`:
- `exploration.md`
- `proposal.md`
- `spec.md`
- `design.md`
- `tasks.md`
- `verification.md`

---

## Fast-Forward Mode

For simple changes, use `/sdd-ff`:

```
/sdd-ff <feature>  # Skips full DAG, goes straight to tasks
```

---

## Related Commands

- `/sdd-init` — Initialize project
- `/sdd-ff` — Fast-forward mode
- `/sdd-apply` — Implement tasks
- `/sdd-verify` — Verify implementation
