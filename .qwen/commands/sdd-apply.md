---
description: Implement SDD tasks — writes code following specs and design with rust-skills compliance
---

# /sdd-apply

Implement tasks from `tasks.md` following spec and design documents.

---

## Usage

```
/sdd-apply [task-filter]
```

Example:
```
/sdd-apply          # Apply all incomplete tasks
/sdd-apply phase-1  # Apply only phase 1 tasks
```

---

## Process

### Step 1: Announce

```
Using sdd-apply to implement tasks.
Using rust-skills for idiomatic Rust patterns.
```

### Step 2: Load Skills (MANDATORY)

```
Using rust-skills for idiomatic Rust patterns.
```

### Step 3: Read Dependencies

- `tasks.md` — Task list
- `spec.md` — Acceptance criteria
- `design.md` — Architecture decisions

### Step 4: Delegate to @rust-api

```
Delegating to @rust-api:
 - Project: ${project_name}
 - Directory: ${current_directory}
 - Task: Implement [task from tasks.md]
 - Spec: [relevant scenarios]
 - Design: [relevant decisions]
 - Reminder: Load rust-skills before writing code
```

### Step 5: TDD Workflow (if enabled)

```
1. @rust-tester writes failing test
2. @rust-api implements to pass test
3. Refactor with rust-skills
```

### Step 6: Update tasks.md

Mark completed tasks:
```markdown
- [x] 1.1 [Task] ✅
```

---

## Output

- Updated source code in `src/`
- Updated `tasks.md` with completed items
- Session summary in Engram

---

## Verification

After apply:

```fish
cargo build    # Should compile
cargo test     # Tests should pass
cargo clippy   # Should be clean
```

---

## Next Steps

```
/sdd-verify  # Verify implementation against spec
```

---

## Related Commands

- `/sdd-tasks` — View task list
- `/sdd-verify` — Verify implementation
- `/sdd-new` — Start new change
