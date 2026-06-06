# Research: Evals And Runtime Observability

**Feature**: 072-evals-runtime-observability
**Date**: 2026-06-05

## Research Tasks

### 1. Trace Storage Format Compatibility

**Decision**: Use existing `.boundline/traces/` directory as the sole compaction input. Each trace item is already an identifiable artifact (decision, finding, transcript, context packet) with enough metadata for classification.

**Rationale**: The existing trace format already supports item-level identity and metadata. Compaction reads from and writes to the same directory tree, with compaction events emitted to a separate events log. No new storage backend is needed.

**Alternatives considered**:
- SQLite-backed trace store: would add schema migration complexity and a new persistence surface for a read-only gate.
- In-memory-only compaction: would lose the durability guarantee that compaction results survive process restarts.

### 2. JSONL Export Without External Libraries

**Decision**: Use `serde_json` with `Write`-based line-by-line serialization. No external streaming JSON library is needed.

**Rationale**: `serde_json` is already in the dependency tree. JSONL is trivial — each line is a self-contained JSON object terminated by `\n`. The event volume (hundreds to low thousands per session) is well within `serde_json`'s performance envelope.

**Alternatives considered**:
- `simd-json`: faster but adds a native dependency and is not needed at this scale.
- Protocol Buffers or Avro: binary formats add tooling dependencies and hurt debuggability for a CLI-first tool.

### 3. Compaction Algorithm Design

**Decision**: Single-pass in-memory classification with a pre-built classification table keyed by item type. The table maps each known item type to its retention class. Items not in the table are classified by conservative tiebreaking (stricter class wins). Oversized traces (>50k items) are rejected with an actionable message unless the operator confirms.

**Rationale**: A single pass keeps the implementation simple, predictable, and easy to test. The 50k-item bound is generous for multi-turn agent sessions while preventing unbounded memory use. Conservative tiebreaking aligns with the hard rule that compaction must never destroy required evidence.

**Alternatives considered**:
- Multi-pass streaming: adds complexity for a scale that isn't needed in V1.
- LLM-assisted classification: violates the "no hidden intelligence" principle and would make compaction non-deterministic.

### 4. Eval Runner Architecture

**Decision**: Implement eval fixtures as Rust test functions under `tests/` that can be invoked by both `cargo test` (CI) and a dedicated `boundline evals run` command (local). The eval runner loads fixtures from `.boundline/evals/` and produces a structured JSON summary.

**Rationale**: Dual invocation (CI via `cargo test`, local via CLI) gives operators flexibility. Rust test harness provides assertion infrastructure and CI integration for free. The dedicated CLI command adds human-readable output and session-aware fixture loading.

**Alternatives considered**:
- Separate eval binary: would require a new workspace member crate and separate build step.
- Script-based evals (Python/shell): would add a language dependency and lose Rust type safety for fixture shapes.

### 5. Structured Event Vocabulary

**Decision**: Define event types as typed Rust enums with `serde` derives. Each variant maps to one event type in the JSONL export. The `schema_version` field is a `&'static str` constant per variant.

**Rationale**: Typed enums ensure that every event shape is validated at compile time. `serde` derives produce correct JSON without manual serialization. Version constants per variant satisfy the per-event-type `schema_version` requirement from the spec.

**Alternatives considered**:
- `serde_json::Value` with runtime validation: loses compile-time shape guarantees.
- Separate event schema registry (Protobuf `.proto` files): overkill for a CLI tool with a single producer.

### 6. Sensitive Data Filtering

**Decision**: Field-level allowlists per event type. Each event type defines which fields are safe to include in exports. Fields not in the allowlist are either omitted or redacted with a placeholder. No general-purpose regex or ML-based scanner.

**Rationale**: Allowlists are deterministic, auditable, and easy to extend per event type. They avoid the false-positive and false-negative risks of pattern-based scanners.

**Alternatives considered**:
- Regex-based secret scanning: fragile, false-positive prone, and not appropriate for structured data.
- No filtering: violates FR-014 and SC-006.

### 7. Provider Catalog Audit

**Decision**: No change to `assistant/catalog/model-catalog.toml`. This feature does not introduce or modify any model-family routing.

**Rationale**: Evals, compaction, and observability are runtime infrastructure features. They do not affect model selection, prompt engineering, or inference routing.

**Alternatives considered**: N/A — this is a no-change audit.

### 8. Canon Compatibility

**Decision**: Continue Canon 0.67.0 compatibility. This feature consumes Canon-governed evidence refs already present in traces but does not introduce new Canon packet dependencies or schema changes.

**Rationale**: Trace compaction operates on Boundline-owned artifacts. Canon evidence refs are treated as opaque identifiers — compaction never mutates Canon-owned data.

**Alternatives considered**: N/A — no Canon-side change is proposed or needed.
