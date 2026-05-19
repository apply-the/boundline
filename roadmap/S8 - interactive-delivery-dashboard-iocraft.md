# S8 - Interactive Delivery Dashboard

## Owner

Boundline

## Status

Next feature, re-scoped from standalone product to operator shell

## Strategic Role

S8 is the operator surface for governed delivery. It should make Boundline's
runtime legible during complex, multi-step work without creating a second
implementation of Boundline.

The dashboard is an operator shell over existing Boundline commands, session
state, trace events, checkpoints, plans, findings, and Canon references.

## Problem

Linear terminal output is no longer enough for:

- project-scale delivery
- stop semantics
- review councils
- guidance and guardian findings
- Context Pack inspection
- Canon artifact inspection
- confirmation, rejection, replanning, recovery, and evidence review

Without a dashboard, Boundline can be architecturally strong but operationally
hard to trust.

## Core Scope

### Must Cover

- session selector
- live Pilot Loop monitor
- current phase and active step
- stop rules panel
- Context Pack explorer
- guidance, guardian, and finding panel
- current GoalPlan panel
- trace timeline
- checkpoint view
- confirm, reject, replan, and recover actions
- read-only Canon project memory explorer
- read-only packet and evidence viewer
- dashboard-oriented doctor view
- keyboard-first navigation
- clear degraded mode when terminal capabilities are limited

### Must Not Cover In V1

- duplicated workflow engine
- independent config implementation
- independent init implementation
- independent governance logic
- independent provider orchestration
- new state store
- different behavior from the CLI

## Architectural Model

```text
boundline CLI/runtime remains authoritative
dashboard reads state and event streams
dashboard invokes existing commands
dashboard never forks runtime semantics
```

The standard `boundline` automation surface remains the source of truth for
CI, scripting, assistant command packs, and normal terminal use. A separate
dashboard crate or feature-gated binary is acceptable only if it consumes the
same state and command contracts as the CLI.

## Suggested Technology

Primary candidate:

- `iocraft` for interactive terminal UI, if it fits current Rust ergonomics and
  rendering needs

Fallback:

- `ratatui` if iocraft proves too immature or too restrictive

Supporting pieces:

- stable JSON event model from Boundline runtime
- file watcher for `.boundline/session.json`, trace files, and findings
- snapshot rendering for terminal compatibility
- no TUI dependency in the slim CLI binary unless feature-gated or separated

## Required Runtime Prerequisite

S8 should push Boundline to formalize structured runtime events:

```text
session_started
context_pack_built
plan_ready
stop_rule_triggered
guardian_started
guardian_finished
finding_emitted
checkpoint_created
confirmation_required
run_started
run_finished
recovery_available
```

## Acceptance Criteria

- Dashboard can attach to an existing Boundline session.
- Dashboard can launch a new session by invoking existing commands.
- Stop rules are visible without scrolling.
- Context Pack items show reason, source, budget cost, and authority.
- Guardian findings show severity, evidence refs, and resolution status.
- Canon artifacts are browsable read-only.
- Confirm, reject, replan, and recover actions produce the same runtime behavior
  as CLI.
- CLI works unchanged without dashboard dependencies.
- Dashboard has tests for rendering critical states.

## Risks

- Dashboard becomes a parallel product.
- UI code leaks into core runtime.
- Runtime state is not structured enough.
- Pretty UI hides weak trace semantics.

## Hard Rule

The dashboard must reveal Boundline's truth. It must not create a new truth.
