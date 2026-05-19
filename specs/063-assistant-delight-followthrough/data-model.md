# Data Model: S7.1 Assistant Delight Follow-Through

## Design Approach

This slice reuses existing authoritative session and trace state, then adds
small projection-oriented models for the new explanation, inspect, parity, and
feedback behaviors. No new external persistence surface is required.

## Existing Authoritative Entities

### ActiveSessionRecord

- Purpose: authoritative workspace-local session snapshot used by status,
  inspect, and orchestration.
- Existing fields used by this slice:
  - `session_id`
  - `workspace_ref`
  - `goal`
  - `latest_status`
  - `latest_terminal_reason`
  - `latest_trace_ref`
  - `created_at`
  - `updated_at`
  - `governance_lifecycle`
  - `latest_voting`
- Relationships:
  - references the active trace through `latest_trace_ref`
  - may carry the latest reasoning-profile state through nested governance
    lifecycle data
- Validation rules:
  - non-empty `session_id` and `workspace_ref`
  - `updated_at >= created_at`
  - terminal session states require a terminal reason
- State transitions relevant here:
  - bootstrap -> active session
  - active session -> blocked, clarification-required, failed, exhausted, or
    succeeded
  - active session -> updated usefulness signal summary after delight output

### TraceSummaryView

- Purpose: flattened, inspectable trace projection already consumed by CLI
  output surfaces.
- Existing fields used by this slice:
  - `context_summary`
  - `context_credibility`
  - `context_primary_inputs`
  - `context_provenance`
  - `context_staleness_reason`
  - `decision_timeline`
  - `executed_steps`
  - `recovery_events`
  - `governance_timeline`
  - `governance_reason`
  - `governance_approval_provenance`
  - `governance_next_action`
  - `review_timeline`
  - `reasoning_profile`
  - `terminal_status`
  - `terminal_reason`
- Relationships:
  - aggregates one trace into operator-facing evidence buckets
  - links reasoning, review, governance, context, and recovery into one view
- Validation rules:
  - must preserve `terminal_status` and `terminal_reason`
  - view buckets may be empty only when the output also discloses the missing
    source or fallback state
- State transitions relevant here:
  - raw persisted trace -> flattened trace summary
  - flattened trace summary -> explanation and inspect closure projections

### ProfileActivationRecord

- Purpose: persisted reasoning-profile activation details used to explain why a
  profile was selected and what it contributed.
- Existing fields used by this slice:
  - `profile_id`
  - `trigger`
  - `activation_reason`
  - `status`
  - `participants`
  - `posture`
  - `independence`
  - `outcome`
  - `confidence`
- Relationships:
  - nested inside `TraceSummaryView.reasoning_profile`
  - may also be carried through session governance lifecycle state
- Validation rules:
  - non-empty activation and stage identifiers
  - non-empty activation reason
  - profile identity must match the selected definition
  - optional posture, outcome, and confidence records must each validate when
    present
- State transitions relevant here:
  - absent -> active reasoning profile disclosure
  - active -> degraded or unavailable disclosure when evidence becomes weak
  - active -> completed outcome and next-action projection

## New Or Extended Projection Entities

### ReasoningProfileDisclosure

- Purpose: user-facing explanation of whether advanced reasoning was active and
  what it changed.
- Fields:
  - `profile_id`
  - `status`
  - `trigger`
  - `activation_reason`
  - `confidence_level`
  - `admission_effect`
  - `posture_contract_line`
  - `participant_summary`
  - `contribution_summary`
  - `fallback_disclosure`
- Source relationships:
  - derived from `ProfileActivationRecord`
  - joined with existing delight explanation summaries from status or inspect
- Validation rules:
  - if a profile is present, `activation_reason` must be surfaced
  - if `confidence_level` is present, `contribution_summary` or an explicit
    fallback reason must also be surfaced
  - if no profile is present, `fallback_disclosure` is required
- State transitions:
  - `not-available` -> `active`
  - `active` -> `degraded`
  - `active` or `degraded` -> `completed`

### InspectClosureView

- Purpose: human-facing inspect surface for one of the remaining S7.1 closure
  views.
- Fields:
  - `view_kind` (`context`, `council`, `timeline`)
  - `headline`
  - `narrative_lines`
  - `source_attribution`
  - `missing_inputs`
  - `terminal_status`
  - `terminal_reason`
  - `next_action`
- Source relationships:
  - derived from `TraceSummaryView`
  - may include reasoning-profile disclosure when that affects the view
- Validation rules:
  - `narrative_lines` must be non-empty when the underlying view has any
    authoritative source lines
  - if the underlying source set is empty, `missing_inputs` must explicitly say
    what is absent
  - timeline output must preserve the authoritative terminal status and reason
- State transitions:
  - `requested` -> `rendered`
  - `requested` -> `missing-state`
  - `rendered` -> `rendered-with-fallback`

### HostParityDecision

- Purpose: explicit support contract for how each assistant host exposes the
  delight follow-through surface.
- Fields:
  - `host_id` (`claude`, `codex`, `copilot`, `cursor`, `gemini`)
  - `support_mode` (`repo-local-full`, `copy-ready-assets`, `manual-fallback`)
  - `default_palette_commands`
  - `contextual_commands`
  - `state_authority`
  - `fallback_cli`
  - `operator_note`
- Source relationships:
  - derived from assistant manifests, host docs, and packaged command assets
- Validation rules:
  - every host must declare exactly one `support_mode`
  - every host must reference `.boundline/session.json` or CLI output as state
    authority
  - fallback modes require an explicit operator note and CLI path
- State transitions:
  - `manual-fallback` -> `copy-ready-assets`
  - `copy-ready-assets` -> `repo-local-full`
  - no host may remain in an undocumented intermediate state

### DelightFeedbackSignal

- Purpose: lightweight, session-scoped measure of whether a delight surface was
  useful.
- Fields:
  - `session_id`
  - `trace_ref`
  - `first_useful_answer_at`
  - `first_useful_answer_command`
  - `total_explanations`
  - `attributed_explanations`
  - `accepted_next_actions`
  - `overridden_next_actions`
  - `next_action_outcome` (`accepted`, `overridden`, `not-applicable`, `unknown`)
  - `override_reason`
  - `captured_at`
- Source relationships:
  - anchored to `ActiveSessionRecord` and the current trace summary
  - projected through status or inspect rather than a separate analytics feed
- Validation rules:
  - `captured_at` is required when any signal field is set
  - `override_reason` is only valid when `next_action_outcome = overridden`
  - `first_useful_answer_command` is only valid when
    `first_useful_answer_at` is present
  - `first_useful_answer_at` is recorded only when a delight surface proposes a
    bounded next action and that next action is later accepted without an
    intervening override in the same session
  - `attributed_explanations <= total_explanations`
  - `accepted_next_actions` and `overridden_next_actions` must remain
    non-negative and together define the acceptance or override rate
- State transitions:
  - `not-recorded` -> `answer-recorded`
  - `answer-recorded` -> `accepted`
  - `answer-recorded` -> `overridden`
  - `answer-recorded` -> `not-applicable`

## Relationship Summary

- `ActiveSessionRecord` identifies the active workspace session and latest trace.
- `TraceSummaryView` provides the flattened operator evidence used by delight
  and inspect projections.
- `ProfileActivationRecord` feeds `ReasoningProfileDisclosure`.
- `TraceSummaryView` feeds `InspectClosureView`.
- Assistant manifests and host docs feed `HostParityDecision`.
- `ActiveSessionRecord` plus delight output events feed `DelightFeedbackSignal`.