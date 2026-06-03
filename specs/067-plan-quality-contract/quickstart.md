# Quickstart: Plan Quality Contract

## 1. Use An Isolated Temporary Workspace

Do not run Boundline CLI commands against the Boundline repository root. Create
or use a disposable fixture workspace for every runtime validation scenario.

Expected result:

- the Boundline source tree remains free of workspace-local `.boundline/`
  session state
- each scenario has its own isolated session and traces

## 2. Verify Ready Planning

Capture a bounded goal, provide planning input that includes a rationale and an
explicit validation strategy, then request planning and status.

Expected result:

- `plan_quality_state` is `ready`
- no blocking plan-quality finding remains
- accepted low-impact assumptions remain visible when applicable
- execution handoff may be offered only after later planning gates also pass

## 3. Verify Missing Validation Strategy Blocks Handoff

Use a focused fixture or test helper to create a plan with goal quality
satisfied and an empty validation strategy, then request the next planning or
execution-admission step.

Expected result:

- `plan_quality_state` is `clarification_required`
- `plan_quality_findings` includes `verification_strategy`
- exactly one `phase_request` is emitted
- the session remains non-terminal and execution handoff is withheld
- status and trace output preserve the finding and accepted assumptions

## 4. Verify Recovery Uses The Same Session

Answer the emitted question with an explicit validation strategy and resume
through the provided continuation.

Expected result:

- the same session is re-evaluated
- the earlier blocked assessment remains trace-visible
- the effective assessment transitions to `ready` when no blocking finding
  remains
- execution handoff is offered only after later planning gates also pass

## 5. Verify Compatibility

Load a fixture session snapshot that predates the additive `plan_quality`
projection and inspect status.

Expected result:

- the older snapshot deserializes successfully
- status rendering completes without failure
- consumers that ignore the additive projection remain compatible

## 6. Verify Assistant Assets

Run the focused assistant contract tests.

```bash
cargo test --test contract assistant_command_definition_contract::
```

Expected result:

- Copilot, Claude, Codex, and Antigravity planning assets contain the
  standardized sections
- each host preserves the plan-quality fields and structured recovery routes
- no host invents execution continuation while quality is blocked

## 7. Validate Release Closure

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test unit
cargo test --test contract
cargo test --test integration human_input_capture_flow::
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Then list the actual changed or created implementation files and intersect
their diff with uncovered LCOV lines:

```bash
implementation_files=(
  src/domain/goal_plan.rs
  src/domain/session.rs
  src/orchestrator/session_runtime.rs
  src/orchestrator/session_runtime_native_goal_plan.rs
  src/orchestrator/session_runtime_planning_runtime.rs
  src/cli/session.rs
  src/cli/output_session_status.rs
  src/cli/output_orchestrate.rs
  src/cli/inspect/projections.rs
  src/cli/output_run_trace.rs
)
git diff --unified=0 origin/main...HEAD -- "${implementation_files[@]}" \
  | python3 scripts/common/coverage/intersect_patch_coverage.py \
      --lcov lcov.info "${implementation_files[@]}"
```

Expected result:

- formatting passes
- clippy reports zero warnings
- focused and workspace-relevant tests pass
- changed or created implementation files meet at least 95% patch coverage
- release metadata, README, docs, tech docs, changelog, roadmap status, and
  assistant metadata consistently describe release `0.67.0`
