---
name: sdd-design
description: 'SDD Phase 4: Create design.md with architecture decisions and rationale. Use after spec approval.'
license: MIT
metadata:
  version: "1.0.0"
  phase: "4/8"
---

# SDD-DESIGN — Phase 4: Design

> Define HOW to build it. Architecture decisions with rationale.

---

## Purpose

Create design documentation that explains the architecture, patterns, and trade-offs.

---

## Process

### Step 1: Load Context

```
Using sdd-design to create architecture design.
```

### Step 2: Delegate to @rust-api / @rust-project

```
Delegating to @rust-api:
 - Project: ${project_name}
 - Task: Design architecture for [feature] following spec requirements
```

### Step 3: Create design.md

```markdown
# Design — [feature]

## Architecture Overview

[Diagram or description of components]

## Components

| Component | Responsibility | Dependencies |
|-----------|----------------|--------------|
| [Name] | [What it does] | [Dependencies] |

## Module Structure

```
src/
├── [module]/
│   ├── mod.rs
│   └── ...
```

## Key Decisions

| Decision | Rationale | Alternatives Considered |
|----------|-----------|------------------------|
| [Choice] | [Why] | [Option A, Option B] |

## Data Flow

[How data moves through the system]

## Error Handling

[Strategy for errors]

## Testing Strategy

[Approach to testing]
```

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| `design.md` | `.qwen/artifacts/` or Engram |

---

## Delegation

**Delegates to:**
- `@rust-api` — For API design
- `@rust-project` — For module structure

**Next phase:** `sdd-tasks`

---

## Related Skills

- **sdd-spec** — Previous phase
- **sdd-tasks** — Next phase
- **rust-skills** — For design patterns
