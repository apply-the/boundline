# Research: Session-Native Orchestrator

**Feature**: 013-session-native-orchestrator  
**Date**: 2026-04-29

## R1: Decision Object Representation

**Question**: How should the decision object serialize into session state and traces?

**Decision**: Flat JSON object within session trace events and session state.

**Rationale**: Flat JSON keeps the model inspectable via `boundline inspect` and
debuggable with `jq`. Evidence inputs are string references (trace event IDs,
file paths, Canon artifact paths) rather than embedded copies.

**Alternatives Considered**:
- Nested object graph with embedded evidence: rejected because it inflates
  trace size and complicates queries.
- Separate decision store: rejected because it fragments the inspection
  surface.

**Concrete Shape**:

```json
{
  "id": "uuid",
  "decision_type": "analyze|code|test|fix|replan",
  "target": "src/lib.rs",
  "rationale": "test failure in auth module requires targeted fix",
  "expected_outcome": "auth_test passes after patching validate()",
  "evidence_inputs": [
    "trace:event-id-123",
    "file:src/auth.rs",
    "canon:.canon/requirements/auth.md"
  ],
  "status": "pending|dispatched|verified|failed|recovered"
}
```

## R2: Workspace Signal Collection

**Question**: What workspace signals are cheaply available without external service calls?

**Decision**: File tree enumeration via `std::fs::read_dir` (recursive, bounded
depth), manifest parsing (`Cargo.toml` for Rust, `package.json` for JS/TS),
Boundline config check (`.boundline/config.toml`), Canon artifact presence (`.canon/`).

**Rationale**: All signals are local filesystem reads. No network calls, no
process spawning. Fast and deterministic.

**Alternatives Considered**:
- LSP integration: rejected, too heavy and requires running language server.
- Git status: useful later but not needed for initial plan derivation.
- AST parsing: rejected for this slice, would require language-specific parsers.

**Bounded Depth**: File tree enumeration stops at depth 4 by default. Configurable
later if needed.

## R3: Flow Inference Heuristics

**Question**: Which keyword/signal patterns map reliably to built-in flows?

**Decision**: Simple keyword matching on goal text with workspace signal boosting.

**Rules**:

| Keywords                                    | Workspace Signal         | Inferred Flow |
| ------------------------------------------- | ------------------------ | ------------- |
| fix, bug, broken, failing, regression, test | failing test output      | bug-fix       |
| add, implement, feature, new, create        | new files expected       | change        |
| deliver, release, ship, deploy, complete    | multi-stage scope        | delivery      |
| (none match)                                | —                        | no flow       |

**Rationale**: Deterministic, testable, no false positives on common engineering
goal language. No LLM call needed.

**Alternatives Considered**:
- LLM-based classification: rejected, adds latency and non-determinism to a
  safety-critical routing decision.
- User-authored flow mapping config: good for later, not needed for first slice.

## R4: Tool Adapter Extension

**Question**: How to extend tool adapters for structured output?

**Decision**: Build `ToolResult` from existing `StepExecutionResult` plus command
metadata. No trait signature change.

**Rationale**: The existing `ToolAdapter::execute` returns `StepExecutionResult`
which already contains `output` (Value), `status` (ExecutionStatus), and
`error` (Option<ErrorInfo>). The new `ToolResult` adds structured fields
(`exit_code`, `stdout`, `stderr`, `diff`, `duration`) by extracting them from
the `output` Value or from the concrete tool implementation.

**Alternatives Considered**:
- New trait method `execute_structured`: rejected, unnecessary indirection.
- Replace `StepExecutionResult` entirely: rejected, breaks existing code.

## R5: Fixture Compatibility Routing

**Question**: How to route between decision loop and fixture path?

**Decision**: Predicate chain in `run.rs` and `session_runtime.rs`:

1. If session has `goal_plan` → use decision loop
2. If `--profile` flag is explicit → use fixture path
3. If `.boundline/execution.json` exists and no active session goal → use fixture path
4. If session has goal but no plan → error: run `boundline plan` first
5. Else → error: no execution context available

**Rationale**: Clean, deterministic, backward-compatible. Session-native path
takes precedence but fixture remains accessible via explicit opt-in.

**Alternatives Considered**:
- CLI flag only (`--legacy`): rejected, breaks session-native UX.
- Auto-detect and merge both paths: rejected, too complex and hard to debug.
