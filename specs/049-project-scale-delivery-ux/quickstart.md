# Quickstart: Boundline Project-Scale Delivery UX

These checks describe how to validate the implemented feature once code work begins. They are not implementation instructions for the current specification-only turn.

## 1. Global Bootstrap In An Uninitialized Workspace

```bash
mkdir -p /tmp/boundline-uninitialized
cd /tmp/boundline-uninitialized
boundline assistant install --host codex --scope user
```

From the supported host, invoke:

```text
/boundline:doctor
/boundline:status
/boundline:continue
/boundline:init
```

Expected result:

- `doctor` reports Boundline install state, Canon pairing state, workspace readiness, and repair/init guidance.
- `status` says the workspace is not initialized and gives the exact CLI command.
- `continue` says no active session exists and does not infer state from chat history.
- `init` runs or provides the exact `boundline init` command.

## 2. Repo-Local Session State Remains Authoritative

```bash
boundline init --assistant codex
boundline status --json
```

Expected result:

- Repo-local assistant commands become available after initialization.
- CLI and assistant status report the same session state and next action.
- `.boundline/session.json` remains the state source.

## 3. Idea-To-Code Project-Scale Path

```bash
boundline start
boundline goal --goal "Build a customer onboarding capability with audit logging"
boundline plan
```

Expected result:

- Boundline proposes a bounded staged path.
- Discovery appears when the problem is unclear.
- Requirements appear when scope must be bounded.
- System-shaping and architecture appear when capability and structural boundaries matter.
- Backlog decomposes the work into implementation slices.
- Each implementation slice has its own checkpoint, validation expectation, trace, and next action.
- Boundline asks for confirmation before material stage transitions.

## 4. Explicit Governed Stage Work

```bash
boundline govern --mode architecture --brief docs/architecture.md
boundline govern --mode security-assessment --goal "Assess auth and audit logging risks"
boundline govern --mode pr-review --base refs/heads/main --head HEAD
```

Expected result:

- Boundline checks Canon `0.45.0` capabilities before governed execution.
- Supported modes route to Canon at stage boundaries and persist packet refs.
- Unsupported or unavailable modes stop with repair guidance.
- Approval-gated stages show approval state and next action.

## 5. Voting Trigger Boundaries

Exercise these scenarios with fixtures or integration tests:

- High-impact architecture decision.
- Implementation slice that exhausts validation retries.
- PR-ready diff with material risk.
- Low-risk local refactor with preserved-behavior evidence.

Expected result:

- High-risk scenarios trigger configured voting.
- Blocking findings prevent continuation unless adjudicated or explicitly overridden by allowed policy.
- `status`, `next`, and `inspect` show voting state and next action.
- Low-risk local work skips voting by default.

## 6. Delivery Pilot Model Docs

Review:

```text
README.md
docs/getting-started.md
docs/architecture.md
docs/delivery-model.md
docs/guides/assistant-plugin-packages.md
docs/review-voting.md
```

Expected result:

- Docs state: "Large work is supported by decomposition, not by unbounded autonomy."
- Docs explain `observe -> decide -> act -> verify -> update context`.
- Docs include the customer onboarding with audit logging example.
- Docs state stop conditions for insufficient context, blocked governance, exhausted validation, excessive risk, voting blocks, pending approval, and boundary overflow.

## 7. Final Verification

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected result:

- Formatting passes.
- Clippy emits no warnings or errors.
- Tests pass.
- Created or modified Rust files meet at least 95% coverage.
