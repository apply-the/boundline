# Phase Requests

Boundline 0.67.0 uses `phase_request` as the single recovery handoff for goal
clarification, plan-quality clarification, and planning-stage artifact
requests.

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
