---
name: sdd-propose
description: 'SDD Phase 2: Create proposal.md with intent, scope, and rollback plan. Use after exploration.'
license: MIT
metadata:
  version: "1.0.0"
  phase: "2/8"
---

# SDD-PROPOSE — Phase 2: Proposal

> Define WHAT to build and WHY before designing HOW.

---

## Purpose

Create a clear proposal that stakeholders can review and approve before any design work begins.

---

## Process

### Step 1: Load Context

```
Using sdd-propose to create proposal after exploration.
```

### Step 2: Create proposal.md

```markdown
# Proposal — [feature]

## Intent

[One sentence: what problem are we solving?]

## Scope

### In Scope
- [Feature 1]
- [Feature 2]

### Out of Scope
- [Explicitly excluded]

## Success Criteria

- [ ] Criterion 1
- [ ] Criterion 2

## Rollback Plan

[How to undo this change if needed]

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| [Risk] | [High/Med/Low] | [Mitigation] |
```

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| `proposal.md` | `.qwen/artifacts/` or Engram |

---

## Delegation

**Delegates to:**
- `@rust-researcher` — For risk analysis

**Next phase:** `sdd-spec`

---

## Related Skills

- **sdd-explore** — Previous phase
- **sdd-spec** — Next phase
