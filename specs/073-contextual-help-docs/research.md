# Research: Contextual Help And Documentation Architecture (Boundline)

**Feature**: 073-contextual-help-docs
**Date**: 2026-06-06

## Research Tasks

### 1. State Inspection Surface

**Decision**: Reuse the existing readiness, probe, and session status surfaces already available in `src/domain/session.rs` and `src/domain/goal_plan.rs`. No new file-system scanning beyond what the existing session loader already does.

**Rationale**: `help-next` is a projection over existing typed state, not a new data source. The session model already exposes lifecycle phase, blocked states, guardian findings, and configuration readiness. Adding a duplicate inspection path would create maintenance drift.

**Alternatives considered**: Direct file-system scanning of `.boundline/` without going through the session model — rejected because it would bypass typed validation and create a second code path for the same data.

### 2. Link Map Format

**Decision**: TOML file with `[metadata]` (schema_version) and `[links]` (diagnostic_key = "relative/path") sections. Committed in the repository under `.boundline/help-links.toml`. Loaded at runtime with `toml` crate (already in dependency tree).

**Rationale**: TOML is already used for `config.toml`. The `toml` crate is already a workspace dependency. The format is human-editable and version-aware. A missing map or missing key produces a non-blocking warning with a fallback to the general troubleshooting page.

**Alternatives considered**:
- JSON: equally viable but less consistent with the existing TOML config surface.
- Hardcoded `&'static str` constants in Rust: rejected per spec — makes link updates require recompilation.
- Remote URL fetch: adds network dependency and violates offline-first design.

### 3. Structured Event Design

**Decision**: Add a new event type `HelpNextRequested` to the existing `EventType` enum in `src/domain/observability.rs`. The payload includes state, diagnostics count, recommended action, command, docs link, and output format. Emitted synchronously before the command returns.

**Rationale**: Spec 072 established the structured event vocabulary. Adding one event type to the existing enum is the minimal path. Emitting synchronously ensures the event is in the trace before the operator sees the output.

**Alternatives considered**:
- Async/background emission: adds complexity for no benefit — `help-next` is fast (<1s).
- No event: rejected by spec clarification.

### 4. CLI Flag Design

**Decision**: Three modes — default (single top issue + count), `--all` (full ordered list), `--json` (structured machine output). Mutually exclusive flags not needed; `--json` and `--all` can combine.

**Rationale**: Matches the pattern established by `boundline evals run --json`. `--all` is additive to the default behavior. `--json` changes the output format but preserves the same diagnostic data.

**Alternatives considered**: Separate subcommands (`help-next list`, `help-next status`) — rejected because `help-next` should remain a single discoverable entry point.

### 5. Provider Catalog Audit

**Decision**: No change to `assistant/catalog/model-catalog.toml`. This feature does not introduce or modify any model-family routing.

**Rationale**: `help-next` is a pure CLI diagnostic. It does not affect model selection, prompt engineering, or inference routing.

### 6. Canon Compatibility

**Decision**: Continue Canon 0.67.0 compatibility. No new Canon packet dependencies. The Canon companion spec (`canon/specs/073-contextual-help-docs/`) is independently owned.

**Rationale**: `help-next` consumes only Boundline-owned state. Canon diagnostic data (mode, packet, evidence) is outside Boundline's scope.
