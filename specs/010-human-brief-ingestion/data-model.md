# Data Model: Human-Facing Brief Ingestion

## ExternalTaskInput

- Purpose: Captures the raw user-facing request before Synod resolves files, deduplicates sources, or blocks for clarification.
- Fields:
  - `input_id`: Stable identifier for the captured input attempt.
  - `origin_channel`: `cli` or `assistant`.
  - `goal_text`: Optional direct text supplied by the operator.
  - `explicit_brief_paths`: Ordered list of Markdown paths supplied explicitly on the command surface.
  - `referenced_brief_paths`: Ordered list of Markdown paths discovered in direct text and accepted as candidate workspace references.
  - `governance_intent`: Optional `GovernanceIntent` expressed in business terms.
  - `captured_at`: Millisecond timestamp for the input attempt.
- Validation rules:
  - At least one of `goal_text`, `explicit_brief_paths`, or `referenced_brief_paths` must be present.
  - `goal_text`, when present, must be non-empty after trimming.
  - File paths must be canonicalizable against the active workspace before they can move into normalized state.
  - `governance_intent` may be omitted for ungoverned execution.

## InputSourceReference

- Purpose: Represents one accepted authored source after workspace-bound resolution and deduplication.
- Fields:
  - `source_id`: Stable identifier inside the normalized bundle.
  - `kind`: `direct_text`, `attached_markdown`, or `referenced_markdown`.
  - `workspace_path`: Optional canonical workspace-relative path for file-backed sources.
  - `display_name`: Human-readable name shown in status or inspect output.
  - `precedence`: Zero-based accepted order after deduplication.
  - `content_hash`: Optional content hash for file-backed sources.
  - `deduplicated_from`: Optional list of raw source identifiers merged into this accepted source.
  - `captured_excerpt`: Optional short excerpt used only for direct-text summaries.
- Validation rules:
  - `precedence` values must be unique and contiguous within one bundle.
  - File-backed sources must resolve to Markdown files within the workspace boundary.
  - `deduplicated_from` must never include the accepted `source_id` itself.
  - `content_hash` is required for file-backed sources once the file is accepted.

## AuthoredBriefBundle

- Purpose: Stores the normalized, inspectable human-authored input that later planning and execution steps consume.
- Fields:
  - `bundle_id`: Stable identifier referenced by session and trace projections.
  - `summary`: Short bounded statement of the accepted task intent.
  - `sources`: Ordered list of `InputSourceReference` values.
  - `governance_intent`: Optional normalized `GovernanceIntent`.
  - `resolution_state`: `captured`, `clarification_required`, or `ready`.
  - `primary_goal_text`: Optional direct text retained as the highest-level task statement.
  - `clarification_ref`: Optional identifier of the blocking `ClarificationRecord`.
  - `captured_at`: Millisecond timestamp for the accepted bundle.
- Validation rules:
  - A bundle must contain at least one accepted source.
  - `summary` must be non-empty when `resolution_state = ready`.
  - `clarification_ref` is required when `resolution_state = clarification_required`.
  - `resolution_state = ready` is invalid while an open clarification exists.

## ClarificationRecord

- Purpose: Represents one explicit blocking question Synod must resolve before planning or execution can continue.
- Fields:
  - `clarification_id`: Stable identifier.
  - `reason_kind`: `missing_context`, `source_conflict`, `missing_source`, `unsupported_source`, or `unbounded_request`.
  - `prompt`: Human-facing clarification question.
  - `missing_fields`: Ordered list of external business values still needed.
  - `blocking_sources`: Ordered list of `source_id` values involved in the block.
  - `turn_index`: Clarification turn count starting at `1`.
  - `status`: `open`, `answered`, or `exhausted`.
- Validation rules:
  - Only one `ClarificationRecord` may be `open` for the active bundle at a time.
  - `turn_index` must stay within the first-slice limit of `1..=2`.
  - `prompt` must not mention internal stage IDs, manifest fields, or Canon packet wiring.
  - `status = exhausted` requires that planning remains blocked until new human input arrives.

## GovernanceIntent

- Purpose: Holds the human-facing declaration that governed execution is requested and the business values needed to support it.
- Fields:
  - `requested`: Whether governed execution was requested explicitly or implied by other governance business fields.
  - `runtime_preference`: Optional `local` or `canon` hint.
  - `risk`: Optional business risk label.
  - `zone`: Optional governance zone.
  - `owner`: Optional owner or responsible team.
- Validation rules:
  - `requested = false` requires every other field to be empty.
  - If any of `risk`, `zone`, or `owner` is provided, `requested` becomes true.
  - `runtime_preference`, when present, must remain a human-facing runtime choice and never expose stage IDs or Canon modes.

## DerivedTaskDraft

- Purpose: Represents the bounded planning seed Synod derives from the normalized brief bundle before execution begins.
- Fields:
  - `draft_id`: Stable identifier for the derived task attempt.
  - `bundle_id`: Parent `AuthoredBriefBundle` identifier.
  - `bounded_goal`: Delivery-focused task statement ready for flow selection and planning.
  - `flow_hint`: Optional `bug-fix`, `change`, or `delivery` hint.
  - `planning_ready`: Whether the draft can proceed into normal planning.
  - `validation_targets`: Ordered list of files or commands that bound later validation.
  - `governance_overlay`: Optional normalized governance overlay derived from `GovernanceIntent`.
  - `blocking_clarification_ref`: Optional open clarification preventing planning.
- Validation rules:
  - `planning_ready = true` requires `bounded_goal` to be non-empty and `blocking_clarification_ref` to be absent.
  - `flow_hint`, when present, must be one of the existing built-in flows.
  - `governance_overlay` may exist only when governance was requested or derived from the accepted bundle.

## Relationships

- One `ExternalTaskInput` becomes one `AuthoredBriefBundle` after workspace-bound resolution.
- One `AuthoredBriefBundle` contains one or more `InputSourceReference` values.
- One `AuthoredBriefBundle` may produce zero or one open `ClarificationRecord` at a time.
- One `AuthoredBriefBundle` may include zero or one `GovernanceIntent`.
- One `AuthoredBriefBundle` produces at most one active `DerivedTaskDraft` for the current session revision.

## Persistence Notes

- The normalized `AuthoredBriefBundle`, any `ClarificationRecord`, and the resulting `DerivedTaskDraft` should live with the active task and session state rather than in a new user-authored manifest.
- Session and inspect projections should expose summaries and provenance derived from these records without forcing the user to read raw JSON.