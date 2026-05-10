# Data Model: Guided Init TUI and Runtime Catalog

## Overview

This slice does not add a new persistent workspace record. It introduces a
bounded in-memory interaction model for one `boundline init` run, plus a bundled
runtime/model catalog used to propose and validate route defaults before the
existing config and execution files are written.

## Entities

### InitInteractionState

- Purpose: Represent the current guided init run before any write occurs.
- Fields:
  - `workspace_ref`: target workspace path for bootstrap.
  - `canon_mode_selection`: pending Canon approval choice.
  - `assistant_surfaces`: selected assistant runtimes.
  - `route_table`: current per-slot route drafts.
  - `current_step`: active guided step or review screen.
  - `validation_message`: contextual correction message for the active step when present.
  - `write_preview`: summary of files and assets that would be created or updated.
- Lifecycle:
  - Created when `boundline init` enters guided mode.
  - Mutated step-by-step as the operator confirms choices.
  - Discarded on cancellation or after confirmed writes complete.

### BundledModelCatalog

- Purpose: Provide repository-managed runtime/model presets and default-route seeding metadata for guided init.
- Source Asset: `assistant/catalog/model-catalog.toml`, compiled into the CLI for offline use.
- Fields:
  - `source_label`: human-readable catalog source, such as `bundled`.
  - `catalog_version`: catalog revision identifier or release tag.
  - `updated_at`: displayable timestamp for the bundled catalog revision.
  - `runtime_entries`: grouped runtime/model choices available to guided init.
  - `default_routes`: default route suggestions derived from supported assistant selections.
- Lifecycle:
  - Loaded from the bundled asset when init starts.
  - Read-only for the lifetime of the command.
  - Updated only by repository changes in later releases.

### Catalog TOML Format

The bundled asset uses a human-reviewable TOML layout so catalog revisions can
ship with the repository while remaining easy to inspect.

```toml
[metadata]
source_label = "bundled"
catalog_version = "0.47.0"
updated_at = "2026-05-09"

[[runtimes]]
runtime = "copilot"
display_name = "GitHub Copilot"

[[runtimes.models]]
model_id = "gpt-5.4"
display_name = "GPT-5.4"

[[runtimes.models]]
model_id = "gpt-5.4-mini"
display_name = "GPT-5.4 Mini"

[[runtimes]]
runtime = "claude"
display_name = "Claude"

[[runtimes.models]]
model_id = "sonnet-4.5"
display_name = "Sonnet 4.5"

[default_routes]
planning = { runtime = "copilot", model_id = "gpt-5.4" }
implementation = { runtime = "claude", model_id = "sonnet-4.5" }
verification = { runtime = "copilot", model_id = "gpt-5.4" }
review = { runtime = "claude", model_id = "sonnet-4.5" }
```

Rules:

- `metadata.source_label`, `metadata.catalog_version`, and `metadata.updated_at`
  are required.
- Every supported runtime (`claude`, `codex`, `copilot`, `gemini`) must have at
  least one bundled model entry.
- `default_routes` is optional as a whole, but when present it may only contain
  the four required slot names.
- Catalog entries may evolve across releases without changing the stored route
  model, which remains `runtime + model_id`.

### CatalogRuntimeEntry

- Purpose: Describe the selectable models for one assistant runtime.
- Fields:
  - `runtime`: runtime identifier such as `copilot` or `claude`.
  - `display_name`: operator-facing runtime label.
  - `models`: bundled model choices for that runtime.
  - `default_slots`: optional preferred slots or route seeds for that runtime.
- Constraints:
  - Runtime identifiers must align with the supported assistant surfaces already used by Boundline config.

### CatalogModelEntry

- Purpose: Describe one bundled model choice within a runtime.
- Fields:
  - `model_id`: stable identifier stored in route config.
  - `display_name`: operator-facing label.
  - `status`: bundled or deprecated state if needed later.
  - `notes`: optional operator-facing explanation.
- Constraints:
  - `model_id` must be non-empty and stable across one release.
  - Display labels may change without changing stored route semantics.

### RouteDraft

- Purpose: Represent the in-progress choice for one required route slot.
- Fields:
  - `slot`: one of `planning`, `implementation`, `verification`, or `review`.
  - `runtime`: selected runtime identifier.
  - `model_id`: bundled or custom model identifier.
  - `origin`: `bundled`, `assistant-default`, or `custom`.
  - `is_complete`: whether the slot has a usable selection.
  - `warning`: optional warning such as `custom model id is unverified`.
- Lifecycle:
  - Initialized from bundled defaults or left unset.
  - Updated during slot edits.
  - Serialized into existing routing config only after final confirmation.

### AssistantAssetPlan

- Purpose: Summarize which repository-managed assistant assets will be scaffolded
  or refreshed for the selected assistant surfaces.
- Fields:
  - `surface`: assistant surface identifier.
  - `source_paths`: bundled source asset paths under `assistant/`.
  - `created_files`: number of assets newly written to the workspace.
  - `updated_files`: number of assets refreshed in the workspace.
  - `unchanged_files`: number of assets already matching the bundled contents.
- Lifecycle:
  - Derived after assistant-surface selection and before final write summary.
  - Rendered in preview and final status output.

### InitSummary

- Purpose: Present the final review screen before Boundline writes any files.
- Fields:
  - `canon_mode_selection`
  - `assistant_surfaces`
  - `route_table`
  - `catalog_source`
  - `warnings`
  - `planned_changes`
- Constraints:
  - Must be fully derivable from the current `InitInteractionState`.
  - Must remain stable enough for integration and contract assertions.

### ProgressActivity

- Purpose: Describe a bounded init step that should surface visible progress.
- Fields:
  - `label`: operator-facing activity text.
  - `interactive_mode`: whether spinner-based rendering is allowed.
  - `started_at`: time the activity began.
  - `terminal_state`: success, failure, or cancellation outcome.
- Constraints:
  - One progress activity may be rendered at a time.
  - Spinner frames must never leak into non-interactive output.

## Relationships

- `InitInteractionState` owns zero or more `RouteDraft` entries, one per slot.
- `BundledModelCatalog` owns `CatalogRuntimeEntry` records, which in turn own `CatalogModelEntry` records.
- `InitSummary` is a read-only projection of `InitInteractionState` plus catalog metadata.
- `ProgressActivity` is attached to specific init sub-steps such as catalog loading, file writes, or assistant asset seeding.
- `AssistantAssetPlan` is derived from the selected assistant surfaces and the existing repository-managed `assistant/` asset inventory.

## Validation Rules

- Guided init must show all four route slots during review, even when one or more slots remain unset.
- Guided init may commit with unset slots only when the summary explicitly marks them unset and the operator confirms that outcome.
- A `RouteDraft` with `origin = custom` must retain its runtime and show a visible warning before confirmation.
- Empty assistant selections are valid only when the resulting route table and summary remain explicit about the lack of assistant-seeded defaults.
- Cancellation before final confirmation must drop the entire `InitInteractionState` without partial writes.
- Non-interactive init bypasses step-by-step interaction but must still produce a valid `InitSummary` equivalent internally before writes proceed.
- If guided interaction is unavailable because the command lacks a usable TTY, init must either operate entirely from non-interactive inputs or fail with explicit guidance to rerun with `--non-interactive`.
