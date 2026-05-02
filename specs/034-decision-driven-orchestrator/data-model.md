# Data Model: Decision-Driven Orchestrator

## ActionSelector

- **Purpose**: Represents the single bounded next action chosen for one native
  loop iteration.
- **Key fields**:
  - `selector_kind`: one of `read`, `search`, `modify`, `test`, `ask`, or `replan`
  - `target`: bounded file, symbol, workspace slice, or clarification subject
  - `rationale`: operator-facing reason this selector was chosen now
  - `expected_outcome`: intended result of taking the selector
  - `verification_intent`: what evidence will confirm or reject the selector's value
- **Rules**:
  - exactly one selector is active per decision iteration
  - `ask` and `replan` are bounded control selectors, not hidden background work
  - selector choice must be derivable from explicit evidence captured at decision time

## DecisionRecord

- **Purpose**: Persists one complete observe-decide-act-verify result for the
  native loop.
- **Key fields**:
  - `decision_id`
  - existing high-level decision family used by flow or recovery policy
  - embedded `ActionSelector`
  - `status`: pending, dispatched, verified, failed, recovered
  - `tool_result` or action result snapshot
  - `created_at` and `completed_at`
  - `recovery_of`: optional pointer to the prior failed decision being recovered
- **Relationships**:
  - belongs to exactly one session execution history
  - is projected into trace events and session follow-through surfaces
  - may point to earlier decisions when recovery or replanning occurs

## ObservationSnapshot

- **Purpose**: Captures the bounded evidence visible when selecting the next action.
- **Key fields**:
  - `remaining_targets`
  - `accumulated_evidence`
  - `latest_validation_status`
  - `latest_changed_files`
  - `context_summary` and selected context inputs
  - `clarification_state` when the operator already owes missing information
- **Lifecycle**:
  - created at the start of each loop iteration
  - consumed by selector rules
  - summarized into trace payloads or follow-through projections when needed

## ClarificationActionState

- **Purpose**: Represents the explicit operator-facing blocked state created by
  an `ask` selector.
- **Key fields**:
  - `headline`
  - `prompt`
  - `missing_information`
  - `next_capture_action`
- **Rules**:
  - created only when no credible engineering action is currently available
  - must map to one bounded operator recovery path rather than generic failure text
  - cleared when new captured input or replanning resolves the missing information

## DecisionProjection

- **Purpose**: The operator-facing summary of decision-driven execution surfaced
  through plan, run, status, next, and inspect.
- **Key fields**:
  - `current_selector`
  - `selector_rationale`
  - `evidence_basis`
  - `verification_intent`
  - `recovery_state`
  - `terminal_reason` when execution stops
- **Relationships**:
  - derived from `DecisionRecord` and `ObservationSnapshot`
  - rendered on session-native and compatibility-authoritative read surfaces
  - must remain concise enough to explain one bounded next step without replaying the full trace