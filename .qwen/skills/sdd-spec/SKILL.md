---
name: sdd-spec
description: 'SDD Phase 3: Write delta spec.md with Given/When/Then scenarios using RFC 2119 keywords (MUST/SHALL/SHOULD/MAY).'
license: MIT
metadata:
  version: "1.0.0"
  phase: "3/8"
---

# SDD-SPEC — Phase 3: Specification

> Define requirements with RFC 2119 keywords and Given/When/Then scenarios.

---

## Purpose

Create a delta spec that describes WHAT changes (ADDED/MODIFIED/REMOVED) with testable scenarios.

---

## Process

### Step 1: Load Context

```
Using sdd-spec to write delta specification.
```

### Step 2: Create spec.md

```markdown
# Spec — [feature]

## Delta

### ADDED
- [New functionality]

### MODIFIED
- [Changed functionality]

### REMOVED
- [Deleted functionality]

## Requirements (RFC 2119)

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | The system MUST [requirement] | SHALL |
| R2 | The system SHOULD [recommendation] | SHOULD |
| R3 | The system MAY [optional] | MAY |

## Acceptance Criteria (Given/When/Then)

### Scenario 1: [Name]

**Given** [context]
**When** [action]
**Then** [expected result]

### Scenario 2: [Name]

**Given** [context]
**When** [action]
**Then** [expected result]
```

---

## RFC 2119 Keywords

| Keyword | Meaning |
|---------|---------|
| MUST/SHALL | Required (non-negotiable) |
| SHOULD | Recommended (valid exceptions exist) |
| MAY | Optional (implementer's choice) |
| MUST NOT | Prohibited |

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| `spec.md` | `.qwen/artifacts/` or Engram |

---

## Delegation

**Delegates to:**
- `@rust-researcher` — For requirements research

**Next phase:** `sdd-design`

---

## Related Skills

- **sdd-propose** — Previous phase
- **sdd-design** — Next phase
