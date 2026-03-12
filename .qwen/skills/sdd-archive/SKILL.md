---
name: sdd-archive
description: 'SDD Phase 8: Archive completed change. Merge delta specs, close change, save session summary.'
license: MIT
metadata:
  version: "1.0.0"
  phase: "8/8"
---

# SDD-ARCHIVE — Phase 8: Archival

> Close the change. Merge delta specs, save learnings, archive artifacts.

---

## Purpose

Formally close the change by archiving artifacts and saving session learnings.

---

## Process

### Step 1: Load Context

```
Using sdd-archive to close completed change.
```

### Step 2: Merge Delta Specs

If using file-based artifacts:

```markdown
# Changelog — [feature]

## [date]

### ADDED
- [From spec.md]

### MODIFIED
- [From spec.md]

### REMOVED
- [From spec.md]
```

### Step 3: Save Session Summary (Engram)

```
Using mem_session_summary:

## Goal
[What we were building]

## Instructions
[User preferences discovered]

## Discoveries
- [Technical finding 1]
- [Technical finding 2]

## Accomplished
- ✅ [Completed item 1]
- ✅ [Completed item 2]

## Next Steps
- [What remains]

## Relevant Files
- `src/[file]` — [description]
```

### Step 4: Save Key Learnings

```
## Key Learnings:

1. [Learning 1]
2. [Learning 2]
```

### Step 5: Archive Artifacts

Move artifacts to archive:

```
.qwen/artifacts/
├── [feature]/
│   ├── exploration.md
│   ├── proposal.md
│   ├── spec.md
│   ├── design.md
│   ├── tasks.md
│   └── verification.md
```

### Step 6: Mark Change Complete

Update change tracking:

```markdown
## Closed Changes

| Feature | Date | Status |
|---------|------|--------|
| [feature] | [date] | ✅ Archived |
```

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| Archived artifacts | `.qwen/artifacts/[feature]/` |
| Session summary | Engram memory |
| Changelog entry | `CHANGELOG.md` or equivalent |

---

## Delegation

**Delegates to:**
- None (final phase)

**Next phase:** Change complete. Ready for `/sdd-new` for next feature.

---

## Related Skills

- **sdd-verify** — Previous phase
- **Engram protocol** — For memory persistence
