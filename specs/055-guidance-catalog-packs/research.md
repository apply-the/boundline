# Research: Guidance Catalog And Guardian Rule Packs

## Decision 1: Use typed TOML-backed models for catalog manifests and indexes

- **Decision**: Represent pack manifests, catalog manifests, guidance entries, and guardian rule seeds with typed Rust structs and enums rather than ad hoc `toml::Value` traversal.
- **Rationale**: The repository constitution and Rust language rules require explicit, typed models for stable serialized shapes. Typed models also make validation errors, tracing, and precedence logic easier to express and test.
- **Alternatives considered**:
  - Ad hoc `toml::Value` parsing: rejected because it hides field-level contracts and invites magic strings.
  - JSON-only manifests: rejected because the repo already uses TOML for human-editable configuration and pack metadata.

## Decision 2: Split pack metadata into `pack.toml`, `catalog-manifest.toml`, `guidance-index.toml`, and `guardian-index.toml`

- **Decision**: Keep identity and compatibility in `pack.toml`, catalog-level defaults and layout in `catalog/catalog-manifest.toml`, and entry declarations in separate guidance and guardian indexes.
- **Rationale**: This preserves a clean boundary between pack identity, catalog defaults, and entry-level declarations, which makes selective validation and future Canon promotion easier.
- **Alternatives considered**:
  - One combined manifest file: rejected because it becomes hard to validate and reason about as the entry set grows.
  - Per-entry standalone manifests only: rejected because aggregate catalog discovery and precedence inspection become expensive and opaque.

## Decision 2A: Absorb the former Phase 7 addendum into canonical 055 references

- **Decision**: Keep the normalized, contract-conformant references under `specs/055-guidance-catalog-packs/reference/` and the shipped pack under `assistant/packs/guidance-catalog/` as the authoritative surviving artifacts for 055; the former roadmap addendum is no longer required as a separate repo source tree.
- **Rationale**: The phase-7 design input captured the intended packaging/indexing model and example shapes, but some example values used pre-canonical taxonomy or vocabulary. Normalizing them into 055-owned references and the shipped pack preserves the design intent while giving implementation and tests a stable target without duplicate source trees.
- **Normalization audit**: The 055 references intentionally convert legacy `recommendation` strength labels to `recommended`, expand abbreviated pillar names such as `domain` and `operations` to the canonical 055 taxonomy, and keep inline pack content declarations out of `pack.toml` so the contracts stay split across `catalog-manifest.toml`, `guidance-index.toml`, and `guardian-index.toml`.
- **Alternatives considered**:
  - Keep the roadmap addendum beside the normalized references: rejected because once the contracts were normalized and shipped, it became duplicate, drift-prone source material.
  - Ignore the former phase-7 input entirely: rejected because it would throw away the roadmap-derived design rationale captured in the normalized references.

## Decision 3: Own the canonical pillar taxonomy and metadata vocabulary in 055

- **Decision**: Make 055 the authoritative source for pillar taxonomy, guidance strength values, guardian disposition values, and authority metadata expected by pack content.
- **Rationale**: The moved scope is about content packaging and classification, not S2.1 runtime execution. Keeping taxonomy and vocabulary here lets S2.1 consume a stable catalog contract without absorbing content-authoring scope.
- **Alternatives considered**:
  - Keep taxonomy in S2.1: rejected because it makes 054 harder to close and mixes runtime behavior with catalog authoring concerns.
  - Scatter taxonomy across example files only: rejected because examples are insufficient as a stable contract surface.

## Decision 4: Validation findings must be explicit and trace-visible

- **Decision**: Invalid packs, entries, lifecycle labels, unsupported guardian kinds, missing files, and precedence conflicts produce explicit warnings or errors that can be persisted and surfaced through inspect surfaces.
- **Rationale**: Hidden fallback or silent skipping violates the constitution's observability and no-hidden-intelligence rules. Operators need a direct explanation for why a pack or entry was not used.
- **Alternatives considered**:
  - Best-effort silent skipping: rejected because it makes catalog behavior untrustworthy.
  - Hard-fail the entire runtime on any invalid entry: rejected because one bad entry should not necessarily prevent loading all other valid content.

## Decision 5: Guidance content must remain runtime-agnostic for Canon promotion compatibility

- **Decision**: Guidance markdown and guardian rule seed files must avoid embedding Boundline-specific runtime semantics so the same content can be promoted into Canon-governed standards without rewriting the file body.
- **Rationale**: The catalog track should support stronger future authority without forcing content duplication or format migration.
- **Alternatives considered**:
  - Embed runtime-only directives in markdown: rejected because it couples content authoring to one runtime implementation.
  - Duplicate separate Canon-ready files: rejected because it creates drift and maintenance overhead.

## Decision 6: Provider-model catalog remains unchanged for this slice

- **Decision**: Keep `assistant/catalog/model-catalog.toml` unchanged for 055 and record an explicit no-change rationale.
- **Rationale**: 055 changes guidance catalog packaging and validation, not provider routing or model selection. The repository constitution still requires the audit step, so this feature records a no-change result rather than expanding functional scope.
- **Audit result (2026-05-15)**: Re-checked the public OpenAI, Anthropic, and Google model-overview pages. The bundled catalog already covers the currently relevant GPT-5.5 / GPT-5.4 family, GPT-5 Codex, Claude Opus 4.7 / Sonnet 4.6 / Haiku 4.5 family, and Gemini 2.5 / 3.1 family identifiers referenced by this workspace. No 055-specific routing delta is required beyond the release-version metadata updates performed elsewhere in the branch.
- **Alternatives considered**:
  - Skip the audit entirely: rejected because the constitution requires it for every feature.
  - Expand 055 to provider routing work: rejected because that would blur feature boundaries and delay delivery.