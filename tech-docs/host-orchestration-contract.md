# Host Orchestration Contract

`boundline orchestrate` is the host-facing orchestration entrypoint for chat surfaces that need explicit, inspectable phase boundaries.

## Command

Use the orchestrator command in one of two primary modes:

```text
boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> --intent continue-until-phase-request --json-stream
boundline orchestrate --workspace <workspace> --intent continue-until-phase-request --request-id <request_id> --answer "<answer>" --json-stream
boundline orchestrate --workspace <workspace> --intent continue-until-terminal --json-stream
```

The command keeps Boundline authoritative for session state, phase transitions, governance, reviewer routing, and terminal conditions.

## Request Fields

- `--workspace <path>`: target workspace root.
- `--goal "<goal>"`: optional new or refined goal text; when present, Boundline captures or updates the active session goal.
- `--brief <path>`: optional authored Markdown brief inputs.
- `--flow <name>`: optional requested flow for planning.
- `--governance <local|canon>` plus `--risk`, `--zone`, `--owner`: optional governance hints carried into goal capture.
- `--assistant-host <host>`: optional host identity used when Boundline can emit assistant-safe follow-up routes such as `assistant_resume_command`.
- `--request-id <request_id>`: required when resuming a structured `phase_request` that expects an explicit host answer or acknowledgement.
- `--answer "<answer>"`: optional free-text answer used to satisfy a structured clarification request before Boundline continues.
- `--planning-stage-complete <stage_key>`: optional explicit acknowledgement that the host completed the requested planning-stage artifact before resuming orchestration.
- `--intent plan-only`: stop after planning.
- `--intent continue-until-phase-request`: stop at the next explicit host boundary.
- `--intent continue-until-terminal`: continue until terminal execution unless a real blocking boundary is encountered.
- `--json-stream`: emit newline-delimited JSON.

## Event Frames

Each output line is one complete JSON object with these stable top-level fields:

- `event_id`
- `timestamp_ms`
- `event_kind`
- `session_ref` when a session is available
- `phase_kind` when the event belongs to a phase
- `stage_key` when the stage is known
- `message`
- `artifact` when the event records or requests an artifact
- `instruction` when the operator must act
- `resume_command` when Boundline expects an explicit resume step
- `assistant_resume_command` when Boundline can emit a host-safe resume route
- `next_command` when the session already exposes one
- `assistant_next_command` when Boundline can emit a host-safe follow-up route
- `session_status` when a status snapshot is available
- `trace_summary` when execution produced a trace summary

## Supported `event_kind` Values

- `session_opened`
- `session_updated`
- `phase_started`
- `phase_request`
- `artifact_recorded`
- `governance_update`
- `execution_update`
- `terminal`

## `phase_request` Contract

`phase_request` is the explicit handoff from Boundline to the host. Hosts must stop automatic continuation on this event.

A `phase_request` frame includes:

- `phase_kind`
- `stage_key`
- `phase_request.request_id`
- `phase_request.kind`
- `phase_request.reason`
- `phase_request.question`
- `phase_request.expected_answer`
- `artifact.artifact_kind`
- `artifact.artifact_ref` when an artifact path exists
- `instruction`
- `resume_command`
- `assistant_resume_command` when the host mapping is known

Hosts may help author or edit the requested artifact, but Boundline remains the authority that resumes, validates, and advances the session.

## Goal Clarification Requests

Goal clarification gates are runtime objects, not prompt etiquette. When `continue-until-phase-request` stops on a goal clarification, the host must:

- surface `phase_request.reason` and ask exactly `phase_request.question`
- preserve `phase_request.request_id`
- collect one answer matching `phase_request.expected_answer`
- resume with the emitted `resume_command` or an equivalent orchestrator call that includes `--request-id <request_id>` and `--answer "<answer>"`

If clarification is still missing after the answer is applied, Boundline may emit another goal `phase_request`; hosts should continue one structured question at a time until the runtime advances to planning or another terminal boundary.

## Plan Quality Requests

When plan quality stops progress, the host must treat the emitted `phase_request` as the runtime quality gate, not as a planning-stage artifact request. The common case in the first shipped slice is a missing validation strategy, but the host should preserve the same handling for any plan-quality finding that keeps execution handoff blocked.

For a plan-quality `phase_request`, the host must:

- surface `phase_request.reason` and ask exactly `phase_request.question`
- preserve `phase_request.request_id`
- keep `plan_quality_state`, `plan_quality_findings`, and `plan_quality_assumptions` visible in any status snapshot
- resume with the emitted `resume_command` or an equivalent orchestrator call that includes `--request-id <request_id>` and `--answer "<answer>"`

Hosts must not synthesize execution continuation from chat-only assumptions while plan quality remains `clarification_required` or `blocked`.

## Planning Stage Requests

When governed planning selects delivery-stage briefs, `continue-until-phase-request` emits one `phase_request` at a time for the next planning stage Boundline wants the host to help author:

- `plan:requirements`
- `plan:architecture`
- `plan:backlog`

Those frames use `artifact.artifact_kind = planning_stage_brief` and point `artifact.artifact_ref` at the materialized planning-stage brief under `.boundline/governance/planning/<stage>/brief.md` when that brief exists. Their `resume_command` includes `--planning-stage-complete <stage_key>` so the host can acknowledge the stage it just finished before Boundline advances to the next planning handoff. If no stage-specific planning brief is available, Boundline falls back to the session `plan_brief` boundary.

Hosts should treat these planning-stage `phase_request` frames as a sequential handoff: help author or review the requested artifact, then resume with the emitted `resume_command`. Intermediate planning stages resume back into `continue-until-phase-request`; the final planning-stage resume can continue into terminal execution.
