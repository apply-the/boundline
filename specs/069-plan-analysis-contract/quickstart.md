# Quickstart: Plan Analysis Contract

## 1. Use An Isolated Temporary Workspace

Do not run Boundline CLI commands against the Boundline repository root. Create
or use a disposable fixture workspace for every runtime validation scenario.

Expected result:

- the Boundline source tree remains free of workspace-local `.boundline/`
  session state
- each scenario has its own isolated session and traces

## 2. Verify Clean Planning Analysis

Use a focused fixture or test helper to build a session where:

- goal quality is ready
- plan quality is ready
- backlog quality is ready
- success criteria, validation anchors, risks, constraints, and execution
  readiness evidence are mutually consistent

Expected result:

- `planning_analysis_state` is `clean`
- `planning_analysis_findings` is omitted or empty
- `planning_analysis_coverage` reports aligned counts, including any available
  risk and constraint coverage signals
- execution handoff may be offered

Manual validation for `SC-002`:

- start from `boundline status` or `boundline inspect` output only
- do not open raw packet files or source documents
- confirm within 30 seconds that the session is execution-ready by reading
  `planning_analysis_state` plus the rendered coverage summary

## 3. Verify A Critical Coherence Gap Blocks Execution

Create a session where earlier gates are ready but one required success
criterion or execution input is not covered by the plan and governed backlog
evidence.

Expected result:

- `planning_analysis_state` is `blocked`
- at least one critical finding is reported with source attribution
- execution handoff is withheld
- status, inspect, and assistant surfaces preserve the same blocked state

## 4. Verify Warning-Only Findings Remain Visible Without Blocking

Create a session where the planning picture is broadly coherent but one
non-critical coverage signal remains partial, such as a missing expected
outcome for a non-blocking plan task.

Expected result:

- `planning_analysis_state` is `findings`
- findings remain visible and attributed
- any partial risk or constraint coverage remains visible in the rendered
  coverage summary when those inputs exist
- execution handoff may still be offered because no critical defect remains

## 5. Verify Producer Contract Gaps Stay Honest

Create a session where Canon is part of the active route and execution
readiness depends on a governed field or artifact that only Canon can author,
but that field is absent from the packet.

Expected result:

- `planning_analysis_state` is `blocked`
- one finding has `code = producer_contract_gap`
- the finding references the missing governed artifact
- Boundline does not invent replacement Canon data

## 6. Verify Compatibility

Load a fixture session snapshot that predates planning-analysis persistence and
inspect status.

Expected result:

- the older snapshot deserializes successfully
- status and inspect rendering complete without failure
- no synthetic planning-analysis state is introduced

## 7. Verify Assistant Assets

Run the focused assistant and host contract tests.

```bash
cargo test --test contract assistant_command_definition_contract::
cargo test --test contract host_command_output_contract::
```

Expected result:

- Copilot, Claude, Codex, and Antigravity assets preserve the
  planning-analysis projection
- blocked planning-analysis state routes back to planning, not direct execution
- status and inspect contract output stays additive and machine-readable

## 8. Validate Release Closure

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test unit
cargo test --test contract
cargo test --test integration host_session_runtime_flow::
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Then intersect the changed implementation files with uncovered LCOV lines.
Adjust the file list to the actual diff, but expect the planning-analysis slice
to touch files in this group:

```bash
implementation_files=(
  src/domain/goal_plan.rs
  src/domain/session.rs
  src/orchestrator/session_runtime.rs
  src/orchestrator/session_runtime_planning_runtime.rs
  src/cli/session.rs
  src/cli/output_session_status.rs
  src/cli/inspect/projections.rs
  src/cli/output_orchestrate.rs
)
git diff --unified=0 origin/main...HEAD -- "${implementation_files[@]}" \
  | python3 scripts/common/coverage/intersect_patch_coverage.py \
      --lcov lcov.info "${implementation_files[@]}"
```

Expected result:

- formatting passes
- clippy reports zero warnings
- focused and workspace-relevant tests pass
- changed implementation files meet at least 95% changed-file coverage
- docs, assistant assets, changelog, version metadata, and Canon compatibility
  guidance consistently describe release `0.70.0`
