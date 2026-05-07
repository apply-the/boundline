# Data Model: Guided CLI UX And Clearer Messaging

## Guided Init Prompt Surface

- Purpose: Represents the ordered operator-facing questions and inline guidance
  for `boundline init`.
- Key fields:
  - `prompt_headline`: short human-readable question label.
  - `accepted_values`: supported assistants, slots, or modes shown inline.
  - `default_behavior`: what Enter/blank input means.
  - `example_values`: at least one concrete valid example.
  - `follow_up_hint`: where to inspect or override the resulting value later.
- Validation rules:
  - Must not contradict clap help or post-init summaries.
  - Must be safe for plain-text terminals and remain understandable without
    color.

## Recovery Guidance Message

- Purpose: Represents a user-facing init or doctor failure outcome that helps an
  operator recover immediately.
- Key fields:
  - `invalid_input`: the exact value or state that failed.
  - `failure_class`: syntax, unsupported value, unavailable capability,
    overwrite conflict, or non-interactive limitation.
  - `expected_shape`: human-readable requirement such as
    `planning=copilot:gpt-5.4`.
  - `corrective_action`: example, retry command, or alternate supported value.
  - `exit_status`: non-success result preserved for automation.
- State transitions:
  - Generated from validation before mutation.
  - Ends in explicit stop until the operator reruns with corrected input.

## Effective Route Summary

- Purpose: Represents the final init summary for seeded and explicit routes.
- Key fields:
  - `assistant_selection`: chosen assistant runtimes.
  - `seeded_routes`: slots filled from defaults, including fallback provenance.
  - `explicit_routes`: operator-supplied overrides.
  - `remaining_defaults`: slots still owned by assistant defaults after partial
    overrides.
  - `inspection_hint`: where to inspect/edit the effective configuration.
- Validation rules:
  - Must make blank-input/default behavior obvious after success.
  - Must preserve route owner semantics and not imply silent hidden choices.

## Assistant Setup Summary

- Purpose: Represents the repository-local assistant asset scaffolding status
  emitted by init preview and apply mode.
- Key fields:
  - `surface`: Claude, Codex, Copilot, Gemini, or shared assistant docs.
  - `status`: created, updated, unchanged, or skipped.
  - `created_files`, `updated_files`, `unchanged_files`: per-surface counts.
  - `preview_action`: scaffold vs refresh messaging in preview mode.
- Validation rules:
  - Must remain bounded to files under the active repository root.
  - Must participate in safe rerun preview behavior before overwrite.

## Diagnostic Output Section

- Purpose: Represents a semantically grouped section in init or doctor output.
- Key fields:
  - `kind`: onboarding, capability, warning, success, or next step.
  - `lines`: ordered plain-text-safe content.
  - `rich_emphasis`: optional terminal-capable decoration.
- Validation rules:
  - Must preserve the same meaning across rich and plain text modes.
  - Must keep stdout/stderr semantics and grep-friendly wording intact.