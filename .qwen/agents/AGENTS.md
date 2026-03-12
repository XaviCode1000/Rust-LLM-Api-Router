# Rust LLM API Router — Sub-Agents

This project uses **6 specialized sub-agents** for different aspects of development.

---

## Available Sub-Agents

### sdd-orchestrator
**Role:** SDD Workflow Coordinator | **Mode:** Subagent

Coordinates the Spec-Driven Development (SDD) workflow. Delegates all real work to specialized sub-agents.

**When to use:**
- Starting a new feature/change
- Managing SDD change lifecycle
- Coordinating multiple sub-agents

**Delegates to:**
- `@rust-researcher` for exploration
- `@rust-api` for implementation
- `@rust-reviewer` for verification
- `@rust-tester` for testing

---

### rust-api
**Role:** API REST Specialist | **Mode:** Subagent

API REST implementation with Axum/Actix-web:
- RESTful endpoint design
- Handlers and routing
- Validation and error handling
- Middleware

**When to use:**
- Creating new endpoints
- Implementing request handlers
- Adding validation logic
- Designing API responses

**Delegates to:**
- `@rust-researcher` for up-to-date docs
- `@rust-project` for module structure

---

### rust-project
**Role:** Project Structure Specialist | **Mode:** Subagent

Project structure and organization:
- Workspace organization
- Module structure
- Visibility (pub, pub(crate))
- Re-exports and prelude patterns

**When to use:**
- Creating new modules
- Organizing crate structure
- Setting visibility rules
- Configuring workspaces

---

### rust-reviewer
**Role:** Code Reviewer | **Mode:** Subagent

Code review specialist:
- Best practices (179 rust-skills rules)
- Security review
- Performance analysis
- Anti-pattern detection

**When to use:**
- Before merging code
- Security audit
- Performance optimization
- Code quality check

**Uses:** rust-skills rules for review criteria

---

### rust-tester
**Role:** Testing Specialist | **Mode:** Subagent

Testing and TDD:
- Unit tests
- Integration tests
- TDD workflow
- Property-based testing (proptest)
- Benchmarking (criterion)

**When to use:**
- Writing new tests
- TDD implementation
- Adding test coverage
- Performance benchmarks

---

### rust-researcher 🆕
**Role:** INVESTIGADOR | **Mode:** Subagent

Research and documentation specialist. **ALL sub-agents can delegate to this agent.**

**Tools:**
- `web_search` — General web search
- `web_fetch` — Fetch and extract web content
- `context7_*` — API documentation
- `mcp__jina__*` — Jina AI tools (search, read, images)
- `mcp__exa__*` — EXA research tools

**When to use (by ANY sub-agent):**
- Need up-to-date documentation (2025-2026)
- Searching for API examples
- Investigating security advisories
- Looking for architecture patterns
- Need academic papers (arXiv, SSRN)
- Searching blog posts/tutorials
- Finding crate compatibility info

**Example delegation:**
```
@rust-api: "Necesito docs actualizada de Axum 0.8"
  → Delegates to @rust-researcher

@rust-researcher:
  1. context7 for API docs
  2. web_search for 2025-2026 tutorials  
  3. jina_blog for official posts
  4. Returns: URLs + excerpts + dates
```

---

## SDD Workflow Integration

The SDD workflow delegates to specialized agents:

| SDD Phase | Delegates To |
|-----------|--------------|
| sdd-explore | @rust-researcher, @rust-project |
| sdd-propose | @rust-researcher |
| sdd-spec | @rust-researcher (for requirements) |
| sdd-design | @rust-api, @rust-project |
| sdd-tasks | @rust-api, @rust-tester |
| sdd-apply | @rust-api + rust-skills |
| sdd-verify | @rust-reviewer + @rust-tester |
| sdd-archive | — (closes change) |

---

## Using Sub-Agents

### Manual invocation
```
@rust-api design an endpoint for user auth
@rust-researcher find docs for Axum 0.8 middleware
@rust-reviewer review this module
@rust-tester write tests for auth
@rust-project organize this module structure
```

### Via SDD workflow
```
/sdd-new add-api-key-auth
→ @sdd-orchestrator coordinates
→ @rust-researcher explores codebase
→ @rust-api designs endpoints
→ @rust-api implements with rust-skills
→ @rust-reviewer + @rust-tester verify
```

---

## Skills

- **rust-skills**: 179 Rust best practices (invoke with `/skills rust-skills`)
- **searching-external-documentation**: Research skill (invoke with `/skills searching-external-documentation`)
- **SDD skills**: Spec-Driven Development workflow (8 skills)

---

## Context Injection Protocol

When delegating to a sub-agent, provide:

```
- Project: ${project_name}
- Directory: ${current_directory}
- Task: ${task_description}
```

Example:
```
Delegating to rust-researcher:
 - Project: rust-llm-api-router
 - Directory: /home/gazadev/Dev/my_apps/Rust-LLM-Api-Router
 - Task: Find up-to-date Axum 0.8 middleware examples
```
