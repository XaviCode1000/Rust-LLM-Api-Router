---
name: sdd-explore
description: >
  SDD Phase 1: Explore the codebase to understand current architecture,
  identify risks, and gather context before proposing changes. Use at the
  start of any new feature or modification. Invoke with /skills sdd-explore.
license: MIT
metadata:
  version: "1.0.0"
  phase: "1/8"
  delegates_to:
    - rust-researcher
    - rust-project
---

# SDD-EXPLORE — Phase 1: Exploration

> Understand before changing. Explore the codebase to gather context.

---

## Purpose

Before proposing any change, you MUST understand:
- Current architecture and patterns
- Existing code conventions
- Potential risks and dependencies
- Related modules and files

---

## Process

### Step 1: Load Context

```
Using sdd-explore to understand codebase before proposing changes.
```

### Step 2: Delegate to @rust-researcher

For comprehensive exploration:

```
Delegating to @rust-researcher:
 - Project: ${project_name}
 - Directory: ${current_directory}
 - Task: Explore codebase structure and identify existing patterns for [feature]
```

### Step 3: Analyze Structure

Use `glob`, `grep`, `read_file` to:

1. **Map directory structure**
   ```
   src/
   ├── domain/
   ├── use_cases/
   ├── adapters/
   └── infrastructure/
   ```

2. **Identify patterns**
   - Error handling approach
   - Module organization
   - Naming conventions

3. **Find related code**
   - Similar features
   - Shared utilities
   - Dependencies

### Step 4: Document Findings

Create `exploration.md`:

```markdown
# Exploration Results — [feature]

## Current Architecture

[Describe existing structure]

## Relevant Files

- `src/user/` — User management
- `src/auth/` — Authentication

## Existing Patterns

- Error handling: `thiserror` + `anyhow`
- Module structure: by feature
- Testing: `#[cfg(test)]` modules

## Identified Risks

- [Risk 1]
- [Risk 2]

## Recommendations

- Follow existing [pattern]
- Extend [module] for new feature
```

---

## Output Artifacts

| Artifact | Location | Purpose |
|----------|----------|---------|
| `exploration.md` | `.qwen/artifacts/` or Engram | Exploration results |

---

## When to Use

- Starting a new feature
- Before `sdd-propose`
- When context is unclear
- After long absence from project

---

## Delegation

**Delegates to:**
- `@rust-researcher` — For comprehensive research
- `@rust-project` — For structure analysis

**Next phase:** `sdd-propose`

---

## Related Skills

- **searching-external-documentation** — For external research
- **rust-skills** — For code pattern analysis
- **obsidian** — For local documentation search
