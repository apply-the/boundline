# Plan

Boundline 0.68.0 makes plan quality a runtime gate, not a chat convention.

## What `plan` Does

- evaluates goal quality first
- evaluates plan quality next
- records `plan_quality_state`, `plan_quality_findings`, and
  `plan_quality_assumptions` when present
- stops on one `phase_request` when the plan needs a missing validation
  strategy or another blocking planning input
- keeps execution handoff withheld until the gate clears

## What To Read When It Blocks

Use `status`, `next`, and `inspect` to see the same runtime decision from
different surfaces. Do not invent execution from chat history.
