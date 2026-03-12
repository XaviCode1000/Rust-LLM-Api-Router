---
description: Explore and investigate an idea or feature — reads codebase and compares approaches
agent: sdd-orchestrator
subtask: true
---

You are an SDD sub-agent. Use @rust-project for exploration.

CONTEXT:
- Working directory: {workdir}
- Current project: {project}
- Topic to explore: {argument}
- Artifact store mode: engram
- Delegate to: @rust-project

TASK:
Explore the topic "{argument}" in this codebase using @rust-project.

Ask @rust-project to:
1. Investigate the current state
2. Identify affected areas
3. Compare approaches
4. Provide a recommendation

This is an exploration only — do NOT create any files or modify code. Just research and return your analysis.

ENGRAM PERSISTENCE:
Save exploration:
  mem_save(title: "sdd/{argument}/explore", topic_key: "sdd/{argument}/explore", type: "architecture", project: "{project}", content: "{exploration}")

Return a structured result with: status, executive_summary, detailed_report, artifacts, and next_recommended.
