# Contract: Governance Session Projection

## Purpose

Define the user-visible session and terminal output fields that expose governed stage state through existing `status`, `next`, `run`, and `inspect` surfaces.

## Session View Fields

When governance state exists for the active task, Boundline projects the following optional fields:

- `latest_governance_stage`: Stable stage key such as `bug-fix:implement`
- `latest_governance_runtime`: `local` or `canon`
- `latest_governance_state`: `pending_selection`, `running`, `governed_ready`, `awaiting_approval`, `blocked`, `completed`, or `failed`
- `latest_governance_mode`: Canon mode when the Canon runtime is active
- `latest_governance_run_ref`: Canon run identifier when a Canon run exists
- `latest_governance_packet_ref`: Reference to the governed packet reused by Boundline
- `latest_governance_packet_source_stage`: Source stage that produced the reused governed packet when reuse occurred
- `latest_governance_packet_binding_reason`: Why the packet was reused, such as `same_stage_rerun` or `upstream_stage_context`
- `latest_governance_approval`: `not_needed`, `requested`, `granted`, `rejected`, or `expired`
- `latest_governance_autopilot`: `enabled` or `disabled`
- `latest_governance_decision`: Short headline for the latest autopilot or runtime-selection decision
- `latest_governance_candidates`: Ordered list of candidate autopilot actions considered for the latest decision

## Output Rules

- Fields are omitted entirely when the active task has no governance state.
- `latest_governance_mode` and `latest_governance_run_ref` are omitted for local-runtime stages.
- `latest_governance_packet_ref` is omitted until a governed packet exists.
- `latest_governance_packet_source_stage` and `latest_governance_packet_binding_reason` are omitted when no packet reuse occurred.
- `latest_governance_candidates` is omitted until a governed autopilot or runtime-selection decision has been recorded.
- `next_command` must remain aligned with governance state:
  - For `awaiting_approval`, it should guide the operator toward `status` or `inspect` rather than continuing execution blindly.
  - For `blocked`, it should guide the operator toward the safest inspection or correction command.
  - For `completed` or `governed_ready`, it should continue the normal Boundline flow.
- When the current state is `awaiting_approval`, a later `status`, `step`, or `run` invocation must refresh approval state before rendering a continuation hint.

## Inspect Expectations

- `inspect` must include a governance timeline separate from or clearly distinguishable from recovery and review events.
- Governance timeline entries must make runtime selection, Canon mode binding, approval waits, packet rejection, and autopilot decisions easy to identify.

## Terminal Safety Rules

- A stage with `latest_governance_state = awaiting_approval` or `blocked` must never be rendered as if the stage were already completed.
- If governance is required, `run` must not report a successful ungoverned continuation past the blocked stage.