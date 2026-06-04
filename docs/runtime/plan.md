# Plan

Boundline 0.69.0 makes planning readiness a runtime gate, not a chat
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
- stops on one `phase_request` when a planning gate needs missing validation,
  missing backlog handoff evidence, or another blocking planning input
- keeps execution handoff withheld until the gate clears

## What To Read When It Blocks

Use `status`, `next`, and `inspect` to see the same runtime decision from
different surfaces. Do not invent execution from chat history.
