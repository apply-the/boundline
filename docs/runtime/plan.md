# Plan

Boundline 0.78.0 makes planning readiness a runtime gate, not a chat
convention.

## What `plan` Does

- evaluates goal quality first
- evaluates plan quality next
- evaluates backlog quality after plan quality
- evaluates planning analysis only after backlog quality is ready
- records `plan_quality_state`, `plan_quality_findings`, and
  `plan_quality_assumptions` when present
- records `backlog_quality_state`, `backlog_quality_findings`,
  `backlog_task_count`, `backlog_mvp_scope`, and `backlog_unmapped_items` when
  a Canon backlog packet is expected or available
- records `planning_analysis_state`, `planning_analysis_findings`, and
  `planning_analysis_coverage` after the runtime checks goal outcomes,
  validation anchors, execution handoff inputs, backlog sequencing, and any
  governed evidence already present in the session
- records additive large-codebase context substrate fields such as typed
  `context_pack_entries`, `omission_findings`, `repository_map_state`,
  `snapshot_cache_state`, and `patch_safe_edit_attempts` when advanced context
  selection is active
- stops on one `phase_request` when a planning gate needs missing validation,
  missing backlog handoff evidence, or another blocking planning input
- keeps execution handoff withheld until the gate clears

## What To Read When It Blocks

Use `status`, `next`, and `inspect` to see the same runtime decision from
different surfaces. Do not invent execution from chat history, and do not
treat `planning_analysis_findings` as permission to rewrite Canon or the plan
outside the runtime repair path.

When the large-codebase substrate reports a blocking omission, downgraded
critical artifact, stale tracked snapshot cache, or unsafe oversized full-read
request, planning remains blocked until the runtime can repair or narrow the
context safely.
