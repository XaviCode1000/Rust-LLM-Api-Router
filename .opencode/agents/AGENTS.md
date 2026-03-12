# Rust LLM API Router - Custom Agents

This project uses **Agent Teams Lite** (SDD - Spec-Driven Development) with specialized Rust agents.

---

## Available Agents

### sdd-orchestrator
**Mode:** Agent | **Temperature:** 0.2

Coordinates the SDD workflow. Delegates to specialized agents for each phase.

### rust-api
**Mode:** Subagent | **Temperature:** 0.2

API REST specialist with Actix-web:
- RESTful endpoint design
- Handlers and routing
- Validation and error handling
- Middleware

### rust-project
**Mode:** Subagent | **Temperature:** 0.2

Project structure specialist:
- Workspace organization
- Module structure
- Visibility (pub, pub(crate))

### rust-reviewer
**Mode:** Subagent | **Temperature:** 0.1

Code reviewer:
- Best practices
- Security review
- Performance analysis

### rust-tester
**Mode:** Subagent | **Temperature:** 0.2

Testing specialist:
- Unit tests
- Integration tests
- TDD workflow

---

## Agent Teams Lite Integration

The SDD workflow delegates to specialized agents:

| SDD Phase | Delegates To |
|-----------|--------------|
| sdd-explore | @rust-project |
| sdd-design | @rust-api |
| sdd-apply | @rust-api + rust-skills |
| sdd-verify | @rust-reviewer + @rust-tester |

---

## SDD Commands

```bash
/sdd-init           # Initialize project context
/sdd-new <feature>  # Start new change
/sdd-explore <topic> # Explore codebase
/sdd-apply          # Implement tasks
/sdd-verify         # Verify against specs
/sdd-archive        # Archive completed change
/sdd-ff <feature>  # Fast-forward: proposal → spec → design → tasks
```

---

## Using Agents

### Manual invocation
```
@rust-api design an endpoint for user auth
@rust-reviewer review this module
@rust-tester write tests for auth
```

### Via SDD workflow
```
/sdd-new add-api-key-auth
→ @rust-project explores codebase
→ @rust-api designs endpoints
→ @rust-api implements with rust-skills
→ @rust-reviewer + @rust-tester verify
```

---

## Skills

- **rust-skills**: 179 Rust best practices (invoke with `/rust-skills`)
- **SDD skills**: Spec-Driven Development workflow
