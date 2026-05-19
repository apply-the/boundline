# TD-001: Complexity Hotspots And Structural Debt

## Status

Active - Ongoing Reduction

## Type

Tech Debt

---

# 1. Background

As features accumulated through the delivery, governance, context-intelligence,
reasoning-profile, and assistant-delight slices, several source files and
functions grew beyond maintainable thresholds. This document records the current
hotspot state, establishes a reduction order, and keeps the structural-debt
watchlist local to the Boundline roadmap.

Last verified: 2026-05-19 against the Boundline `0.63.0` worktree after the
`session_runtime` checkpoint, reasoning, and test-module split plus the
`output.rs` run-trace, session-status, trace-summary, explanation, context,
routing/execution, events/diagnostics, delight/inspect-closure, cluster, and
compatibility plus reasoning/adaptive runtime, host-envelope/presentation, and
event-adapter plus support-helper and route/config façade extraction.

---

# 2. Measured Hotspots

## 2.1 Files By Line Count

| File | Total Lines | Notes |
|---|---|---|
| `src/fixture.rs` | 5 519 | Fixture harness; mostly test-support debt, not production runtime debt |
| `src/domain/session.rs` | 4 394 | Domain model plus grouped status-view validation |
| `src/orchestrator/session_runtime.rs` | 4 118 | Checkpoint, reasoning, and tests moved out; core runtime orchestration is still too large |
| `src/cli/init.rs` | 3 660 | Init subcommand logic |
| `src/orchestrator/goal_planner.rs` | 3 424 | Goal planning and Canon input disposition logic |
| `src/orchestrator/session_runtime_tests.rs` | 3 297 | Extracted test-support debt from `session_runtime`; not production runtime debt |
| `src/cli/output.rs` | 2 525 | Run-trace, session-status, trace-summary, explanation, context, routing/execution, events/diagnostics, delight, cluster, compatibility, runtime, host/presentation, and support helpers moved to sibling modules; root file remains oversized |
| `src/orchestrator/context_intelligence.rs` | 2 988 | Context-intelligence orchestration |
| `src/cli/inspect.rs` | 2 941 | Trace inspect and summary rendering |
| `src/cli/session.rs` | 2 936 | Session CLI projection and rendering |
| `src/domain/governance.rs` | 2 881 | Governance domain; better target for module split than small extraction |
| `src/cli.rs` | 2 700 | CLI routing grouped by concern |
| `tests/unit/cli_output.rs` | 2 570 | Large unit-test harness that mirrors the CLI output hotspot |
| `src/orchestrator/engine.rs` | 2 312 | Orchestrator engine runtime flow |
| `src/orchestrator/guidance_runtime.rs` | 2 154 | Guidance and guardian runtime flow |
| `src/cli/config.rs` | 2 140 | Config command surface |
| `src/orchestrator/governance.rs` | 1 961 | Governance orchestration |

Measurement command:

```bash
find src tests -type f -name '*.rs' -exec wc -l {} +
```

This file-count table was refreshed after the `session_runtime` split and the
`output.rs` run-trace/session-status/trace-summary plus explanation, context,
routing/execution, events/diagnostics, delight/inspect-closure, cluster, and
compatibility plus reasoning/adaptive runtime, host-envelope/presentation, and
event-adapter plus support-helper and route/config façade extraction. The
function table below reflects the current module locations for the extracted
output renderers.

## 2.2 Tracked Functions By Line Count

| Function | File | Lines | Priority |
|---|---|---|---|
| `render_run_trace` | `src/cli/output_run_trace.rs` | 628 | P2 |
| `summarize_trace` | `src/cli/inspect.rs` | 479 | P2 - regressed above target |
| `render_session_status` | `src/cli/output_session_status.rs` | 482 | P2 |
| `render_trace_summary` | `src/cli/output_trace_summary.rs` | 262 | P2 |
| `execute_init` | `src/cli/init.rs` | 443 | P2 |
| `advance_workflow` | `src/cli/workflow.rs` | 350 | P3 |

---

# 3. Observations

## `session_runtime.rs`

The in-file tests are gone, and checkpoint plus reasoning concerns now live in
focused sibling modules. That reduced the root production file to 4 118 lines,
with `session_runtime_checkpoint.rs` at 285 lines and
`session_runtime_reasoning.rs` at 976 lines. The remaining issue is still
scope: the core runtime file owns too many orchestration concerns, so the next
intervention should continue splitting by lifecycle boundary rather than fall
back to small helper extraction.

## `session_runtime_tests.rs`

Extracting the test module removed a major source of production-file inflation,
but it created a 3 297-line sibling test file. Treat this as secondary,
test-support debt: it is healthier than keeping the tests inline, but it should
still be split the next time session-runtime behavior is touched in depth.

## `src/cli/output.rs`

The renderer split is progressing into a helper-layer split. `render_run_trace`,
`render_session_status`, and `render_trace_summary` now live in
`output_run_trace.rs` (628 lines), `output_session_status.rs` (482 lines), and
`output_trace_summary.rs` (262 lines), while explanation projection/rendering
logic now lives in `output_explanation.rs` (985 lines), context/semantic
formatting now lives in `output_context.rs` (161 lines), routing plus execution
condition logic now lives in `output_routing.rs` (674 lines), event plus
diagnostics formatting now lives in `output_events.rs` (232 lines), delight and
inspect-closure helpers now live in `output_delight.rs` (145 lines), cluster
renderers and shared cluster-delivery projection helpers now live in
`output_cluster.rs` (140 lines), and compatibility follow-up helpers now live
in `output_compatibility.rs` (47 lines). Reasoning-profile projection and
adaptive runtime summaries now live in `output_runtime.rs` (156 lines), while
command naming, host envelopes, and presentation helpers now live in
`output_host.rs` (141 lines), and checkpoint/error/guidance/next-command helpers
now live in `output_support.rs` (153 lines). Direct event-adapter calls now
import the events module without façade wrappers, and the route/config helper
calls now import `output_routing.rs` directly without local forwarding wrappers.
That drops the root file to 2 525 lines without changing the public API. The
remaining issue is still scope: the root file still carries validation and
status-text helpers, a few thin public route/support façades preserved for
external call sites, and a large inline test module, while
`tests/unit/cli_output.rs` remains a separate maintenance hotspot.

## `src/cli/inspect.rs`

`summarize_trace` was previously marked done, but the current measurement is 479
lines. Treat the earlier extraction as incomplete or regressed and re-open the
item.

## `src/fixture.rs`

This file is a fixture harness, not production runtime debt. It still affects
maintenance cost and should be split opportunistically when fixture behavior is
changed.

## `governance.rs` files

Both the domain and orchestration governance files are large. The intervention
is a module split by concern rather than a broad helper extraction pass.

---

# 4. Work Order

| Step | Item | Status |
|---|---|---|
| 1 | Extract `render_run_trace` from `output.rs` into a focused sibling module | Completed |
| 2 | Re-split `summarize_trace` in `inspect.rs` below the 200-line target | Pending |
| 3 | Extract `render_session_status` from `output.rs` into a focused sibling module | Completed |
| 4 | Extract `render_trace_summary` from `output.rs` into a focused sibling module | Completed |
| 5 | Extract explanation projection and rendering helpers from `output.rs` into a sibling module | Completed |
| 6 | Extract context and semantic formatting helpers from `output.rs` into a sibling module | Completed |
| 7 | Extract routing and execution-condition helpers from `output.rs` into a sibling module | Completed |
| 8 | Extract event-formatting and diagnostics helpers from `output.rs` into a sibling module | Completed |
| 9 | Extract delight and inspect-closure helpers from `output.rs` into a sibling module | Completed |
| 10 | Extract cluster renderers and cluster-delivery helpers from `output.rs` into a sibling module | Completed |
| 11 | Extract compatibility follow-up helpers from `output.rs` into a sibling module | Completed |
| 12 | Extract reasoning-profile and adaptive runtime helpers from `output.rs` into a sibling module | Completed |
| 13 | Extract host-envelope, command-naming, and presentation helpers from `output.rs` into a sibling module | Completed |
| 14 | Remove validation and event-adapter wrappers from `output.rs` by importing the events module directly at call sites | Completed |
| 15 | Extract checkpoint, error-rendering, guidance-projection, and next-command helpers from `output.rs` into a sibling module | Completed |
| 16 | Remove route/config forwarding wrappers from `output.rs` by importing `output_routing.rs` directly at internal call sites | Completed |
| 17 | Extract phase helpers from `execute_init` in `init.rs` | Pending |
| 18 | Split `advance_workflow` into phase-specific helpers | Pending |
| 19 | Continue module-splitting `session_runtime.rs` by concern after the checkpoint, reasoning, and test extraction | In Progress |
| 20 | Module-split large context and planning orchestration files by concern | Pending |
| 21 | Module-split `governance.rs` files by concern | Pending |
| 22 | Split `src/fixture.rs` when fixture behavior is next touched | Pending |

---

# 5. Acceptance Criteria

A hotspot item is considered resolved when:

- The target function is below 200 lines, or the file is split into coherent sub-modules each below 1 000 production lines.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes clean.
- `cargo nextest run --workspace --all-features` passes without regressions.
- No new magic strings or panic-prone paths are introduced in extracted code.
