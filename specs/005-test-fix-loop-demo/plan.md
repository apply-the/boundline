# Implementation Plan: Test-Fix Loop Vertical Slice Demo

**Branch**: `005-test-fix-loop-demo` | **Date**: 2026-04-25 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/005-test-fix-loop-demo/spec.md`

## Summary

Add a single new CLI subcommand `synod run-demo` that provisions an isolated on-disk
demo workspace containing one buggy source file plus one failing test definition,
then drives it to a passing state through the **existing** orchestrator, planner,
agent registry, tool registry, and trace store. The slice reuses the
`analyzer → coder → tester` plan already wired in `src/demo/`, extends the existing
`DemoRunProfile` with a single new `test_fix_loop` profile, and extends the existing
`coder` and `tester` adapters in-place with the minimum logic needed to (a) actually
mutate the seeded source file and (b) report pass/fail by reading that file. The
profile deterministically forces one recoverable failure on the first coder attempt
(retry path) and one `ReplanRequired` failure on the first tester attempt (replan
path), then succeeds. No new abstractions, frameworks, plugin systems, multi-agent
voting, model routing, or Canon integrations are introduced.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies only (`clap`, `serde`,
`serde_json`, `thiserror`, `tracing`, `uuid`); no new crates  
**Storage**: On-disk demo workspace under `<repo>/.synod/demo-workspace/` for the
seeded buggy source and failing test definition; existing file-backed trace store
under `<workspace>/.synod/traces/` for the run trace; no session record is touched
by this slice  
**Testing**: `cargo test --all-targets` plus a focused integration test that drives
`synod run-demo` end-to-end and asserts retry, replan, succeeded terminal state, and
the post-run file contents  
**Target Platform**: macOS / Linux developer workstations (single process, single
thread of control)  
**Project Type**: Single Rust binary crate (`synod`) with the existing
`src/{cli,domain,orchestrator,adapters,registry,demo}/` layout  
**Execution Model**: Strictly sequential. One `Orchestrator::run` call per
invocation, one step active at a time, bounded by the existing `RunLimits`
(`max_retries = 1`, `max_replans = 1`, `max_steps = 6`)  
**Observability Surface**: Existing `FileTraceStore` writes a JSON trace per run
under `<demo-workspace>/.synod/traces/`, plus the existing
`output::render_run_trace` step-by-step console output, augmented with a final
"final source file" path line  
**Performance Goals**: N/A — single demo invocation completes in well under 1
second on a developer workstation; no scale targets  
**Constraints**: Must NOT introduce: new frameworks, plugin systems, multi-agent
voting, model routing, Canon integration, parallelism, background workers, or any
new top-level module. Must reuse existing `Orchestrator`, `StaticPlanner`,
`AgentRegistry`, `ToolRegistry`, `FileTraceStore`, `DemoRunProfile`, and
`build_demo_runtime` surfaces  
**Scale/Scope**: One CLI subcommand, one new profile constructor, two extended
adapters (coder, tester), one tiny seed-workspace helper module, and a small set
of tests. No spec-level scaling concerns.

## Constitution Check

- **Delivery identity** — PASS. The slice is the canonical Synod delivery story:
  failing test → analyze → fix → verify → passing test. See spec User Story 1.
- **Delivery-first scope** — PASS. The slice is execution-first; it adds nothing
  beyond what is needed to demonstrate the loop.
- **Bounded execution** — PASS. Reuses default `RunLimits` (`max_steps = 6`,
  `max_retries = 1`, `max_replans = 1`); terminal conditions are
  `Succeeded | Failed`; see Edge Cases in spec for exhaustion behavior.
- **Stateful execution** — PASS. The orchestrator's `TaskContext` is read by
  every step and updated after each step; the coder writes the file path it
  modified into the step output; the tester reads the seeded file from the
  workspace path stored in the task context.
- **Mutable planning** — PASS. The first tester attempt returns
  `ReplanRequired`, which exercises the existing replan path that inserts an
  additional analyzer + coder pair before the next tester attempt. Plan
  mutations are visible in the trace.
- **Sequential-first design** — PASS. One step at a time; no parallelism;
  `Orchestrator::run` is the single execution loop.
- **Tool-agent symmetry** — PASS. analyzer = agent (think), coder = agent (act
  on file), tester = tool (evaluate). All three transitions are visible in the
  trace as distinct step entries.
- **Observability and explicit intelligence** — PASS. Every step, every
  recovery decision, every plan mutation, and the final terminal state are
  written to the existing trace; the console output renders the same step
  sequence; the only "intelligence" is hardcoded deterministic behavior, which
  is explicit in `endpoints.rs`.
- **Non-goals and external separation** — PASS. No Canon integration. No
  council, model routing, long-term memory, UI/UX, or deployment pipeline. No
  new dependencies.
- **Minimal slice** — PASS. The smallest independently valuable capability is
  exactly User Story 1 (the demo reaches `Succeeded` on a real seeded file);
  US2 and US3 are layered on top by toggling existing flags in the new profile.

All constitution gates PASS. Re-checked after Phase 1 design (see end of
file): still PASS.

## Project Structure

### Documentation (this feature)

```text
specs/005-test-fix-loop-demo/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── run-demo-cli.md
├── checklists/
│   └── requirements.md
├── spec.md
└── tasks.md            # (created by speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── cli.rs              # add `RunDemo` variant to DeveloperCommand + plumbing
├── cli/
│   └── run.rs          # add `execute_run_demo(...)` reusing execute_profile
├── demo.rs             # re-export the new workspace-seed helper
├── demo/
│   ├── endpoints.rs    # extend coder + tester adapters in-place
│   ├── profile.rs      # add `DemoRunProfile::test_fix_loop()`
│   └── workspace.rs    # NEW: tiny helper to seed/reset the demo workspace
├── adapters/
│   └── trace_store.rs  # unchanged
├── orchestrator/       # unchanged
├── registry/           # unchanged
├── domain/             # unchanged
└── bin/synod.rs        # already routes DeveloperCommand; route new variant

tests/
├── integration/
│   └── run_demo_flow.rs    # NEW: end-to-end test for `synod run-demo`
├── contract/
│   └── run_demo_contract.rs # NEW: contract for the `synod run-demo` CLI surface
└── unit/
    └── demo_workspace.rs   # NEW: unit tests for the seed/reset helper
```

**Structure Decision**: Reuse the existing single-crate layout. The only new
file in `src/` is `src/demo/workspace.rs`, a small module that owns the seed and
reset of the on-disk demo workspace. This avoids adding any new top-level
module and keeps the demo concerns under the existing `src/demo/` folder. No
new top-level directories. No new crates. The constitution gate "Minimal slice"
is satisfied.

## Complexity Tracking

> No constitution violations. Table intentionally empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| —         | —          | —                                    |

## Post-Design Constitution Re-Check

All ten gates re-evaluated after Phase 1 design (data-model, contracts,
quickstart): still PASS. The design adds exactly one CLI subcommand, one
profile constructor, one tiny seed/reset helper, and two minimal in-place
adapter extensions. No surface that was forbidden by the constitution or by
the user input was introduced.
