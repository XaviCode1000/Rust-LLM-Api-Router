---
description: Verify implementation against spec — run tests, code review, spec compliance matrix
---

# /sdd-verify

Verify implementation against spec with tests and code review.

---

## Usage

```
/sdd-verify [feature]
```

Example:
```
/sdd-verify add-csv-export
```

---

## Process

### Step 1: Announce

```
Using sdd-verify to verify implementation against spec.
```

### Step 2: Load Skills

```
Using sdd-verify for systematic verification.
```

### Step 3: Delegate to @rust-tester

```
Delegating to @rust-tester:
 - Project: ${project_name}
 - Task: Run all tests and verify coverage for [feature]
```

### Step 4: Delegate to @rust-reviewer

```
Delegating to @rust-reviewer:
 - Project: ${project_name}
 - Task: Code review for [feature] against rust-skills 179 rules
```

### Step 5: Run Verification Commands

```fish
cargo test          # All tests
cargo test -- --nocapture  # With output
cargo clippy        # Linting
cargo fmt --check   # Formatting
```

### Step 6: Create verification.md

```markdown
# Verification — [feature]

## Test Results

| Suite | Status | Coverage |
|-------|--------|----------|
| Unit tests | ✅ Pass (34/34) | 85% |
| Integration tests | ✅ Pass (12/12) | 90% |

## Spec Compliance Matrix

| Requirement | Status | Evidence |
|-------------|--------|----------|
| R1 (MUST) | ✅ | test_scenario_1 |
| R2 (SHOULD) | ✅ | test_scenario_2 |

## Code Review Results

### Critical Issues
None ✅

### Important Improvements
- [ ] [Issue 1]

## rust-skills Compliance

| Category | Status |
|----------|--------|
| Ownership | ✅ |
| Error Handling | ✅ |
| Async/Await | ✅ |
```

---

## Output

- `verification.md` in `.qwen/artifacts/[feature]/`
- Test results in console
- Session summary in Engram

---

## Next Steps

If verification passes:
```
/sdd-archive  # Close change
```

If verification fails:
```
/sdd-apply  # Fix issues and re-apply
```

---

## Related Commands

- `/sdd-apply` — Implement tasks
- `/sdd-archive` — Archive completed change
- `/sdd-new` — Start new change
