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

Last verified: 2026-05-29 against the Boundline `064-session-assistant-fine-tuning` worktree after the
`session_runtime` production-file split down to 681 lines, continued growth in
`init.rs`, `cli.rs`, `session.rs`, and the `session_runtime_tests.rs` sibling
harness, resolution of `summarize_trace` and `advance_workflow`, and the
addition of `orchestrate.rs`, `provider_runtime.rs`, and `session_cli_runtime.rs`
as new tracked surfaces.

---

# 2. Measured Hotspots

## 2.1 Files By Line Count

| File | Total Lines | Notes |
|---|---|---|
| `src/cli/init.rs` | 6 988 | Init and update subcommand logic; grown significantly with `execute_update` and guided-init additions |
| `src/fixture.rs` | 6 631 | Fixture harness; mostly test-support debt, not production runtime debt |
| `src/orchestrator/session_runtime_tests.rs` | 5 878 | Extracted test-support debt from `session_runtime`; continues to grow as runtime behavior expands |
| `src/cli.rs` | 5 192 | CLI routing grouped by concern; major growth since last measurement |
| `src/cli/session.rs` | 5 097 | Session CLI projection and rendering; major growth since last measurement |
| `src/domain/session.rs` | 4 963 | Domain model plus grouped status-view validation |
| `tests/unit/cli_output.rs` | 3 756 | Large unit-test harness that mirrors the CLI output hotspot |
| `src/orchestrator/goal_planner.rs` | 3 624 | Goal planning and Canon input disposition logic |
| `src/domain/governance.rs` | 3 366 | Governance domain; better target for module split than small extraction |
| `src/orchestrator/context_intelligence.rs` | 3 005 | Context-intelligence orchestration |
| `src/cli/output.rs` | 2 960 | Sibling output modules extracted; root file still carries validation, status-text helpers, and a large inline test module |
| `src/cli/orchestrate.rs` | 2 777 | Orchestrate subcommand surface; new hotspot |
| `src/cli/inspect.rs` | 2 382 | Trace inspect and summary rendering |
| `src/orchestrator/engine.rs` | 2 365 | Orchestrator engine runtime flow |
| `src/cli/config.rs` | 2 255 | Config command surface |
| `src/orchestrator/guidance_runtime.rs` | 2 232 | Guidance and guardian runtime flow |
| `src/adapters/provider_runtime.rs` | 2 087 | Provider runtime adapter; new hotspot |
| `src/orchestrator/governance.rs` | 2 005 | Governance orchestration |
| `tests/unit/session_cli_runtime.rs` | 2 034 | Test harness for session CLI runtime; new test-support debt |
| `src/orchestrator/session_runtime.rs` | 681 | Production core split complete; checkpoint, reasoning, and tests are in sibling modules |

Measurement command:

```bash
find src tests -type f -name '*.rs' -exec wc -l {} +
```

This file-count table was refreshed against the `064-session-assistant-fine-tuning`
worktree. Notable movements: `session_runtime.rs` production core dropped from
4 118 to 681 lines; `init.rs`, `cli.rs`, and `session.rs` each grew substantially;
`orchestrate.rs` and `provider_runtime.rs` surfaced as new tracked files. The
function table below reflects the current module locations for the extracted
output renderers.

## 2.2 Tracked Functions By Line Count

| Function | File | Lines | Priority |
|---|---|---|---|
| `render_run_trace` | `src/cli/output_run_trace.rs` | 608 | P1 - still well above 200-line target |
| `render_session_status` | `src/cli/output_session_status.rs` | 547 | P1 - regressed further above target |
| `render_trace_summary` | `src/cli/output_trace_summary.rs` | 282 | P2 |
| `execute_init` | `src/cli/init.rs` | 275 | P2 - improved but still above target |
| `summarize_trace` | `src/cli/inspect.rs` | 178 | Resolved - under 200-line target |
| `advance_workflow` | `src/cli/workflow.rs` | 145 | Resolved - under 200-line target |

---

# 3. Observations

## `session_runtime.rs`

The production core split is largely complete. Checkpoint, reasoning, and all
in-file tests now live in focused sibling modules, bringing the root production
file down to 681 lines. The remaining risk is that `session_runtime_tests.rs`
grew to 5 878 lines as new session-runtime behavior was tested inline; the test
file should be split along behavioral boundaries the next time session-runtime
behavior is touched in depth. Item 19 in the work order can be moved to Completed
for the production file; the test-file split is tracked separately.

## `session_runtime_tests.rs`

The test module is now 5 878 lines, up from 3 297 at the last measurement. It is
test-support debt, not production runtime debt, but the maintenance cost is
significant. The next session-runtime behavioral touch should split this file by
lifecycle phase.

## `src/cli/output.rs`

The sibling-module split is complete for all named renderer and helper concerns.
The root file currently sits at 2 960 lines; it still carries validation helpers,
status-text helpers, a few public façades preserved for external call sites, and
a large inline test module. The extracted sibling modules have themselves become
hotspots: `output_session_status.rs` is 1 150 lines and `render_session_status`
alone spans 547 lines, while `output_run_trace.rs` is 630 lines and
`render_run_trace` spans 608 lines. The next intervention on this surface should
target those two renderer functions rather than the root façade.

## `src/cli/inspect.rs`

`summarize_trace` is now 178 lines, which is under the 200-line target. Mark
item 2 in the work order as Completed.

## `src/cli/init.rs`

The file grew to 6 988 lines, more than doubling since the last measurement.
The addition of `execute_update` and guided-init helpers accounts for the bulk
of the growth. This is now the single largest production file. The phase-helper
extraction for `execute_init` (item 17) needs to be extended to cover
`execute_update` as well, and the file should be split into focused submodules
by command concern.

## `src/cli.rs` and `src/cli/session.rs`

Both files grew substantially: `cli.rs` from 2 700 to 5 192 lines and
`session.rs` from 2 936 to 5 097 lines. These are the two largest non-fixture,
non-init production CLI files and should be split by routing concern in the
next planned intervention.

## `src/cli/orchestrate.rs`

A new file at 2 777 lines that was not previously tracked. Add to the work
order as a module-split target once the higher-priority `init.rs` and
`cli.rs`/`session.rs` items are addressed.

## `src/fixture.rs`

Now at 6 631 lines. It remains fixture harness debt, not production runtime debt,
but the file has grown significantly. Split opportunistically when fixture
behavior is next changed.

## `governance.rs` files

Both the domain and orchestration governance files remain large (3 366 and 2 005
lines respectively). The intervention is a module split by concern rather than a
broad helper extraction pass.

---

# 4. Work Order

| Step | Item | Status |
|---|---|---|
| 1 | Extract `render_run_trace` from `output.rs` into a focused sibling module | Completed |
| 2 | Re-split `summarize_trace` in `inspect.rs` below the 200-line target | Completed |
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
| 17 | Extract phase helpers from `execute_init` and `execute_update` in `init.rs`; split `init.rs` into focused submodules by command concern | Pending |
| 18 | Split `advance_workflow` into phase-specific helpers | Completed |
| 19 | Continue module-splitting `session_runtime.rs` production core by concern | Completed |
| 20 | Module-split large context and planning orchestration files by concern | Pending |
| 21 | Module-split `governance.rs` files by concern | Pending |
| 22 | Split `src/fixture.rs` when fixture behavior is next touched | Pending |
| 23 | Split `session_runtime_tests.rs` by lifecycle phase when session-runtime behavior is next touched in depth | Pending |
| 24 | Reduce `render_run_trace` (608 lines) in `output_run_trace.rs` below 200-line target by extracting stage-rendering helpers | Pending |
| 25 | Reduce `render_session_status` (547 lines) in `output_session_status.rs` below 200-line target by extracting section-rendering helpers | Pending |
| 26 | Module-split `src/cli.rs` (5 192 lines) and `src/cli/session.rs` (5 097 lines) by routing concern | Pending |
| 27 | Module-split `src/cli/orchestrate.rs` (2 777 lines) by subcommand concern | Pending |

---

# 5. Acceptance Criteria

A hotspot item is considered resolved when:

- The target function is below 200 lines, or the file is split into coherent sub-modules each below 1 000 production lines.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes clean.
- `cargo nextest run --workspace --all-features` passes without regressions.
- No new magic strings or panic-prone paths are introduced in extracted code.
