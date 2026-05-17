# TD-001: Complexity Hotspots and Structural Debt

## Status

Active — Ongoing Reduction

## Type

Tech Debt

---

# 1. Background

As features accumulated through S1–S4, several source files and functions grew well beyond maintainable thresholds. This document captures the measured hotspot state as of the S4 closeout, establishes a priority order for reduction, and records progress as each item is addressed.

---

# 2. Measured Hotspots

## 2.1 Files by Line Count (Production + Tests)

| File | Total Lines | Notes |
|---|---|---|
| `src/orchestrator/session_runtime.rs` | 7 074 | Test module begins at line 4 311; production functions are individually short |
| `tests/fixture.rs` | 5 208 | Test fixture harness; not production debt |
| `src/cli/output.rs` | 4 273 | Pure CLI rendering; no tests in-file |
| `src/domain/session.rs` | 3 823 | Domain model + grouped status-view validation |
| `src/cli/init.rs` | 3 659 | Init subcommand logic |
| `src/cli.rs` | 2 660 | CLI routing grouped by concern |
| `src/cli/inspect.rs` | 2 559 | Trace inspect and summary rendering |
| `src/domain/governance.rs` | 2 289 | Governance domain; functions are individually short |
| `src/orchestrator/governance.rs` | 1 625 | Governance orchestration; functions individually short |

## 2.2 Functions by Line Count

| Function | File | Lines | Priority |
|---|---|---|---|
| `render_run_trace` | `src/cli/output.rs` | 570 | P2 |
| `render_session_status` | `src/cli/output.rs` | 449 | P2 |
| `execute_init` | `src/cli/init.rs` | 443 | P2 |
| `summarize_trace` | `src/cli/inspect.rs` | 398 | P3 — partially addressed |
| `advance_workflow` | `src/cli/workflow.rs` | 350 | P3 |

---

# 3. Observations

## `session_runtime.rs`

File size (7 k lines) is misleading. The test module starting at line 4 311 accounts for roughly 40 % of the file. Production functions are individually short (longest is 53 lines). The primary issue is scope: the file owns too many concerns. The correct intervention is a module-split by concern, not function extraction.

## `governance.rs` files

Both the domain and orchestration governance files are large but contain no individually long functions (longest: 55 lines). The intervention is the same as above: module-split by concern rather than function extraction.

---

# 4. Work Order

| Step | Item | Status |
|---|---|---|
| 1 | Extract accumulator structs from `summarize_trace` in `inspect.rs` | Done |
| 2 | Extract render helpers from `render_run_trace` in `output.rs` | Pending |
| 3 | Extract render helpers from `render_session_status` in `output.rs` | Pending |
| 4 | Extract phase helpers from `execute_init` in `init.rs` | Pending |
| 5 | Module-split `session_runtime.rs` by concern | Pending |
| 6 | Module-split `governance.rs` files by concern | Pending |

---

# 5. Acceptance Criteria

A hotspot item is considered resolved when:

- The target function is below 200 lines, or the file is split into coherent sub-modules each below 1 000 production lines.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes clean.
- `cargo nextest run --workspace --all-features` passes without regressions.
- No new magic strings or panic-prone paths are introduced in extracted code.
