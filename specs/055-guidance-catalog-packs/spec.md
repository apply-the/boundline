# Feature Specification: Guidance Catalog And Guardian Rule Packs

**Feature Branch**: `055-guidance-catalog-packs`  
**Created**: 2026-05-15  
**Status**: Draft  
**Input**: User description: "Define the catalog packaging, pillar taxonomy, authority metadata, indexing, validation, and manifest model for guidance and guardian content so that Boundline can consume structured, versioned guidance packs rather than loose Markdown collections"

## Relationship To S2.1

S2.1 (054-guidance-guardian-capabilities) defines runtime behavior for guidance and guardians: loading, resolving, executing, tracing, and exposing.

This specification defines how guidance catalog content is packaged, indexed, classified, versioned, and exposed to the Boundline runtime. It owns the catalog-side pillar taxonomy, strength and disposition vocabulary, manifest and index structure, and rule-pack validation model. It does not define guardian execution, finding emission, lifecycle integration, or Canon governance semantics.

## Guidance Catalog Awareness

This specification moves catalog content authoring and packaging concerns out of S2.1.

S2.1 remains responsible for runtime behavior: loading, resolving, applying, executing, tracing, and exposing guidance and guardian outcomes.

055 is responsible for the catalog surface that S2.1 consumes:

- pack-ready content structure
- catalog manifest and indexes
- canonical pillar taxonomy
- guidance strength vocabulary
- guardian disposition vocabulary
- authority metadata and Canon-promotion compatibility
- entry validation and examples

## Outcome

Boundline can consume guidance catalogs as installable packs rather than loose Markdown collections.

The catalog packaging model supports:

- manifest-based discovery
- canonical pillar-based organization
- guidance indexing by pillar, lifecycle phase, role, language, and framework
- guardian indexing by pillar, kind, rules, lifecycle phase, and disposition
- authority source metadata at entry and catalog level
- lifecycle activation metadata
- Canon promotion compatibility
- trace-friendly resolution
- schema-validated entry shapes

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install A Guidance Catalog Pack (Priority: P1)

As a Boundline operator, I want to install a guidance catalog pack so that Boundline can resolve guidance and guardians from a single declared package with manifest-based discovery.

**Why this priority**: Without a defined catalog shape, content remains unversioned loose files that cannot be reliably loaded, validated, or upgraded. This is the foundation for all other catalog stories.

**Independent Test**: Can be fully tested by preparing a guidance catalog pack with a valid `pack.toml`, `catalog/catalog-manifest.toml`, `catalog/guidance-index.toml`, and `catalog/guardian-index.toml`, then verifying that Boundline can enumerate available guidance and guardian entries from the catalog.

**Acceptance Scenarios**:

1. **Given** a guidance catalog pack with a valid pack manifest, catalog manifest, guidance index, and guardian index, **When** Boundline loads the pack, **Then** it discovers all indexed guidance entries and guardian rule seeds and makes them available for resolution.
2. **Given** a catalog pack where the guidance index references a guidance file that does not exist on disk, **When** Boundline loads the catalog, **Then** it records a structured load warning, skips the missing entry, and continues loading all other valid entries.
3. **Given** a catalog pack with missing or malformed `catalog-manifest.toml`, **When** Boundline attempts to load it, **Then** it rejects the pack with a structured error and does not load partial state.

---

### User Story 2 - Inspect Guidance Resolution From Catalog (Priority: P2)

As a Boundline operator, I want to inspect which guidance was selected from a catalog and why, so that I can trust the runtime context.

**Why this priority**: Resolution transparency is essential for operator trust. Without it, users cannot verify that the correct guidance influenced their session.

**Independent Test**: Can be fully tested by loading a catalog pack alongside a workspace override for the same concern, running a session, and verifying that the resolution trace records authority source, strength, selected entry, and override decisions.

**Acceptance Scenarios**:

1. **Given** a loaded catalog pack, **When** Boundline resolves guidance for a session, **Then** the resolution trace records the authority source, strength, and selected pillar for each resolved entry.
2. **Given** a catalog pack entry and a workspace override for the same concern, **When** Boundline resolves guidance, **Then** the resolution trace records both sources and the override decision.
3. **Given** a catalog pack entry that declares Canon-governed authority, **When** no Canon artifact is available for validation, **Then** the trace records that Canon-governed authority could not be confirmed and local resolution was applied.

---

### User Story 3 - Promote Guidance Through Canon (Priority: P3)

As a governance owner, I want selected guidance files from a catalog pack to be promotable into Canon-governed standards, so that shared recommendations can become project authority without content duplication.

**Why this priority**: Canon promotion compatibility ensures that catalog content can grow in authority over time without requiring rewriting. This depends on catalog loading (US1) being functional.

**Independent Test**: Can be fully tested by verifying that guidance files do not depend on Boundline runtime internals, that Canon promotion metadata is documented, and that Boundline can later resolve Canon-promoted versions of previously pack-provided guidance as higher-authority sources.

**Acceptance Scenarios**:

1. **Given** a guidance catalog entry, **When** it is promoted to a Canon-governed standard, **Then** the content file does not require modification because it does not embed Boundline runtime references.
2. **Given** a previously pack-provided guidance entry that has been promoted to Canon-governed status, **When** Boundline resolves guidance, **Then** it resolves the Canon-governed version at higher authority per resolution-strength precedence.
3. **Given** a catalog manifest with `canon_promotable = true`, **When** a governance owner reviews the catalog, **Then** the promotion metadata describes which fields and paths are preserved during promotion.

---

### User Story 4 - Validate Catalog Shape (Priority: P2)

As a pack maintainer, I want catalog entries to follow a defined schema so that invalid packs fail clearly at load time rather than causing silent runtime errors.

**Why this priority**: Schema validation prevents malformed catalogs from degrading runtime behavior. It depends on the catalog shape being defined (US1).

**Independent Test**: Can be fully tested by preparing catalogs with invalid entries (missing required fields, unsupported lifecycle labels, unsupported guardian kinds, missing referenced files) and verifying that validation produces specific, actionable error messages.

**Acceptance Scenarios**:

1. **Given** a guidance entry with a missing `pillar` field, **When** validation runs, **Then** it rejects the entry with a message identifying the missing field and entry ID.
2. **Given** a guardian entry with an unsupported `kind` value, **When** validation runs, **Then** it rejects the entry with a message identifying the invalid kind and supported alternatives.
3. **Given** a guidance index that references a lifecycle label not in the supported set, **When** validation runs, **Then** it rejects the entry with a message identifying the unsupported label.
4. **Given** a valid catalog pack with all required fields and valid references, **When** validation runs, **Then** it passes without errors or warnings.

### Edge Cases

- A catalog pack declares compatibility with a Boundline version newer than the installed runtime: Boundline records a compatibility warning and loads the catalog in best-effort mode.
- A guidance index contains duplicate IDs: validation rejects the catalog with an explicit duplicate-ID error.
- A guardian index references a `requires_guidance` entry that does not exist in the guidance index: validation records a warning but does not reject the pack (the guidance may come from another source).
- A catalog manifest declares `canon_promotable = true` but individual entries override with non-promotable metadata: per-entry metadata takes precedence over manifest-level defaults.
- A catalog pack contains no guardian entries: valid; guidance-only packs are allowed.
- A catalog pack contains no guidance entries: valid; guardian-only packs are allowed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A guidance catalog pack MUST include a pack manifest (`pack.toml`) declaring identity, version, kind, description, and compatibility metadata.
- **FR-002**: A guidance catalog pack MUST include a catalog manifest (`catalog/catalog-manifest.toml`) declaring authority defaults, layout, included pillars, runtime requirements, and trace configuration.
- **FR-003**: A catalog MUST provide a guidance index (`catalog/guidance-index.toml`) declaring all guidance entries with required metadata.
- **FR-004**: A catalog MUST provide a guardian index (`catalog/guardian-index.toml`) declaring all guardian rule seed entries with required metadata.
- **FR-005**: Guidance index entries MUST include path, pillar, strength, applicable lifecycle phases, and consuming roles.
- **FR-006**: Guardian index entries MUST include pillar, kind, rules, applicable lifecycle phases, and default disposition.
- **FR-007**: Boundline MUST record authority source and strength when resolving catalog entries, exposing this in the resolution trace.
- **FR-008**: Boundline MUST expose catalog entries through inspect surfaces (status, next, inspect CLI commands).
- **FR-009**: Catalog content MUST be usable as built-in, shared pack, workspace override, or Canon-promoted standard without rewriting the content files.
- **FR-010**: Invalid catalog entries MUST produce explicit load findings or warnings with entry ID and field-level detail rather than failing silently or crashing.
- **FR-011**: Catalog packs MUST classify guidance and guardian entries using the canonical Guidance Pillar Taxonomy defined in this specification.
- **FR-012**: Guidance entries MUST use only supported Guidance Strength Classification values defined in this specification.
- **FR-013**: Guardian entries MUST use only supported Guardian Disposition Classification values defined in this specification.
- **FR-014**: Catalog metadata MUST expose authority source information at the catalog level, entry level, or both, sufficient for runtime trace disclosure and severity calibration.

### Scope Boundaries *(mandatory)*

- **In Scope**: catalog manifest schema; guidance index schema; guardian index schema; pack layout conventions; entry-level validation; authority source metadata; strength and disposition classification; lifecycle activation metadata; Canon promotion compatibility; guidance-only and guardian-only pack support; version compatibility declarations.
- **Out of Scope**: guardian execution (owned by S2.1/054); review councils or voting systems (deferred to S3); governance escalation (deferred to S4); semantic retrieval of guidance content (deferred to S5); reasoning profiles (deferred to S6); Canon publication or promotion implementation (Canon-owned); writing all guidance content for every language, framework, or domain.

### Key Entities

- **Catalog Manifest**: A TOML file declaring catalog identity, version, compatibility, authority defaults, layout paths, included pillars, runtime requirements, and trace configuration. Located at `catalog/catalog-manifest.toml`.
- **Guidance Index**: A TOML file mapping guidance entry IDs to file paths, pillars, strengths, lifecycle phases, roles, and optional language/framework metadata. Located at `catalog/guidance-index.toml`.
- **Guardian Index**: A TOML file mapping guardian rule seed IDs to pillars, kinds, rule lists, lifecycle phases, default dispositions, and optional language/framework metadata. Located at `catalog/guardian-index.toml`.
- **Pack Manifest**: A TOML file declaring pack identity, version, kind (`guidance-pack`), description, and compatibility. Located at `pack.toml` in the pack root.
- **Pack Layout**: The directory structure convention for guidance catalog packs: `pack.toml`, `catalog/`, `guidance/`, `guardians/`, optional `schemas/`, optional `examples/`.

## Pillar Taxonomy

This specification defines the canonical Guidance Pillar Taxonomy consumed by guidance catalog packs and by S2.1 runtime resolution:

- `clean-code`, `architecture`, `testing`, `language`, `framework`, `security`
- `domain-language`, `domain-modeling`, `api-contracts`, `migration`
- `observability`, `resilience`, `operations-readiness`, `supply-chain`
- `data-ai`, `optional-ecosystem`

Catalog packs may include entries for any subset of these pillars.

## Guidance Strength Classification

Supported strength values for guidance entries:

- `mandatory`: must be followed; violations produce blocking findings
- `recommended`: should be followed; violations produce advisory findings
- `legacy-warning`: flagging legacy patterns that should be migrated
- `target-excellence`: aspirational quality targets
- `anti-pattern`: known harmful patterns to avoid
- `deprecated`: content superseded by newer guidance

## Guardian Disposition Classification

Supported disposition values for guardian findings:

- `info`: informational observation, no action required
- `observation`: notable pattern worth awareness
- `concern`: potential issue meriting attention
- `warning`: likely issue that should be addressed
- `risk`: significant issue with delivery or quality implications
- `blocker`: issue that should prevent progression
- `error`: execution failure or invalid state

## Lifecycle Activation Labels

Supported lifecycle labels for `applies_to` declarations:

- `planning`, `system-shaping`, `architecture`, `backlog`
- `implementation`, `testing`, `verification`, `review`
- `refactor`, `migration`, `incident`, `supply-chain-analysis`

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A catalog pack with valid manifest, guidance index, and guardian index can be loaded by Boundline and its entries enumerated.
- **SC-002**: Boundline can list guidance entries by pillar, lifecycle phase, language, or framework from a loaded catalog.
- **SC-003**: Boundline can list guardian entries by pillar, kind, lifecycle phase, language, or framework from a loaded catalog.
- **SC-004**: Resolution traces show which source was selected and why when multiple sources provide guidance for the same concern.
- **SC-005**: Canon-promoted versions of a guidance file can be resolved as higher-authority sources without modifying the content file.
- **SC-006**: Invalid catalog entries (missing fields, unsupported labels, missing files) produce specific load errors or warnings that identify the entry and field.

## Non-Goals

This specification does not:

- define guardian execution behavior (owned by S2.1)
- define review council or voting systems (S3)
- define adaptive governance or trust degradation (S4)
- define semantic retrieval of guidance content (S5)
- write all guidance content for every language, framework, or domain
- define Canon publication or promotion implementation (Canon-owned)

## Assumptions

- The Boundline runtime (S2.1) provides the loading, resolution, and execution infrastructure that consumes catalog packs produced according to this specification.
- The Guidance Pillar Taxonomy is owned by this specification and consumed by S2.1 as a stable identifier set.
- Catalog packs are file-system artifacts (directories with TOML metadata and Markdown/TOML content) that do not require network access or package registry infrastructure for the first implementation slice.
- Guidance Markdown files are self-contained prose documents that do not embed Boundline runtime references, ensuring Canon promotion compatibility.
- Guardian rule seed TOML files declare rule metadata and structure but do not contain executable code; execution strategy is determined by the runtime based on guardian kind.

## Reference Material

The original Phase 7 roadmap inputs have been absorbed into the canonical 055 reference set and the shipped bundled pack. They are no longer required as a separate roadmap source tree.

Authoritative contract and example surfaces for this feature now live under `specs/055-guidance-catalog-packs/reference/` and `assistant/packs/guidance-catalog/`.

Canonical references for this specification are available at:

- `specs/055-guidance-catalog-packs/reference/catalog-manifest.toml`
- `specs/055-guidance-catalog-packs/reference/guidance-index.toml`
- `specs/055-guidance-catalog-packs/reference/guardian-index.toml`
- `specs/055-guidance-catalog-packs/reference/guidance-entry.schema.md`
- `specs/055-guidance-catalog-packs/reference/guardian-entry.schema.md`
- `specs/055-guidance-catalog-packs/reference/pack.toml`
- `specs/055-guidance-catalog-packs/reference/resolution-trace.example.json`
- `assistant/packs/guidance-catalog/pack.toml`
- `assistant/packs/guidance-catalog/catalog/catalog-manifest.toml`
- `assistant/packs/guidance-catalog/catalog/guidance-index.toml`
- `assistant/packs/guidance-catalog/catalog/guardian-index.toml`
