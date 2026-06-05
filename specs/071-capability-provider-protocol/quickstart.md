# Quickstart: External Capability Provider Protocol

## 1. Use An Isolated Temporary Workspace

Do not run Boundline CLI commands against the Boundline repository root. Create
or use disposable fixture workspaces for every provider-protocol validation
scenario.

Expected result:

- the Boundline source tree remains free of workspace-local `.boundline/`
  session state
- each scenario has isolated provider registration, activation, trace, and
  evidence records

## 2. Verify Discovery Does Not Activate A Provider

Create or simulate a provider that is discoverable on the machine but has not
been explicitly registered.

Expected result:

- the provider can appear in discovery output
- the provider remains inactive and unavailable for execution
- no provider-backed run is admitted on discovery alone

## 3. Verify Interrupted Setup Preserves The Previous Active Provider

Create a workspace with one already active provider, then start registration or
activation of a replacement provider and interrupt the flow before setup
completes.

Expected result:

- the replacement provider is not marked active
- the previous provider remains the authoritative active provider
- traces or status output record the interrupted activation clearly

## 4. Verify Unavailable Providers Are Blocked Before Execute

Create a provider fixture that passes registration but fails health or becomes
unavailable before execution begins.

Expected result:

- provider-backed execution is blocked before `execute`
- the runtime records a readiness failure rather than a generic execution
  failure
- `status` or `inspect` surfaces the blocked reason without opening raw trace
  files

## 5. Verify Prepare Reports Missing Context And Evidence

Use a provider fixture that requires specific context or evidence before it can
execute safely.

Expected result:

- `prepare` reports required context, optional context, missing evidence, and
  expected artifacts
- missing required context or evidence blocks admission or forces an explicit
  degraded path
- no silent fallback invents absent provider inputs

## 6. Verify Permission Admission Is Explicit And Least Privilege

Use a fixture where the provider declares broader required permissions than the
current Boundline stage policy should allow.

Expected result:

- the request includes an explicit permission envelope
- the runtime blocks or degrades the request before `execute` if the envelope
  is insufficient or conflicts with runtime policy
- the resulting failure is classified as a permission admission failure

## 7. Verify Execute And Collect-Evidence Stay Non-Authoritative

Use a provider fixture that returns findings, artifacts, evidence refs, and
patch proposals.

Expected result:

- `execute` returns structured provider output
- `collect_evidence` normalizes claims, evidence refs, limitations, and
  reproducibility metadata
- provider proposals do not directly mutate Boundline-owned state without an
  explicit validation disposition

## 8. Verify Conflict Rule Fails Closed

Create a fixture where provider metadata, specialized profile metadata, or
Boundline runtime policy disagree on capability identity, lifecycle support,
permissions, or evidence requirements.

Expected result:

- the stricter Boundline runtime policy wins
- provider-backed execution fails closed before `execute`
- the failure is inspectable as a metadata or policy conflict, not a silent
  merge

## 9. Verify Inspectable Provider State

Run one successful and one blocked provider-backed scenario.

Expected result:

- `status` or `inspect` shows provider identity, activation state, capability,
  validation disposition, evidence refs, and limitations
- operators can distinguish readiness, permission, execution, and
  post-execution validation failures from those surfaces alone

Manual validation for the operator-facing explanation criterion:

- start from `boundline status` or `boundline inspect` output only
- do not open raw provider payload files or trace documents
- confirm within 30 seconds why one provider-backed run was accepted or blocked

## 10. Validate Release Closure

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test unit
cargo test --test contract
cargo test --test integration capability_provider_activation_flow::
cargo test --test integration capability_provider_execution_flow::
cargo test --test integration host_session_runtime_flow::
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Then intersect the changed implementation files with uncovered LCOV lines.
Adjust the file list to the actual diff, but expect the provider-protocol slice
to touch files in a group like:

```bash
implementation_files=(
  src/domain/capability_provider.rs
  src/domain/configuration.rs
  src/domain/session.rs
  src/adapters/capability_provider_runtime.rs
  src/adapters/capability_provider_runtime/command.rs
  src/adapters/capability_provider_runtime/http.rs
  src/orchestrator/capability_provider_runtime.rs
  src/orchestrator/session_runtime.rs
  src/cli/provider.rs
  src/cli/inspect/projections.rs
  src/cli/output_host.rs
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
- focused provider-protocol tests pass
- changed Rust implementation files meet at least 95% changed-file coverage
- docs, assistant assets, changelog, version metadata, and Canon compatibility
  guidance consistently describe release `0.72.0`
