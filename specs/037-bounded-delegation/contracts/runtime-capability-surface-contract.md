# Contract: Runtime Capability Surface

**Feature**: 037-bounded-delegation  
**Date**: 2026-05-03

## Purpose

Define how Boundline exposes declared runtime capability and effort policy through
configuration, planning, and execution surfaces.

## Required Surface

- `config show` must expose the effective routed slot, its assistant binding,
  its declared capability summary, and its declared effort policy.
- `plan`, `run`, `status`, `next`, and `inspect` must be able to name when a
  capability rule or effort policy changed the bounded next action.
- Capability and effort projection must remain attributable to the same
  authority source used for effective routing.

## Explicit Boundaries

- Capability projection must not imply that a runtime was probed or validated
  dynamically unless Boundline recorded that evidence explicitly.
- Effort policy must not silently override route ownership or execution limits.
- The surface must not introduce a second provider abstraction vocabulary that
  competes with the existing slot-routing model.