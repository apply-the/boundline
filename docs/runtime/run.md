# Run

`run` continues only after the planning gates are ready.

## What Can Still Stop Execution

- `goal_quality_state`
- `plan_quality_state`
- `backlog_quality_state`
- `planning_analysis_state`
- large-codebase context omissions or unsafe oversized full-read refusal
- capability-provider readiness, permission admission, and accepted evidence
  when the selected route depends on an active provider

Execution is withheld when planning still depends on missing or downgraded
critical context. If the large-codebase substrate reports repository-map
failure, stale tracked snapshot cache, digest-only evidence where full context
is required, or patch-safe edit drift on a large file, route back to planning
repair instead of forcing execution forward.

When a capability provider is active, `run` may also stop on provider-specific
runtime policy:

- setup requirements still incomplete
- health failure before execute
- explicit permission-envelope mismatch
- prepare-time missing context or evidence
- post-execution evidence rejection
