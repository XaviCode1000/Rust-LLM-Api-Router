---
description: Initialize SDD context for new project — creates artifact structure and baseline documentation
---

# /sdd-init

Initialize Spec-Driven Development context for the current project.

---

## Purpose

Set up the SDD workflow infrastructure:
- Create artifact directories
- Initialize baseline documentation
- Load project context into Engram memory

---

## Process

### Step 1: Announce

```
Using sdd-init to initialize SDD context.
```

### Step 2: Create Directory Structure

```bash
mkdir -p .qwen/artifacts
mkdir -p docs/architecture
```

### Step 3: Save Project Context (Engram)

```
Using mem_save:
  title: "Project context initialized"
  type: "config"
  content: |
    **What**: Initialized SDD context for project
    **Where**: ${current_directory}
    **Project**: ${project_name}
```

### Step 4: Create Baseline Files

Create `docs/architecture/overview.md`:

```markdown
# Architecture Overview — ${project_name}

## Current Structure

[To be filled by sdd-explore]

## Key Decisions

[To be documented]
```

---

## Output

```
.qwen/
├── artifacts/          # Created
└── settings.json

docs/
└── architecture/
    └── overview.md     # Created
```

---

## Next Steps

After initialization:

```
/sdd-new <feature>   # Start first feature
```

---

## Related Commands

- `/sdd-new` — Start new feature
- `/sdd-explore` — Explore codebase
