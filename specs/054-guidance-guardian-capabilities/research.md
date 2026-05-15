# Research: Guidance And Guardian Capabilities

## Current Implementation Surfaces

- `src/orchestrator/goal_planner.rs` already resolves bounded planning context, domain families, optional Canon inputs, and expert-pack outcomes before planning continues.
- `src/domain/domain_templates.rs` already carries domain-template guidance metadata and bounded context credibility signals that can anchor guidance-source projection.
- `src/domain/configuration.rs` already owns effective route resolution for planning, implementation, verification, and review, which is the correct routing surface for semantic guardians.
- `src/orchestrator/session_runtime.rs`, `src/domain/trace.rs`, `src/cli/session.rs`, and `src/cli/output.rs` already persist and project bounded runtime state through `status`, `next`, and `inspect`.
- The repository already keeps assistant-facing assets under `assistant/`, while workspace-local runtime inputs live under `.boundline/` and Canon knowledge remains repo-visible rather than runtime-owned.

## Decision 1: Store built-in shared capabilities as repository-managed assistant assets

- **Decision**: Introduce built-in shared guidance and guardian assets under `assistant/guidance/`, `assistant/guardians/`, and `assistant/packs/`, while keeping workspace overrides under `.boundline/guidance/` and `.boundline/guardians/`.
- **Rationale**: This keeps shared capability content repo-managed and inspectable, matches the repo's existing assistant-asset ownership boundary, and preserves the local-first model required by the spec.
- **Alternatives considered**:
  - Use `roadmap/` as the runtime source: rejected because roadmap drafts are specification inputs, not stable runtime assets.
  - Hardcode all guidance and guardian content in Rust modules: rejected because it hides operator-visible content in compiled code and makes updates noisier.
  - Introduce a new top-level `packs/` runtime surface: rejected because the feature can stay smaller by reusing `assistant/` plus `.boundline/`.

## Decision 2: Introduce typed capability and finding models in the domain layer

- **Decision**: Add typed Rust models for guidance manifests, guardian manifests, authority sources, resolution outcomes, execution records, and structured findings in a dedicated domain module, then persist projections through existing goal-plan and trace surfaces.
- **Rationale**: The repository constitution requires typed serialized shapes and explicit, inspectable control flow. Typed models also make resolution and projection rules deterministic and testable.
- **Alternatives considered**:
  - Emit ad hoc JSON payloads directly in traces: rejected because stable runtime shapes must not be modeled as raw maps or repeated field-name strings.
  - Keep findings as CLI-only formatted text: rejected because `status`, `next`, `inspect`, and downstream governance need reusable structured state.
  - Store all capability state only in session views: rejected because planning also needs persisted resolution context before execution begins.

## Decision 3: Reuse existing runtime routing for semantic guardians

- **Decision**: Map `llm` and `hybrid` guardians onto existing Boundline routing slots and degrade explicitly when no suitable route is available, instead of introducing a guardian-specific slot or provider-selection layer.
- **Rationale**: The feature scope explicitly excludes model catalog and provider-readiness management. Existing effective routing already resolves planning, implementation, verification, and review slots and is the smallest credible routing owner for this slice.
- **Alternatives considered**:
  - Add a new guardian routing slot: rejected because it broadens configuration scope without being necessary for the first slice.
  - Choose a provider directly inside guardian execution: rejected because it hides routing decisions and duplicates configuration logic.
  - Silently fall back to deterministic-only behavior when routing is missing: rejected because the constitution forbids invisible fallback behavior.

## Decision 4: Project capability resolution and findings through existing read-side surfaces

- **Decision**: Extend goal-plan, trace, and session-native projection surfaces so `status`, `next`, and `inspect` explain loaded sources, skipped sources, guardian order, findings, and degraded outcomes without adding a new top-level CLI command.
- **Rationale**: The primary operator workflow is session-native. Reusing those surfaces keeps the feature bounded, inspectable, and aligned with current operator expectations.
- **Alternatives considered**:
  - Add a dedicated `boundline guardian` command: rejected because it creates a second operator story for the same runtime state.
  - Write separate report files outside traces: rejected because it would split authoritative read-side state.
  - Recompute findings or source resolution on every read-side command: rejected because persisted state is more explicit and stable.

## Decision 5: Keep Canon as an optional authority source, not a required runtime dependency

- **Decision**: Treat Canon-governed standards as an optional governed authority that can enrich or override shared built-ins according to declared precedence, but never make Canon availability a precondition for guidance resolution or guardian execution.
- **Rationale**: This preserves the agreed Canon-aware, not Canon-dependent boundary and keeps the feature independently testable without Canon runtime control flow.
- **Alternatives considered**:
  - Require Canon artifacts for guardian calibration: rejected because it would violate the local-first runtime boundary.
  - Ignore Canon entirely in the first slice: rejected because the spec explicitly includes Canon-governed standards in precedence and authority disclosure.

## Decision 6: Retain a no-change provider-doc audit in planning only

- **Decision**: Record a no-change provider-doc audit in planning artifacts to satisfy the repository constitution, while keeping model catalog and provider readiness outside the feature's functional scope.
- **Rationale**: The constitution currently requires provider-doc review for every feature. The audit can stay in planning as a compliance note without polluting the spec or runtime design.
- **Alternatives considered**:
  - Reintroduce model catalog coverage into the feature spec: rejected because it dilutes the S2.1 scope.
  - Skip the audit entirely: rejected because it would fail the current constitution and plan gate.

## Provider-Doc Audit

- Reviewed current OpenAI models and Codex docs, Anthropic Claude models overview, and Google Gemini models docs on 2026-05-15.
- OpenAI still documents `gpt-5.5` as the flagship model, keeps `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano` on the current models page, and still exposes Codex as the active coding surface; no obvious rename or removal requires a bundled OpenAI catalog change for this slice.
- Anthropic still documents `claude-opus-4-7`, `claude-sonnet-4-6`, and `claude-haiku-4-5` as current API aliases, while still referencing the `4.6` and `4.x` migration lines that match the bundled legacy entries; no bundled Claude catalog change is required for this slice.
- Google Gemini still documents `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite`, `gemini-3.1-pro-preview`, `gemini-3-flash-preview`, `gemini-3.1-flash-lite`, and `gemini-3.1-flash-lite-preview`; no bundled Gemini catalog change is required for this slice.
- This audit exists only to satisfy the repository constitution and does not expand the feature scope beyond existing runtime routing.

## Likely Touchpoints

- `assistant/guidance/`
- `assistant/guardians/`
- `assistant/packs/`
- `src/domain/domain_templates.rs`
- `src/domain/configuration.rs`
- `src/domain/goal_plan.rs`
- `src/domain/guidance.rs`
- `src/domain/trace.rs`
- `src/orchestrator/goal_planner.rs`
- `src/orchestrator/guidance_runtime.rs`
- `src/orchestrator/session_runtime.rs`
- `src/cli/output.rs`
- `src/cli/session.rs`
- `tests/unit/`
- `tests/integration/`
- `tests/contract/`
- `README.md`
- `docs/architecture.md`
- `docs/configuration.md`
- `docs/getting-started.md`
- `roadmap/S2-1 - guidance-and-guardian-capabilities.md`
- `CHANGELOG.md`
- `Cargo.toml`
- `AGENTS.md`
