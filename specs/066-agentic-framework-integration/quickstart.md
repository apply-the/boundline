# Quickstart: Agentic Framework Integration

## 1. Prepare A Temporary Workspace And The Three Repositories

Use a disposable workspace for Boundline lifecycle validation. Do not run this
walkthrough against the Boundline repository root, because the feature persists
workspace-local state under `.boundline/`.

Expected result:

- the Boundline host repo remains the runtime and contract source of truth
- `../boundline-framework-template/` is available as the reusable adapter
  scaffold repo
- `../boundline-adapter-speckit/` is available as the concrete Speckit adapter
  repo

Optional bootstrap validation:

```bash
cargo test --manifest-path ../boundline-framework-template/Cargo.toml --no-run
cargo test --manifest-path ../boundline-adapter-speckit/Cargo.toml --no-run
```

Expected bootstrap result:

- both sibling repositories compile as standalone Rust crates
- the template scaffold exposes the planned `framework-adapter-v1` contract line
- the Speckit scaffold exposes adapter ID `speckit` and binary name
  `boundline-adapter-speckit`

## 2. Verify The Default No-Adapter Path Still Uses Built-In Behavior

Run:

```bash
boundline init --workspace <workspace>
boundline goal --workspace <workspace> --goal "Ship a bounded feature"
boundline plan --workspace <workspace>
boundline status --workspace <workspace>
```

Expected result:

- no adapter selection is required to complete the baseline workflow
- `.boundline/config.toml` contains no active adapter block
- stage output and status continue to show built-in Canon-aware behavior as the
  execution source

## 3. Register The Known Speckit Profile Explicitly

Run:

```bash
boundline adapter add speckit --workspace <workspace>
boundline adapter show --workspace <workspace>
boundline adapter show --workspace <workspace> --json
```

Expected result:

- guided setup offers the known Speckit profile and resolves the default binary
  `boundline-adapter-speckit`
- PATH discovery may prefill the command, but the adapter becomes active only
  after the explicit `adapter add` action completes
- `.boundline/config.toml` gains an `[adapter]` selection block plus the stored
  resolved field values for the active schema
- JSON output shows adapter ID `speckit`, the compatibility line, supported
  transports declaring V1 JSON over stdin/stdout, declared stage overrides,
  declared hook subscriptions, and the resolved config-completeness state
- the adapter show report is enough to confirm V1 transport compatibility before
  `plan` or `run` invokes a claimed stage

## 4. Run The Lifecycle With Speckit Enabled

Run:

```bash
boundline plan --workspace <workspace>
boundline run --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:

- Speckit only owns the stages it declared in its capability manifest, and the
  corrected known-profile map is explicit: `goal` stays native, `plan` maps to
  workflow ID `speckit-planning`, `run` maps to workflow ID
  `speckit-implementation`, and `status` plus `inspect` remain Boundline-owned
  surfaces
- the claimed Speckit stages enter through the split workflow assets
  `.specify/workflows/speckit/planning.yml` and
  `.specify/workflows/speckit/implementation.yml`, while the response payloads
  and Boundline status surfaces continue to report the semantic workflow IDs
- the adapter bridge, not the YAML asset alone, enforces conditional clarify,
  the mandatory `speckit.analyze` planning-readiness loop, bounded remediation
  or analyze re-check accounting, and run-stage validation or status capture
- a claimed `plan` stage runs the Speckit planning lifecycle, including the
  mandatory `speckit.analyze` readiness gate, and it must not be considered
  complete while blocking findings remain unresolved
- when analyze finds blocking findings, the adapter either executes bounded
  remediation work and re-runs analyze or returns a blocked outcome after at
  most two remediation or analyze re-check cycles in the same claimed `plan`
  attempt
- a claimed `run` stage invokes implementation-only behavior through
  `speckit.implement` plus implementation validation or status capture; it does
  not rerun planning commands
- `inspect` and the underlying session and trace data show the stage execution
  source (`built_in` vs `adapter`), workflow ID, adapter ID, produced artifact
  refs, findings or validation summaries, and hook-delivery outcomes
- if the adapter emits structured stderr during preflight, stage execution, or
  hook delivery, Boundline may capture parseable lines into traces without
  changing the command outcome decided by the stdout response envelope
- V1 execution remains one-shot stdio only; there is no graceful-shutdown or
  long-lived adapter lifecycle to validate in this slice

## 5. Verify Non-Interactive Missing-Config Handling

Validate both registration-time and runtime handling:

```bash
boundline adapter add custom --workspace <workspace> --id custom-demo --command /path/to/adapter --non-interactive
```

Then, in a workspace that already selected Speckit, clear one required stored
field and rerun `plan` or `run`.

Expected result:

- non-interactive `adapter add` blocks before activation and names the missing
  field or fields plus the recovery command
- if an already selected adapter later returns a blocked preflight, Boundline
  keeps the pre-claim boundary explicit through the reported fallback reason and
  does not let the adapter silently claim the `plan` or `run` stage
- the host never prompts implicitly when `non_interactive = true`

## 6. Verify Post-Claim Adapter Failure Stops The Run

Use a fixture Speckit adapter build or test mode that fails after claiming one
declared stage, then run:

```bash
boundline run --workspace <workspace>
boundline status --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:

- the currently claimed stage is marked failed
- the lifecycle stops immediately and requires operator intervention before it
  can continue
- Boundline does not silently revert that stage to built-in behavior after the
  adapter has already claimed ownership

## 7. Bootstrap And Validate The Template And Speckit Repositories

After the feature lands, run the sibling repo checks:

```bash
cd ../boundline-framework-template
cargo test

cd ../boundline-adapter-speckit
cargo test
```

Expected result:

- the template repo builds from a real scaffold instead of its current empty Git
  state
- the Speckit repo builds the `boundline-adapter-speckit` binary against the
  same protocol line used by the host
- both repos pin a released `boundline-adapters` reference rather than a local
  path dependency

## 8. Confirm Compatibility And Release Signals Stay Visible

Run:

```bash
boundline adapter show --workspace <workspace> --json
```

Expected result:

- the report shows the active protocol line, the adapter version, and the
  supported Boundline version range
- the report surfaces the adapter's declared supported transports so V1 JSON
  over stdin/stdout compatibility is inspectable without invoking a stage
- mismatched compatibility metadata is surfaced as a blocked or unsupported
  state before execution starts
- V1 validation remains bounded to one-shot stdio exchanges; no graceful
  shutdown or other long-running transport lifecycle flow is expected here
- release docs in the Boundline, template, and Speckit repos can point to the
  same compatibility line without requiring lockstep releases

## 9. Post-Correction Validation Snapshot

The corrected stage-map slice was rerun on 2026-06-01 after the split
workflow-entrypoint clarification, the retirement of the legacy combined
workflow surface, and the final Phase 6 documentation closure.

Validated commands:

```bash
cargo test --test contract framework_adapter_protocol_contract::
cargo test --test contract runtime_routing_contract::
cargo test --test integration framework_adapter_activation::cross_repo_speckit_binary_smoke_bridges_real_specify_plan_and_completes_run
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --manifest-path ../boundline-framework-template/Cargo.toml --test contract
cargo fmt --manifest-path ../boundline-framework-template/Cargo.toml --all --check
cargo clippy --manifest-path ../boundline-framework-template/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path ../boundline-adapter-speckit/Cargo.toml --test contract
cargo test --manifest-path ../boundline-adapter-speckit/Cargo.toml --test override_flow
cargo test --manifest-path ../boundline-adapter-speckit/Cargo.toml --test config_flow
cargo fmt --manifest-path ../boundline-adapter-speckit/Cargo.toml --all --check
cargo clippy --manifest-path ../boundline-adapter-speckit/Cargo.toml --all-targets -- -D warnings
```

Observed corrected evidence:

- the host still rejects unsupported transport declarations before plan or run
  stage claim and records the explicit fallback reason
- the standard stdout envelope remains authoritative even when adapters emit
  structured stderr alongside in-band failures or protocol errors
- the real cross-repo Speckit smoke now proves workflow ID
  `speckit-planning` for the claimed plan stage, workflow ID
  `speckit-implementation` for the claimed run stage, and implementation-only
  run behavior with Boundline-owned visibility over artifacts and validation
  refs
- the template and Speckit sibling repos still declare the same V1 stdio JSON
  transport and bounded one-shot command set as the host
- the split workflow entrypoints under `.specify/workflows/speckit/` are now
  the only live bundled Speckit workflow surfaces; the legacy combined workflow
  asset, registry entry, and hidden run-stage fallback were retired before the
  final rerun
- the Speckit bridge continues to treat clarify when required, the mandatory
  `speckit.analyze` readiness gate, bounded remediation or analyze re-check
  loops, and run-stage validation or status capture as explicit bridge-owned
  behavior instead of hidden workflow engine control flow
- no graceful-shutdown lifecycle exists in the released protocol line, so the
  validated surface remains limited to one-shot `describe`, `preflight`,
  `execute-stage`, and `emit-hook`
- the provider-catalog refresh result remained no-change for this release line;
  the current model families already matched the catalog refresh landed on
  2026-05-30
- a read-only `/speckit.analyze` rerun passed for the corrected 066 packet
  slice, leaving only release-packaging follow-through rather than any
  remaining semantic blocker