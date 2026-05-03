# Quickstart: Product Unification And Surface Closure

**Feature**: 032-workflow-surface-closure  
**Date**: 2026-05-02

This walkthrough shows the intended `0.32.0` operator story: discover a named
workflow, run it through the same primary Boundline path used by direct native
execution, inspect routing plus assistant binding from workflow-facing output,
and keep explicit compatibility follow-up visibly subordinate.

## 1. Discover the bounded workflow entrypoints

```bash
cargo run --bin boundline -- workflow list --workspace <workspace>
```

Expected behavior:
- Workflow discovery exposes the available workflow names, phase chains,
  summaries, and the exact `workflow run` command needed to start each one.
- Assistant guidance for Claude, Codex, and Copilot can map directly to the
  same command instead of sending the operator to undocumented raw CLI usage.
- Gemini guidance uses the same workflow vocabulary even though it remains
  CLI-first in this release.

## 2. Start a named workflow on the primary Boundline path

```bash
cargo run --bin boundline -- workflow run governed-delivery --workspace <workspace> --goal "Fix the failing add test"
```

Expected behavior:
- The workflow command stays on the same primary session-native product story as
  `start -> capture -> plan -> run`.
- Output keeps workflow identity and phase explicit while preserving routing,
  execution condition, and the same bounded `next_command` model.
- If the active native route requires an unsupported assistant runtime, the run
  stops explicitly instead of silently switching bindings.

## 3. Inspect routing and binding from workflow follow-through

```bash
cargo run --bin boundline -- workflow status --workspace <workspace>
```

Expected behavior:
- Workflow-aware status keeps `workflow`, `workflow_phase`, routing, route
  ownership, and route-config projection visible on the same summary surface.
- The operator can identify the authoritative slot route and assistant binding
  without reading config files directly.
- The next action remains bounded and uses the same Boundline-owned vocabulary as
  direct native follow-through.

## 4. Resume or inspect without changing products

```bash
cargo run --bin boundline -- workflow resume --workspace <workspace>
cargo run --bin boundline -- workflow inspect --workspace <workspace>
```

Expected behavior:
- Resume keeps the workflow on the same primary Boundline path when more bounded
  work is credible.
- Inspect combines workflow identity with trace-backed evidence without
  pretending the operator left the workflow product surface.
- Blocked, governance-gated, and terminal states remain explicit and actionable.

## 5. Keep explicit compatibility follow-up visibly subordinate

```bash
cargo run --bin boundline -- run --workspace <workspace> --goal "Fix the failing add test" --compatibility
cargo run --bin boundline -- inspect --workspace <workspace>
```

Expected behavior:
- Compatibility remains an explicit opt-in route and its follow-up continues to
  report compatibility ownership clearly.
- Workflow or direct native guidance does not imply that compatibility is the
  default path.
- Assistant surfaces preserve one product identity: users are operating Boundline,
  with compatibility remaining a subordinate exception and Canon visible only as
  bounded governance inside that product.