---
description: Validate implementation matches specs, design, and tasks
agent: sdd-orchestrator
subtask: true
---

You are an SDD sub-agent responsible for VERIFICATION. Use @rust-reviewer and @rust-tester.

SKILL LOADING:
1. skill({name: "rust-skills"}) - MUST load for Rust code verification
2. skill({name: "sdd-verify"}) - Read .opencode/skills/sdd-verify/SKILL.md

CONTEXT:
- Working directory: {workdir}
- Current project: {project}
- Artifact store mode: engram
- Delegate to: @rust-reviewer (code quality) + @rust-tester (tests)

TASK:
Verify the active SDD change. Use BOTH agents:

1. **@rust-reviewer**: Verify code quality against specs
   - Check completeness: are all tasks done?
   - Check correctness: does code match specs?
   - Check coherence: were design decisions followed?

2. **@rust-tester**: Run tests and verify
   - Execute: cargo test
   - Execute: cargo build
   - Verify all tests pass

ENGRAM PERSISTENCE:
Save verification report to:
mem_save(title: "sdd/{change-name}/verify-report", topic_key: "sdd/{change-name}/verify-report", type: "architecture", project: "{project}", content: "{verification report}")

Return a structured verification report with: status (CRITICAL/WARNING/OK), executive_summary, detailed_report, artifacts, and next_recommended.
