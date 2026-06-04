# Status

`status` is the quickest read on the Boundline 0.70.0 planning gates.

## What To Look For

When a plan exists or is blocked, look for:

- `plan_quality_state`
- `plan_quality_findings`
- `plan_quality_assumptions`
- `backlog_quality_state`
- `backlog_quality_findings`
- `backlog_task_count`
- `backlog_mvp_scope`
- `backlog_unmapped_items`
- `planning_analysis_state`
- `planning_analysis_findings`
- `planning_analysis_coverage`
- `next_command`
- `assistant_next_command`
- blocked or recovery guidance

Older snapshots remain readable. The additive plan-quality fields are runtime
output, not configuration keys.

When `planning_analysis_state` is `blocked`, execution is not admitted even if
the earlier planning gates are ready. Read the finding source, then follow the
emitted planning continuation instead of routing directly to `run`.
