---
name: sdd-verify
description: 'SDD Phase 7: Verify implementation against spec with tests and review. Use @rust-reviewer and @rust-tester.'
license: MIT
metadata:
  version: "1.0.0"
  phase: "7/8"
  delegates_to:
    - rust-reviewer
    - rust-tester
---

# SDD-VERIFY — Phase 7: Verification

> Verify implementation against spec with tests and code review.

---

## Purpose

Ensure the implementation meets all spec requirements and follows rust-skills best practices.

---

## Process

### Step 1: Load Context

```
Using sdd-verify to verify implementation against spec.
```

### Step 2: Delegate to @rust-tester

```
Delegating to @rust-tester:
 - Project: ${project_name}
 - Task: Run all tests and verify coverage for [feature]
```

### Step 3: Delegate to @rust-reviewer

```
Delegating to @rust-reviewer:
 - Project: ${project_name}
 - Task: Code review for [feature] against rust-skills 179 rules
```

### Step 4: Create verification.md

```markdown
# Verification — [feature]

## Test Results

| Suite | Status | Coverage |
|-------|--------|----------|
| Unit tests | ✅ Pass (34/34) | 85% |
| Integration tests | ✅ Pass (12/12) | 90% |
| Benchmarks | ✅ Within limits | - |

## Spec Compliance Matrix

| Requirement | Status | Evidence |
|-------------|--------|----------|
| R1 (MUST) | ✅ | test_scenario_1 |
| R2 (SHOULD) | ✅ | test_scenario_2 |
| R3 (MAY) | ✅ | Implemented |

## Code Review Results

### Critical Issues

None ✅

### Important Improvements

- [ ] [Issue 1]

### Minor Suggestions

- [ ] [Suggestion 1]

## rust-skills Compliance

| Category | Status |
|----------|--------|
| Ownership | ✅ |
| Error Handling | ✅ |
| Async/Await | ✅ |
| Memory | ✅ |

## Verification Commands

```fish
cargo test          # All tests pass
cargo clippy        # No warnings
cargo fmt --check   # Formatted correctly
```
```

---

## Output Artifacts

| Artifact | Location |
|----------|----------|
| `verification.md` | `.qwen/artifacts/` or Engram |
| Test results | Console output |

---

## Delegation

**Delegates to:**
- `@rust-tester` — For test execution
- `@rust-reviewer` — For code review

**Next phase:** `sdd-archive`

---

## Related Skills

- **sdd-apply** — Previous phase
- **sdd-archive** — Next phase
- **rust-skills** — For review criteria
