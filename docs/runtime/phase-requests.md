# Phase Requests

Boundline 0.79.0 uses `phase_request` as the single recovery handoff for goal
clarification, plan-quality clarification, backlog-quality clarification, and
planning-stage artifact requests.

## Rules

- ask one question at a time
- preserve `phase_request.request_id`
- answer with the runtime's `expected_answer` shape
- resume with the emitted `resume_command` or assistant-safe route
- do not infer execution from chat history

## Plan-Quality Requests

When plan quality is missing a credible validation strategy, the runtime keeps
the session non-terminal, preserves `plan_quality_state`, findings, and
assumptions, and asks the operator for the one missing input that can clear the
gate.

## Backlog-Quality Requests

When a full Canon backlog packet is present but still lacks a governed
execution handoff or equivalent downstream-ready evidence, the runtime keeps
planning non-terminal, preserves the additive backlog-quality fields, and asks
exactly one focused follow-up instead of inventing executable work.

## Planning-Analysis Recovery

When planning analysis blocks execution, Boundline preserves
`planning_analysis_state`, `planning_analysis_findings`, and
`planning_analysis_coverage` in the session and routes back through the same
planning continuation model. The runtime may ask for one missing authored input
or require a regenerated Canon packet, but it must not fabricate Canon-owned
evidence or skip directly to execution.
