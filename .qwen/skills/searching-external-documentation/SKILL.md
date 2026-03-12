---
name: searching-external-documentation
description: >
  Search and extract up-to-date documentation, API references, security advisories,
  academic papers, and blog posts (2025-2026). Use when needing current information
  about Rust crates, frameworks, or best practices. Invoke with /skills searching-external-documentation.
license: MIT
metadata:
  version: "1.0.0"
  tools_required:
    - web_search
    - web_fetch
    - context7_*
    - mcp__jina__*
    - mcp__exa__*
---

# Searching External Documentation

> Skill para búsqueda y extracción de documentación actualizada (2025-2026).

---

## When to Apply

Use this skill when:
- Need up-to-date API documentation (crates, frameworks)
- Searching for code examples from 2025-2026
- Investigating security advisories
- Looking for architecture patterns
- Need academic papers (arXiv, SSRN)
- Searching blog posts or tutorials
- Finding crate compatibility information

**ALL sub-agents should delegate research tasks to @rust-researcher, who uses this skill.**

---

## Tools Available

### 1. Context7 (API Documentation)

```
Using context7_* to search for crate documentation.
```

**Use for:**
- Official API docs
- Function signatures
- Type definitions
- Usage examples

### 2. Jina AI Tools

| Tool | Purpose |
|------|---------|
| `mcp__jina__search_web` | General web search |
| `mcp__jina__read_url` | Extract content from URLs |
| `mcp__jina__search_arxiv` | Academic papers |
| `mcp__jina__search_ssrn` | Social sciences papers |
| `mcp__jina__search_jina_blog` | Blog posts (Rust, AI) |
| `mcp__jina__search_images` | Diagrams, charts |
| `mcp__jina__search_bibtex` | BibTeX citations |

### 3. EXA Research Tools

| Tool | Purpose |
|------|---------|
| `mcp__exa__web_search_exa` | Web search with clean content |
| `mcp__exa__deep_researcher_*` | Deep research reports (15s-2min) |
| `mcp__exa__get_code_context_exa` | Code from GitHub, Stack Overflow |
| `mcp__exa__company_research_exa` | Company information |

---

## Search Protocol

### For Crate Documentation

```
1. context7 for official API docs
2. web_search for 2025-2026 tutorials
3. jina_blog for official blog posts
4. exa_code_context for GitHub examples
```

### For Security Advisories

```
1. web_search: "rust security advisory [crate] 2025..2026"
2. jina_search_arxiv: academic papers on vulnerabilities
3. exa_deep_researcher: comprehensive security report
```

### For Architecture Patterns

```
1. jina_search_web: "Rust clean architecture 2025"
2. exa_code_context: "[framework] architecture examples"
3. web_fetch: extract patterns from repositories
```

---

## Output Format

Return structured results:

```markdown
## Research Results: [topic]

### Official Documentation

| Source | URL | Date | Relevance |
|--------|-----|------|-----------|
| [Name] | [URL] | [Date] | HIGH/MED/LOW |

### Key Findings

1. **Finding 1** — Description with excerpt
2. **Finding 2** — Description with excerpt

### Code Examples

```rust
// Example from [source]
// URL: ...
```

### Recommendations

- Use [pattern] because...
- Avoid [anti-pattern] because...

### References

- [Title](url) — Date
```

---

## Examples

### Example 1: Axum Middleware

```
User: "Find Axum 0.8 middleware examples"

Using searching-external-documentation to find up-to-date Axum middleware patterns.

1. context7: Search Axum API docs
2. web_search: "Axum 0.8 middleware tutorial 2025"
3. jina_blog: Search official Tokio blog
4. exa_code_context: "axum middleware examples"

Results:
- Official docs: https://docs.rs/axum/0.8/...
- Tutorial: https://... (2025-11)
- GitHub examples: https://github.com/...
```

### Example 2: Security Advisory

```
User: "Any security issues with tokio 1.x?"

Using searching-external-documentation to find security advisories.

1. web_search: "tokio security advisory 2025..2026"
2. jina_search_arxiv: "tokio vulnerability"
3. exa_deep_researcher: "tokio security history"

Results:
- RustSec advisories: ...
- CVE reports: ...
- Mitigation strategies: ...
```

---

## Best Practices

### 1. Verify Dates

Always check publication dates. Prefer 2025-2026 sources.

### 2. Cross-Reference

Use multiple sources to verify information.

### 3. Extract Clean Content

Use `web_fetch` or `jina_read_url` for clean markdown extraction.

### 4. Cite Sources

Always include URLs and dates in results.

### 5. Use Deep Research for Complex Topics

For comprehensive research, use `exa_deep_researcher` (15s-2min).

---

## Integration with rust-researcher

This skill is used by **@rust-researcher** sub-agent.

When any sub-agent needs research:

```
@rust-api: "Need Axum docs"
  → Delegates to @rust-researcher
@rust-researcher:
  → Uses searching-external-documentation skill
  → Returns structured results
```

---

## Hardware Awareness (Haswell/HDD/8GB)

```fish
# Web searches are I/O light
# No ionice needed

# Deep research can take 2min
# Run in background if long-running
```

---

## Related Skills

- **rust-skills** — 179 Rust best practices
- **sdd-explore** — SDD exploration phase
- **obsidian** — Search local Obsidian vault
