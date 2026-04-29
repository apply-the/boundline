# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Rust 1.95.0, Python 3.11 or NEEDS CLARIFICATION]  
**Primary Dependencies**: [core libraries and runtime dependencies or NEEDS CLARIFICATION]  
**Storage**: [e.g., in-memory state, files, database, or N/A]  
**Testing**: [e.g., cargo test, pytest, contract tests or NEEDS CLARIFICATION]  
**Target Platform**: [e.g., macOS/Linux developer workstations, Linux CI or NEEDS CLARIFICATION]
**Project Type**: [e.g., single library crate, CLI, service, or NEEDS CLARIFICATION]  
**Execution Model**: [e.g., sequential task loop with bounded retries or NEEDS CLARIFICATION]  
**Observability Surface**: [e.g., persisted execution trace, structured logs, CLI summary or NEEDS CLARIFICATION]  
**Performance Goals**: [delivery-facing targets or NEEDS CLARIFICATION]  
**Constraints**: [explicit limits, non-goals, and external boundaries or NEEDS CLARIFICATION]  
**Scale/Scope**: [expected task volume, step count, or user reach or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: Explain how this feature directly improves bounded engineering task delivery.
- Delivery-first scope: Confirm execution, orchestration, decomposition, or validation work is prioritized ahead of optimization or polish.
- Bounded execution: Identify explicit start conditions, terminal conditions, and max step or retry limits.
- Stateful execution: Describe shared task context, read and write points, and justify any stateless segment.
- Mutable planning: Describe initial planning plus replanning, step insertion, or replacement behavior.
- Sequential-first design: Confirm one-step-at-a-time execution or justify why the constitution allows an exception.
- Tool-agent symmetry: Show how reasoning and action remain explicit in the execution model.
- Observability and explicit intelligence: List trace surfaces, visible decisions, failure signals, and any heuristic behavior that must be exposed.
- Non-goals and external separation: Confirm the plan does not depend on Canon behavior or reintroduce deferred scope such as councils or voting outside an explicitly reprioritized, bounded review slice, provider abstraction complexity beyond the approved slice, long-term memory, UI/UX, or deployment pipelines.
- Minimal slice: Explain the smallest independently valuable capability delivered by this plan.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Keep the structure minimal, delivery-focused, and sequential-
  first. Do not introduce extra top-level projects or UI/runtime surfaces unless
  the Constitution Check explicitly justifies them.
-->

```text
src/
├── domain/
├── orchestrator/
├── agents/
├── tools/
└── tracing/

tests/
└── unit/
├── integration/
└── contract/
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above. If new top-level directories are introduced, explain why
the constitution allows that additional complexity.]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., background worker] | [specific delivery need] | [why sequential execution is insufficient] |
