# Quickstart: `synod run-demo`

**Feature**: 005-test-fix-loop-demo  
**Date**: 2026-04-25

This quickstart shows the full developer experience of the new `synod run-demo`
command. It assumes the slice has been implemented and `cargo build` succeeded.

## 1. From a clean checkout

```sh
cd /path/to/synod
cargo build --release
./target/release/synod run-demo
```

Expected console output (key lines, exact wording may differ):

```text
[run-demo] step 1 of 3 — analyzer "analyze" — outcome=success attempt=1
[run-demo] step 2 of 3 — coder "code"     — outcome=recoverable attempt=1
[run-demo] step 2 of 3 — coder "code"     — outcome=success attempt=2
[run-demo] step 3 of 3 — tester "verify"  — outcome=replan_required attempt=1
[run-demo] step 4 of 5 — analyzer "analyze#replan-1" — outcome=success attempt=1
[run-demo] step 5 of 5 — coder "code#replan-1"       — outcome=success attempt=1
[run-demo] step 5 of 5 — tester "verify"             — outcome=success attempt=2
[run-demo] terminal status: Succeeded
[run-demo] trace: /path/to/synod/.synod/demo-workspace/.synod/traces/<id>.json
[run-demo] final source file: /path/to/synod/.synod/demo-workspace/src/buggy.rs
```

## 2. Inspect the fixed source file

```sh
cat .synod/demo-workspace/src/buggy.rs
```

The file MUST no longer contain the seeded `// TODO-BUG: ...` marker; the
function body MUST be the fixed version.

## 3. Inspect the trace

```sh
ls .synod/demo-workspace/.synod/traces/
cat .synod/demo-workspace/.synod/traces/<id>.json | jq '.terminal_status'
```

`terminal_status` MUST be `"Succeeded"`. The trace JSON MUST contain (at
least): one `Recoverability::Retryable` entry on the `code` step, one
`Recoverability::ReplanRequired` entry on the first `verify` step, and an
inserted analyzer + coder step pair before the final successful `verify`.

## 4. Re-run is idempotent

```sh
./target/release/synod run-demo
```

The workspace is reset on every invocation. The output and final file
contents MUST be identical to the previous run.

## 5. What this demonstrates

- Synod can take a real failing test in a real on-disk workspace and drive it
  to a passing state through its existing orchestrator.
- Retry works: the first coder attempt fails recoverably and is retried.
- Replan works: the first tester attempt triggers a replan, which inserts a
  fresh analyze + code pair and the next tester attempt passes.
- The whole flow is bounded by `RunLimits` and produces an inspectable trace.
