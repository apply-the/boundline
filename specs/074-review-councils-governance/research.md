# Research: Review Councils And Role-Gated Governance

**Feature**: 074-review-councils-governance
**Date**: 2026-06-06

## Research Tasks

### 1. Ruleset Validation Strategy

**Decision**: Validate the TOML ruleset at load time using `toml` crate deserialization into typed structs. Check for: duplicate rule IDs, contradictory activation/skip for the same guardian under the same condition, missing required fields, and unrecognized guardian IDs. Fail closed on any validation error with a structured error message.

**Rationale**: `toml` crate is already in the dependency tree. Typed deserialization catches structural errors. Contradiction detection is a post-deserialization check comparing all rule outputs for the same guardian under the same match condition.

**Alternatives considered**: Custom parser without TOML — rejected because TOML is the established config format in Boundline.

### 2. Guardian Execution Evidence Model

**Decision**: Each guardian invocation must write a `guardian.execution.record` structured event with guardian ID, execution status (success/failure/unavailable), finding count, and output trace ref. The council adjudicator reads these records to determine mandatory guardian compliance.

**Rationale**: Structured events (from spec 072) already provide the append-only trace infrastructure. Adding one event type per guardian execution reuses the existing event vocabulary rather than creating a separate evidence store.

**Alternatives considered**: Separate guardian-execution log file — rejected because it duplicates the event infrastructure.

### 3. Council Adjudication Contract

**Decision**: The `council.decision.produced` event records: adjudicator role, authority zone, findings reviewed (accepted/rejected/deferred counts), dissent status, and final outcome (clean/blocked). The CLI command reads the latest guardian activation plan and findings, applies the single-adjudicator model, and emits the event.

**Rationale**: Single-adjudicator with a well-defined event contract keeps the implementation bounded while producing trace-visible decisions. The event is consumable by the existing `boundline trace export --format jsonl` surface.

**Alternatives considered**: Council decision stored in `.boundline/` as a separate file — rejected because structured events already provide persistence.

### 4. Provider Catalog Audit

**Decision**: No change to `assistant/catalog/model-catalog.toml`. Guardian routing does not involve model-family selection.

### 5. Canon Compatibility

**Decision**: Continue Canon 0.70.0 compatibility. Guardian routing and council adjudication are Boundline-owned and do not require Canon packet changes.
