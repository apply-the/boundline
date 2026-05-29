# Boundline Plan Analysis Contract

## Summary

Add a read-only planning analysis pass at the end of Boundline planning. This is the Boundline analogue of Speckit analyze, but it remains runtime-owned and non-destructive. It should report cross-artifact consistency issues across the captured goal, plan projection, governed packets, backlog, and validation strategy before execution is offered.

The analysis is part of `/boundline-plan` in v1. A separate command can be considered later only if runtime usage shows a need.

## Public And Runtime Interface Changes

Add optional planning analysis fields to session status, orchestrate snapshots, and rendered output:

- `planning_analysis_state`: `clean`, `findings`, or `blocked`
- `planning_analysis_findings`: concise findings with severity and source labels where available
- `planning_analysis_metrics`: counts such as requirements, tasks, coverage, ambiguity, and critical issues
- `planning_analysis_coverage`: requirement or success-criteria coverage summary

These fields are additive and should be omitted when analysis has not run.

## Runtime Behavior

The analysis pass runs after plan quality and backlog quality are available. It must not modify files, packets, tasks, or session artifacts except for persisting its own runtime projection.

Detection should focus on high-signal findings:

- goal, plan, backlog, and validation strategy inconsistencies
- success criteria with no mapped plan or backlog coverage
- vague or unmeasurable plan items
- backlog tasks with no mapped goal, plan decision, or acceptance criterion
- governance, risk, or contract conflicts
- terminology drift that would affect implementation

Severity should use a compact scale:

- `critical`: execution should not proceed
- `high`: likely implementation or validation failure
- `medium`: important ambiguity or missing non-functional coverage
- `low`: wording or traceability improvement

If analysis is blocked, Boundline must not offer execution. It should route through the existing `phase_request` or governed planning-stage gate.

## Assistant Asset Updates

Update `/boundline-plan`, `/boundline-status`, `/boundline-inspect`, and `/boundline-run` assets where relevant:

- preserve `planning_analysis_state`, `planning_analysis_findings`, `planning_analysis_metrics`, and `planning_analysis_coverage`
- treat `planning_analysis_state: blocked` as a real stop condition
- do not apply remediation edits automatically from analysis output
- route to the emitted planning continuation or `/boundline-plan` when analysis blocks execution

## Tests

Add coverage for:

- clean plan reports `planning_analysis_state: clean`
- uncovered success criterion reports findings and coverage gap
- task with no mapped goal or plan rationale is reported
- governance contradiction produces a blocked analysis state
- analysis output is read-only and does not mutate plan/backlog artifacts
- assistant assets document the read-only analysis contract and blocked routing behavior

Run:

- `cargo test --test unit`
- `cargo test --test contract`
- `cargo test --test integration human_input_capture_flow::`

## Assumptions

- No standalone analyze command is added in v1.
- Analysis persists only runtime projection fields; it does not rewrite governed packets.
- Speckit hooks and `.specify/extensions.yml` remain out of scope.
- Canon backlog schema changes, if needed, must be handled through a Canon Speckit feature before Boundline depends on them.
