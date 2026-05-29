# Boundline Plan Quality Contract

## Summary

Add a Speckit-inspired quality contract to Boundline planning without turning Boundline into a file-first `specs/` workflow. The runtime remains authoritative: `/boundline-plan` plans the active session from an already captured goal, and planning must stop on a structured `phase_request` whenever the plan lacks enough quality to proceed safely.

This follows the same shape as the goal quality contract: additive runtime fields, concise findings, accepted assumptions, assistant-safe routing, and one interactive gate at a time.

## Speckit Seed Notes

- Seed role: first planning-readiness gate in the Speckit analogue sequence.
- First slice: expose `plan_quality_state` and block execution for one missing
  validation-strategy case while preserving `phase_request` routing.
- Depends on: existing goal-quality gate and assistant-safe handoff fields.
- De-duplication: shared gate rendering, `phase_request` handling, and
  assistant routing should be reused by backlog and analysis gates rather than
  restated in separate implementations.

## Public And Runtime Interface Changes

Add these optional fields to session status, orchestrate session snapshots, and rendered status output when a plan exists or planning is blocked:

- `plan_quality_state`: `ready`, `clarification_required`, or `blocked`
- `plan_quality_findings`: concise machine-readable labels for missing or weak planning inputs
- `plan_quality_assumptions`: inferred defaults accepted by the runtime

The fields are additive. Existing consumers that ignore unknown JSON fields must continue to work. Existing `phase_request`, `assistant_resume_command`, and `assistant_next_command` remain the interactive contract.

## Runtime Behavior

Plan quality validation runs after goal quality is satisfied and before Boundline offers execution.

The runtime should check that the plan has:

- technical context sufficient for implementation
- explicit constraints and implementation boundaries
- architecture or approach decisions, including rationale where relevant
- validation strategy tied to the goal success criteria
- governance or risk implications when materially relevant
- no unresolved `NEEDS CLARIFICATION` equivalent in runtime-owned planning state

If quality is insufficient, planning remains non-terminal and emits `phase_request` with exactly one question. The backlog of possible questions is bounded and prioritized by impact: scope and safety first, then user-facing behavior, then technical detail.

Speckit-style artifacts should be mapped to Boundline artifact roles, not required as files:

- research decisions map to planning rationale or Canon discovery/requirements packets
- data model maps to system-shaping or architecture packets
- contracts map to architecture or backlog packets
- quickstart maps to validation strategy or run brief evidence

## Assistant Asset Updates

Update `/boundline-plan` assets for Copilot, Claude, Codex, and Antigravity with the standardized planning sections:

- `User Input`
- `Pre-Execution Checks`
- `Execution Flow`
- `Plan Quality Validation`
- `Reasonable Defaults`
- `Gate Handling`
- `Output Interpretation`
- `Next-Step Routing`
- `Done When`

The assets must state that planning cannot proceed from chat-only assumptions when either `goal_quality_state` or `plan_quality_state` is blocked. They must preserve `plan_quality_state`, `plan_quality_findings`, `plan_quality_assumptions`, and any emitted `phase_request`.

## Tests

Add unit and contract coverage for:

- planning blocks when goal quality is unresolved
- planning blocks when technical context or validation strategy is missing
- low-impact omitted details are recorded as `plan_quality_assumptions`
- status and orchestrate JSON include plan quality projection when present
- assistant plan assets contain the standardized sections and blocked-quality routing rules
- existing planning flows continue to pass when quality is ready

Run:

- `cargo test --test unit`
- `cargo test --test contract`
- `cargo test --test integration human_input_capture_flow::`

## Assumptions

- No new CLI subcommand is required.
- No `specs/` feature directory or Speckit file generation is added.
- Speckit hooks remain out of scope; Boundline uses `phase_request` handoffs.
- Canon may provide planning packets, but Boundline owns the final planning readiness projection.
