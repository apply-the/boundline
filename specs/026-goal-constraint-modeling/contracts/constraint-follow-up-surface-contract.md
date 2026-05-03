# Contract: Constraint Follow-Up Surface

**Feature**: 026-goal-constraint-modeling  
**Date**: 2026-05-01

## Purpose

Define how planning and follow-up surfaces preserve the negotiated constraint
story after capture.

## Required Surface

- `plan`, `run`, `status`, `next`, and `inspect` must identify the active
  acceptance boundary or summarize it clearly when that boundary materially
  shapes what Boundline can do next.
- Follow-up output must expose which constraint or tradeoff is currently binding
  when the session is blocked, failed, exhausted, or inspect-only.
- When a tradeoff was chosen, the surface must summarize why that choice won
  over the rejected alternative.
- The surface must preserve enough negotiation context that an operator can
  challenge the planning story without reopening hidden planner heuristics.

## Explicit Boundaries

- Follow-up surfaces must not collapse negotiated state back to a plain goal
  string once planning has begun.
- The surface must not imply success when the acceptance boundary is still not
  satisfied or not inspectable.
- Follow-up output must not dump raw internal state when a bounded summary is
  sufficient.