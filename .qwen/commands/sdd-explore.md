---
description: Explore codebase — analyze structure, patterns, and risks before proposing changes
---

# /sdd-explore

Explore the codebase to understand current architecture before making changes.

---

## Usage

```
/sdd-explore [topic]
```

Example:
```
/sdd-explore authentication-module
```

---

## Process

### Step 1: Announce

```
Using sdd-explore to explore codebase for: [topic]
Using searching-external-documentation if external research needed.
```

### Step 2: Load Skills

```
Using sdd-explore for systematic codebase exploration.
```

### Step 3: Delegate to @rust-researcher

```
Delegating to @rust-researcher:
 - Project: ${project_name}
 - Directory: ${current_directory}
 - Task: Explore codebase structure and identify patterns for [topic]
```

### Step 4: Create exploration.md

```markdown
# Exploration — [topic]

## Current Architecture

[Description]

## Relevant Files

- `src/[file]` — Description

## Existing Patterns

- Pattern 1
- Pattern 2

## Identified Risks

- Risk 1
- Risk 2

## Recommendations

- Recommendation 1
```

---

## Output

- `exploration.md` in `.qwen/artifacts/[topic]/`
- Session summary in Engram

---

## Next Steps

```
/sdd-propose  # Create proposal based on exploration
```

---

## Related Commands

- `/sdd-new` — Start full change
- `/sdd-propose` — Create proposal
- `/sdd-ff` — Fast-forward workflow
