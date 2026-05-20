# Data Model: Interactive Delivery Dashboard

## DashboardSnapshot

Represents the complete operator-facing state needed to render the dashboard at one point in time.

### Fields

- `workspace_ref`: absolute or display-safe workspace reference.
- `snapshot_id`: unique id for the snapshot.
- `captured_at`: timestamp for snapshot assembly.
- `session_revision`: revision or equivalent monotonic value used to detect stale actions.
- `session_view`: `DashboardSessionView`.
- `timeline`: ordered list of `RuntimeEventProjection`.
- `panels`: `InspectionPanelSet`.
- `actions`: list of currently allowed `DashboardActionOption`.
- `degraded_state`: optional `DegradedDashboardState`.
- `branding`: `TerminalBrandMark`.

### Validation Rules

- Must identify whether state came from session-native state, compatibility trace state, or degraded fallback.
- Must include either a valid session view or a degraded state explaining why no session view is available.
- Must not include mutable governed-reference fields.
- Must not represent an action as allowed unless the current state supports it.

## DashboardSessionView

Represents the first-screen summary of the active delivery state.

### Fields

- `session_id`
- `goal`
- `route_kind`
- `route_owner`
- `active_flow`
- `flow_state`
- `goal_plan_state`
- `goal_plan_revision`
- `current_stage`
- `current_step_id`
- `current_step_index`
- `execution_condition`
- `latest_status`
- `next_action_label`
- `next_command`
- `blocking_reason`
- `compatibility_context`

### Validation Rules

- Must label compatibility context explicitly when the current follow-up comes from compatibility trace state.
- Must include a blocking reason when `execution_condition` is waiting, blocked, failed, exhausted, invalid, or degraded.
- Must preserve the same next-command meaning as the normal status or next surface.

## RuntimeEventProjection

Represents one item in the dashboard timeline.

### Fields

- `event_id`
- `event_kind`: session, plan, action, verification, retry, replan, recovery, checkpoint, governance, finding, degraded, terminal.
- `occurred_at`
- `stage`
- `step_id`
- `status`
- `headline`
- `evidence_refs`
- `trace_ref`
- `details`

### Validation Rules

- Must preserve ordering from authoritative trace or session evidence.
- Must not invent events that are not represented in Boundline-owned state.
- Must include trace or session evidence for action, failure, recovery, and terminal events when available.

## InspectionPanelSet

Groups dashboard panel data for deeper inspection.

### Fields

- `goal_plan`: optional `GoalPlanPanel`.
- `evidence`: list of `EvidencePanelItem`.
- `context_degradation`: list of degraded or omitted context facts.
- `stop_rules`: list of active or recently applied stop conditions.
- `findings`: list of `FindingPanelItem`.
- `checkpoints`: list of `CheckpointPanelItem`.
- `governed_references`: list of `GovernedReferencePanelItem`.

### Validation Rules

- Must allow empty panel lists without failing snapshot assembly.
- Must distinguish unavailable data from available empty data.
- Must keep governed references read-only.

## DashboardActionOption

Represents one action currently available to the operator.

### Fields

- `action_kind`: confirm, reject, replan, recover, launch, continue, inspect-only.
- `label`
- `description`
- `requires_reason`
- `requires_confirmation`
- `target_session_revision`
- `expected_result`
- `disabled_reason`

### Validation Rules

- Must include `target_session_revision` for mutating actions.
- Must not expose a mutating action when the session is stale, invalid, or blocked by a higher-priority stop condition.
- `reject` and selected `replan` actions must capture a bounded operator reason when required by the current state.

## DashboardActionRequest

Represents the operator's request to apply an action.

### Fields

- `request_id`
- `workspace_ref`
- `action_kind`
- `target_session_id`
- `target_session_revision`
- `operator_reason`
- `requested_at`

### Validation Rules

- Must fail closed when the target session revision does not match current authoritative state.
- Must reject unknown action kinds.
- Must reject missing operator reasons when the chosen action requires one.
- Must not mutate governed artifacts.

## DashboardActionResult

Represents the outcome of applying one dashboard action.

### Fields

- `request_id`
- `outcome`: applied, refused, degraded.
- `state_transition`
- `next_snapshot_ref`
- `next_command`
- `trace_refs`
- `refusal_reason`
- `operator_message`

### Validation Rules

- Applied results must include refreshed state evidence or a next snapshot reference.
- Refused results must include a refusal reason and the current valid next action when available.
- Degraded results must point to the normal command path that remains available.

## DegradedDashboardState

Represents fallback behavior when the dashboard cannot operate fully.

### Fields

- `degraded_reason`
- `severity`: info, warning, blocked.
- `available_commands`
- `unavailable_panels`
- `recovery_hint`

### Validation Rules

- Must be explicit whenever interactive rendering, workspace discovery, state loading, action dispatch, or governed-reference reading fails.
- Must never hide normal command fallback when one exists.

## TerminalBrandMark

Represents dashboard branding.

### Fields

- `wordmark_lines`
- `color_profile`: color, monochrome.
- `min_width`
- `fallback_label`

### Validation Rules

- Must contain only terminal-safe text.
- Must not depend on image files, SVGs, or wide ANSI banner art.
- Must degrade to plain `boundline` when color or width is insufficient.

## State Transitions

```text
No Workspace
  -> DegradedDashboardState

No Active Session
  -> DashboardSnapshot(inspect-only actions)
  -> launch action
  -> DashboardSnapshot(session initialized)

Plan Proposed
  -> confirm action
  -> Plan Confirmed

Plan Proposed
  -> reject action
  -> Replan Requested or Stopped

Ready To Run
  -> continue action
  -> Running or Terminal

Failed / Blocked / Exhausted
  -> recover action
  -> Recovery Proposed, Replan Requested, or Stopped

Any Snapshot
  -> stale action request
  -> DashboardActionResult(refused)

Any Snapshot
  -> degraded read or render
  -> DegradedDashboardState
```
