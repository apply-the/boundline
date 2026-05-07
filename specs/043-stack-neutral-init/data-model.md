# Data Model: Stack-Neutral Workspace Entry

## Workspace Entry Assessment

- Purpose: Represents whether a local directory can enter the primary Boundline workflow.
- Fields:
  - `workspace_ref`: absolute workspace path
  - `exists`: whether the directory exists
  - `writable`: whether local state can be persisted
  - `trace_store_ready`: whether `.boundline/traces/` is usable or creatable
  - `execution_profile_state`: ready, missing, ignored, or invalid
  - `readiness_status`: ready or blocked
  - `blocking_reasons`: explicit reasons when readiness fails
- Relationships:
  - Feeds native direct-run and `doctor --workspace` output.
  - Remains separate from stack or domain credibility, which planning resolves later.

## Assistant Target Catalog Entry

- Purpose: Declares the default model Boundline should use for one supported assistant runtime.
- Fields:
  - `runtime`: `claude`, `copilot`, `codex`, or `gemini`
  - `default_model`: repository-managed model identifier
  - `source`: built-in catalog or later override source
- Relationships:
  - Used by init-time route seeding.
  - Must stay aligned with effective built-in routing defaults.

## Seeded Route Selection

- Purpose: Captures which route slots were filled automatically during initialization.
- Fields:
  - `slot`: planning, implementation, verification, or review
  - `runtime`: selected assistant runtime
  - `model`: selected model identifier
  - `selection_source`: explicit override, assistant default, or built-in fallback through selected assistants
- Relationships:
  - Persisted into workspace routing config.
  - Projected through init output and later config inspection.

## Hygiene Defaults Profile

- Purpose: Represents the bounded set of ignore and hygiene defaults that Boundline may seed for a workspace.
- Fields:
  - `universal_patterns`: defaults that apply regardless of stack
  - `technology_patterns`: defaults keyed by domain family
  - `tool_patterns`: defaults keyed by detected repository tools
  - `eligible_files`: target files such as `.gitignore`, `.dockerignore`, `.eslintignore`, `.prettierignore`, `.terraformignore`, `.helmignore`
  - `provenance`: why each pack was selected, skipped, or blocked
- Relationships:
  - Derived from selected or detected domain families plus repository cues.
  - Applied merge-only so local overrides survive repeated initialization.
