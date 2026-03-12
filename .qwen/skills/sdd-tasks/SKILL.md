---
name: sdd-tasks
description: 'SDD Phase 5: Create numbered, phased task checklist in tasks.md. Use after design approval.'
license: MIT
metadata:
  version: "1.0.0"
  phase: "5/8"
---

# SDD-TASKS — Phase 5: Task Planning

> Break design into numbered, phased tasks with clear completion criteria.

---

## Purpose

Create an actionable task list that can be tracked during implementation.

---

## Process

### Step 1: Load Context

```
Using sdd-tasks to create implementation task list.
```

### Step 2: Create tasks.md

```markdown
# Tasks — [feature]

## Phase 1: Foundation

- [ ] 1.1 [Task description]
  - Files: `src/[file]`
  - Done when: [criteria]

- [ ] 1.2 [Task description]
  - Files: `src/[file]`
  - Done when: [criteria]

## Phase 2: Implementation

- [ ] 2.1 [Task description]
  - Files: `src/[file]`
  - Done when: [criteria]

## Phase 3: Testing

- [ ] 3.1 Write unit tests
  - Files: `src/[file]` (tests module)
  - Done when: All tests pass

- [ ] 3.2 Write integration tests
  - Files: `tests/[file]`
  - Done when: All tests pass

## Phase 4: Verification

- [ ] 4.1 Code review
  - Reviewer: @rust-reviewer
  - Done when: No critical issues

- [ ] 4.2 Performance check
  - Done when: Benchmarks within limits
```

---

## Task Format

Each task MUST have:
- **Number** — Unique identifier (e.g., 1.1, 2.3)
- **Description** — Clear action verb + object
- **Files** — Affected paths
- **Done when** — Completion criteria

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| `tasks.md` | `.qwen/artifacts/` or Engram |

---

## Delegation

**Delegates to:**
- `@rust-api` — For implementation tasks
- `@rust-tester` — For testing tasks

**Next phase:** `sdd-apply`

---

## Related Skills

- **sdd-design** — Previous phase
- **sdd-apply** — Next phase
