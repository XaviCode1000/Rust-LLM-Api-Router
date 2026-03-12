---
description: Archive completed SDD change — merge delta specs, save session summary, close change
---

# /sdd-archive

Archive a completed SDD change and save learnings.

---

## Usage

```
/sdd-archive [feature]
```

Example:
```
/sdd-archive add-csv-export
```

---

## Process

### Step 1: Announce

```
Using sdd-archive to close completed change: [feature]
```

### Step 2: Load Skills

```
Using sdd-archive for change archival.
```

### Step 3: Verify Completion

Check that:
- [ ] All tasks completed
- [ ] Tests passing
- [ ] Code review approved
- [ ] verification.md exists

### Step 4: Merge Delta Specs

Update changelog with changes from `spec.md`:

```markdown
# Changelog

## [date] — [feature]

### ADDED
- [From spec.md]

### MODIFIED
- [From spec.md]

### REMOVED
- [From spec.md]
```

### Step 5: Save Session Summary (Engram)

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

### Step 6: Save Key Learnings

```
## Key Learnings:

1. [Learning 1]
2. [Learning 2]
```

### Step 7: Archive Artifacts

Move to archive:
```
.qwen/artifacts/[feature]/
├── exploration.md
├── proposal.md
├── spec.md
├── design.md
├── tasks.md
└── verification.md
```

### Step 8: Mark Change Complete

Update change tracking:
```markdown
## Closed Changes

| Feature | Date | Status |
|---------|------|--------|
| [feature] | [date] | ✅ Archived |
```

---

## Output

- Archived artifacts in `.qwen/artifacts/[feature]/`
- Session summary in Engram memory
- Changelog entry updated

---

## Next Steps

Change is complete. Ready for next feature:
```
/sdd-new <next-feature>
```

---

## Related Commands

- `/sdd-verify` — Previous step
- `/sdd-new` — Start next change
- `/summary` — Generate project summary
