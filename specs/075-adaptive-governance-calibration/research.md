# Research: Adaptive Governance Calibration

**Feature**: 075-adaptive-governance-calibration
**Date**: 2026-06-06

## Research Tasks

All Technical Context fields were filled with known values from the workspace (Rust 1.96.0, existing dependencies, existing patterns). No NEEDS CLARIFICATION markers remained. The research phase confirms best practices and pattern reuse.

## Decisions

### 1. Calibration Policy File Format

**Decision**: TOML, matching existing `.boundline/` conventions (`.boundline/config.toml`, `.boundline/guardian-rules.toml`). File named `.boundline/calibration-policy.toml`.

**Rationale**: TOML is already the workspace configuration format. The existing `toml` crate handles deserialization. Typed serde models (per constitution language rules) will be used.

**Alternatives considered**: JSON (less human-friendly for policy authoring), YAML (no existing workspace support).

### 2. Control Level Type Design

**Decision**: Typed Rust enum `ControlLevel` with variants `Advisory`, `Catch`, `Rule`, `Hook`. Each variant carries structured metadata (reason, threshold, override policy reference).

**Rationale**: Matches existing domain patterns (`CouncilOutcome`, `RetentionClass`). Enables exhaustive matching in adjudication logic. Constitution requires typed enums for stable shapes.

**Alternatives considered**: String-based levels (violates no-magic-literals rule), integer encoding (less readable in traces).

### 3. CLI Command Surface

**Decision**: New `boundline override` developer command following the existing `boundline council` pattern. Registered in `DeveloperCommand` enum with a `--workspace`, `--guardian-id`, `--control-id`, `--level`, `--reason`, `--expiry` flag set. Consumed by existing `boundline run` and `boundline continue` dispatch.

**Rationale**: Matches the clarified normative surface from Q2. The existing CLI pattern (derive-based clap enums, `DeveloperCommand` routing, `DispatchOutcome` return) is well-established and tested.

**Alternatives considered**: Interactive prompt (adds TUI dependency, harder to automate), `--override` flag on `run` (deferred per Q2 clarification as a future shortcut).

### 4. Trust Metric Computation

**Decision**: Trust counters stored in typed `GuardianTrustRecord` struct persisted alongside the trace store. True positive rate = TP / (TP + FP), computed only over adjudicated findings above the minimum evidence threshold. No rate emitted when sample size is insufficient.

**Rationale**: Matches Q6 clarification. The formula is simple and auditable. Deferred findings are excluded until resolved, avoiding premature metric contamination.

**Alternatives considered**: Weighted moving average (adds complexity without clear benefit for v1), per-session rates (too noisy with small samples).

### 5. Structured Event Emission

**Decision**: Four new `EventType` variants in `src/domain/observability.rs`: `ControlLevelAssigned`, `ControlLevelGraduated`, `ControlDegraded`, `ControlEscalated`. Each carries a typed payload struct with schema versioning.

**Rationale**: Existing pattern from spec 072 (Evals and Runtime Observability). Reuses the `StructuredRuntimeEvent` envelope, event deduplication, and JSONL export infrastructure.

**Alternatives considered**: Separate event channel (unnecessary complexity for v1), logging-only (not trace-visible, fails SC-005).

### 6. Override Record Storage

**Decision**: Override records persisted as TOML in `.boundline/overrides.toml` (one record per finding), consumed by `boundline run`/`continue` before adjudication. Records are trace-visible via `boundline inspect`.

**Rationale**: TOML is consistent with existing config files. Flat file avoids schema migration concerns for v1. Records are small and human-readable.

**Alternatives considered**: SQLite table (adds schema migration burden for a simple key-value store), embedded in trace JSONL (harder to query before adjudication).

### 7. Degradation and Escalation Rules

**Decision**: Degradation rules are evaluated inline during council adjudication when a provider availability check fails. Escalation triggers are checked after adjudication. Both emit structured trace events.

**Rationale**: Degradation and escalation are council-adjacent concerns, not separate subsystems. Inline evaluation keeps the code path simple and avoids introducing a rules engine.

**Alternatives considered**: Separate degradation daemon or background process (over-engineered for v1).

## Provider Catalog No-Change Audit

No new provider or model dependencies are introduced. The existing `assistant/catalog/model-catalog.toml` is unaffected. Confirmed: no catalog update required.
