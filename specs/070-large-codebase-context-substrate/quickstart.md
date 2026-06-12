# Quickstart: Large Codebase Context Substrate

## 1. Use An Isolated Temporary Workspace

Do not run Boundline CLI commands against the Boundline repository root. Create
or use a disposable fixture workspace for every runtime validation scenario.

Expected result:

- the Boundline source tree remains free of workspace-local `.boundline/`
  session state
- each scenario has its own isolated session, traces, and derived
  context-substrate artifacts

## 2. Verify Unsafe Oversized Full Reads Are Refused

Create a fixture workspace with at least one oversized source file, log, or
generated artifact that would be unsafe to read in full by default.

Expected result:

- the substrate performs search-before-read first
- a naive full-read path is refused, downgraded, or redirected before unsafe
  full content loading occurs
- the resulting context projection records the refusal reason and the
  alternative mode that was used instead

## 3. Verify Critical Context Omission Blocks Admission

Create a fixture where a critical artifact such as the active spec, plan,
contract, failing test, or execution gate file cannot be included at the
required fidelity.

Expected result:

- the context-pack state is blocked
- at least one omission finding has blocking severity
- planning or execution admission is withheld instead of continuing with a
  silently degraded pack

## 4. Verify Inspectable Inclusion And Omission

Create a large-repository fixture where some artifacts are full, some are
excerpted, some are compacted by digest, and some ambient or archived entries
are omitted.

Expected result:

- `status` or `inspect` shows included and omitted entries with fidelity tier,
  inclusion mode, authority, and reason
- compacted artifacts still preserve source attribution
- archived or discardable context does not enter the normal planning pack

Manual validation for the operator-facing explanation criterion:

- start from `boundline status` or `boundline inspect` output only
- do not open raw packet files or source documents
- confirm within 30 seconds why one selected item was included and one omitted
  item was excluded

## 5. Verify Repository Map Narrowing

Create a fixture where multiple candidate files share similar names or topics
and only symbol, import, test, or changed-file relations separate the relevant
artifact from the irrelevant ones.

Expected result:

- repository-map-assisted discovery narrows the candidate set before a large
  read occurs
- the final projection explains which local signals caused the selected item to
  outrank omitted ones

## 6. Verify Digest-Backed Compaction

Use a large log, diff, or generated artifact that remains relevant but should
not dominate the pack.

Expected result:

- the artifact is represented as a digest-backed reference plus bounded summary
  or excerpt
- the projection records how to resolve the full source on demand
- the compacted artifact is not treated as lost or silently ignored

## 7. Verify Snapshot Cache Freshness And Diagnostics

Create a reusable local snapshot, then trigger freshness events such as a
branch switch, merge, config change, schema change, adapter change, or Canon
packet change.

Expected result:

- the snapshot cache transitions to stale or degraded before reuse
- the runtime does not treat stale cache state as authoritative planning truth
- diagnostics surface tracked-cache or stale-cache repair guidance when
  applicable

## 8. Verify Patch-Safe Editing Expectations

Use a large file where the intended edit applies only to a bounded section and
where anchor drift can be simulated.

Expected result:

- the runtime or helper logic uses anchored hunks rather than a full-file
  rewrite strategy
- anchor drift produces a rejected or manual-review-required outcome
- successful application requires post-apply verification

## 9. Validate Release Closure

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test unit
cargo test --test contract
cargo test --test integration context_intelligence_semantic_flow::
cargo test --test integration context_intelligence_semantic_inspect::
cargo test --test integration host_session_runtime_flow::
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Then intersect the changed implementation files with uncovered LCOV lines.
Adjust the file list to the actual diff, but expect the large-codebase
context-substrate slice to touch files in a group like:

```bash
implementation_files=(
  src/domain/context_intelligence.rs
  src/domain/goal_plan.rs
  src/domain/project_index.rs
  src/domain/session.rs
  src/orchestrator/context_intelligence.rs
  src/orchestrator/goal_planner.rs
  src/orchestrator/session_runtime_planning_context.rs
  src/cli/diagnostics.rs
  src/cli/inspect/projections.rs
  src/cli/output_context.rs
  src/cli/output_runtime.rs
  src/cli/output_session_status.rs
)
git diff --unified=0 origin/main...HEAD -- "${implementation_files[@]}" \
  | python3 scripts/common/coverage/intersect_patch_coverage.py \
      --lcov lcov.info "${implementation_files[@]}"
```

Expected result:

- formatting passes
- clippy reports zero warnings
- focused context-substrate tests pass
- changed Rust implementation files meet at least 95% changed-file coverage
- docs, assistant assets, changelog, version metadata, and Canon compatibility
  guidance consistently describe release `0.72.5`

Manual validation for `SC-004`:

- run the maintained large-repository fixture set repeatedly enough to compare
  total context-pack selection time across runs
- record the per-run selection duration from the runtime or trace surface
- confirm that at least 95 percent of maintained runs finish initial
  context-pack selection within 10 seconds
