---
name: sdd-apply
description: 'SDD Phase 6: Implement tasks from tasks.md following specs and design. Load rust-skills before coding.'
license: MIT
metadata:
  version: "1.0.0"
  phase: "6/8"
  delegates_to:
    - rust-api
    - rust-tester
---

# SDD-APPLY — Phase 6: Implementation

> Write code following specs, design, and rust-skills best practices.

---

## Purpose

Implement tasks from `tasks.md` while adhering to spec requirements and design decisions.

---

## Process

### Step 1: Load Context

```
Using sdd-apply to implement tasks.
Using rust-skills for idiomatic Rust patterns.
```

### Step 2: Load rust-skills (MANDATORY)

```
Using rust-skills for idiomatic Rust patterns.
```

**Critical rules to follow:**
- `err-no-unwrap-prod` — No unwrap in production
- `async-no-lock-await` — No locks across await
- `own-borrow-over-clone` — Borrow over clone
- `mem-avoid-format` — Avoid unnecessary allocations

### Step 3: Delegate to @rust-api

```
Delegating to @rust-api:
 - Project: ${project_name}
 - Directory: ${current_directory}
 - Task: Implement [task from tasks.md]
 - Spec: [relevant scenarios from spec.md]
 - Design: [relevant decisions from design.md]
 - Reminder: Load rust-skills before writing code
```

### Step 4: TDD Workflow (if enabled)

```
1. @rust-tester writes failing test
2. @rust-api implements to pass test
3. Refactor with rust-skills
4. Repeat
```

### Step 5: Update tasks.md

Mark completed tasks:

```markdown
- [x] 1.1 [Task description] ✅
```

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| Updated code | `src/` |
| Updated `tasks.md` | `.qwen/artifacts/` or Engram |

---

## Delegation

**Delegates to:**
- `@rust-api` — For code implementation
- `@rust-tester` — For TDD workflow

**Next phase:** `sdd-verify`

---

## Related Skills

- **rust-skills** — For code quality
- **sdd-tasks** — Previous phase
- **sdd-verify** — Next phase
