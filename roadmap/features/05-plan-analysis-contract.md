# Boundline Plan Analysis Contract

Delivered in Boundline `0.70.0`.

## Summary

Add a read-only planning analysis pass at the end of Boundline planning. This
is the Boundline analogue of Speckit analyze, but it remains runtime-owned and
non-destructive. It reports cross-artifact consistency issues across the active
goal, plan projection, governed packets, backlog, validation strategy,
execution handoff, and required governed evidence before execution is offered.

## Delivered Slice

- final planning-readiness gate after plan quality and backlog quality
- source-attributed findings with stable codes and additive coverage metrics
- blocked execution handoff on uncovered success criteria, validation gaps,
  backlog contradictions, missing execution inputs, or producer contract gaps
- compatibility-preserving projection through status, inspect, traces,
  orchestration, and assistant assets

## Public And Runtime Interface Changes

Boundline now projects:

- `planning_analysis_state`
- `planning_analysis_findings`
- `planning_analysis_coverage`

These fields stay additive and remain omitted for compatible older snapshots
where planning analysis never ran.

## Runtime Behavior

Planning analysis runs only after:

1. goal quality
2. plan quality
3. backlog quality

The gate is read-only. It does not mutate files, plan tasks, backlog
artifacts, or Canon packets. When it blocks, the runtime preserves the planning
repair path instead of inventing direct execution continuation.
