# Quickstart: Backlog Contract

## 1. Use An Isolated Temporary Workspace

Do not run Boundline CLI commands against the Boundline repository root. Create
or use a disposable fixture workspace for every runtime validation scenario.

## 2. Verify The Canon `0.67.0` Packet Audit

Use focused fixtures or test helpers that mirror the Canon `0.67.0` backlog
packet artifact set.

Expected result:

- the runtime inspects only artifacts already present in the Canon packet
- the feature does not depend on a hidden Boundline-only backlog schema

## 3. Verify Closure-Limited Packets Block Handoff

Use a focused fixture backlog packet that contains only
`backlog-overview.md` plus `planning-risks.md`.

Expected result:

- `backlog_quality_state` is `blocked`
- `backlog_quality_findings` includes the closure-limited packet reason
- execution handoff is withheld

## 4. Verify Full Packets Without Handoff Use One Clarification Path

Use a focused full backlog packet that is otherwise credible but omits
`execution-handoff.md` or downstream-ready verification anchors.

Expected result:

- `backlog_quality_state` is `clarification_required`
- `backlog_quality_findings` includes `missing_execution_handoff`
- exactly one `phase_request` is emitted
- the session remains non-terminal and execution handoff is withheld

## 5. Verify Ready Backlog Enables Later Gates

Use a focused full backlog packet that includes stable `slice_id`,
implementation refs, independent verification anchors, and
`execution-handoff.md`.

Expected result:

- `backlog_quality_state` is `ready`
- `backlog_task_count` and `backlog_mvp_scope` are populated
- planning analysis may run only after backlog quality is ready

## 6. Verify Compatibility

Load a fixture session snapshot that predates the additive backlog-quality
projection and inspect status and orchestration output.

Expected result:

- the older snapshot deserializes successfully
- status rendering completes without failure
- consumers that ignore the additive projection remain compatible

## 7. Verify Assistant Assets

Run the focused assistant contract tests:

```bash
cargo test --test contract assistant_command_definition_contract::
```

Expected result:

- Copilot, Claude, Codex, and Antigravity plan and run assets preserve the
  backlog-quality fields
- each host keeps the planning-stage resume path when backlog quality is not
  ready
- no host invents a direct run continuation while backlog quality is blocked or
  requires clarification

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

Then list the actual changed or created implementation files and intersect
their diff with uncovered LCOV lines:

```bash
implementation_files=(
  src/domain/governance.rs
  src/domain/goal_plan.rs
)
git diff --unified=0 origin/main...HEAD -- "${implementation_files[@]}" \
  | python3 scripts/common/coverage/intersect_patch_coverage.py \
      --lcov lcov.info "${implementation_files[@]}"
```

Expected result:

- formatting passes
- clippy reports zero warnings
- focused and workspace-relevant tests pass
- changed Rust implementation files meet at least 95% changed-file coverage
- release metadata, Canon `0.67.0` compatibility wording, README, docs, tech
  docs, changelog, and assistant metadata consistently describe release
  `0.69.0`
