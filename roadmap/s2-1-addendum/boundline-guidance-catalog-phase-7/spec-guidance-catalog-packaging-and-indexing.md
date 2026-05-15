# Feature Specification: Guidance Catalog Packaging And Indexing

## Status

Draft

## Relationship To S2.1

S2.1 defines runtime behavior for guidance and guardians.

This specification defines how guidance catalog content is packaged, indexed, versioned, and exposed to Boundline.

## Outcome

Boundline can consume guidance catalogs as installable packs rather than loose Markdown collections.

The catalog must support:

- manifest-based discovery
- guidance indexing
- guardian indexing
- authority source metadata
- lifecycle activation metadata
- Canon promotion compatibility
- trace-friendly resolution

## User Stories

### US1 — Install A Guidance Catalog Pack

As a Boundline operator, I want to install a guidance catalog pack so that Boundline can resolve guidance and guardians from a single declared package.

Acceptance:
- pack manifest is present.
- catalog manifest is present.
- guidance index is present.
- guardian index is present.
- Boundline can list available guidance and guardians.

### US2 — Inspect Guidance Resolution

As a Boundline operator, I want to inspect which guidance was selected and why, so that I can trust the runtime context.

Acceptance:
- resolution trace records authority source.
- resolution trace records strength.
- resolution trace records overrides.
- resolution trace records missing Canon or workspace sources when relevant.

### US3 — Promote Guidance Through Canon

As a governance owner, I want selected guidance files to be promotable into Canon-governed standards, so that shared recommendations can become project authority.

Acceptance:
- guidance files do not depend on Boundline runtime internals.
- Canon promotion notes describe metadata to preserve.
- Boundline can later consume Canon-promoted versions as higher-authority sources.

### US4 — Validate Catalog Shape

As a pack maintainer, I want catalog entries to follow a schema so that invalid packs fail clearly.

Acceptance:
- guidance entry schema exists.
- guardian entry schema exists.
- invalid lifecycle labels are rejected.
- unsupported guardian kinds are rejected.
- missing referenced files are reported as load warnings or errors.

## Functional Requirements

FR-001: A guidance catalog pack MUST include a pack manifest.

FR-002: A guidance catalog pack MUST include a catalog manifest.

FR-003: A catalog MUST provide a guidance index.

FR-004: A catalog MUST provide a guardian index.

FR-005: Guidance index entries MUST include path, pillar, strength, applicable lifecycle phases, and consuming roles.

FR-006: Guardian index entries MUST include pillar, kind, rules, applicable lifecycle phases, and default disposition.

FR-007: Boundline MUST record authority source and strength when resolving catalog entries.

FR-008: Boundline MUST expose catalog entries through inspect surfaces.

FR-009: Catalog content MUST be usable as built-in, shared pack, workspace override, or Canon-promoted standard without rewriting the content.

FR-010: Invalid catalog entries MUST produce explicit load findings or warnings rather than failing silently.

## Success Criteria

SC-001: A catalog pack can be installed and inspected.

SC-002: Boundline can list guidance entries by pillar, lifecycle phase, language, or framework.

SC-003: Boundline can list guardian entries by pillar, kind, lifecycle phase, language, or framework.

SC-004: Resolution traces show why one source overrode another.

SC-005: Canon-promoted versions of a guidance file can be resolved as higher-authority sources.

## Non-Goals

This specification does not:

- define guardian execution
- define review councils
- define adaptive governance
- define semantic retrieval
- define all possible guidance content
- define Canon promotion implementation

## Final Thesis

Markdown content becomes operational only when it is:

```text
declared
indexed
versioned
authority-aware
trace-visible
```

This specification turns the guidance catalog into a runtime-consumable product surface.
