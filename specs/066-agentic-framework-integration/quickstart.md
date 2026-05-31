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
- JSON output shows adapter ID `speckit`, the compatibility line, declared stage
  overrides, declared hook subscriptions, and the resolved config-completeness
  state

## 4. Run The Lifecycle With Speckit Enabled

Run:

```bash
boundline plan --workspace <workspace>
boundline run --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:

- Speckit only owns the stages it declared in its capability manifest
- undeclared stages continue to run through the built-in path
- `inspect` and the underlying session and trace data show the stage execution
  source (`built_in` vs `adapter`), adapter ID, and hook-delivery outcomes

## 5. Verify Non-Interactive Missing-Config Failure

Create a new temporary workspace or clear one required Speckit field from the
stored adapter config, then invoke the lifecycle from a non-interactive host
surface.

Expected result:

- the run fails before any adapter-owned stage begins
- the operator-visible error names the missing field or fields, the adapter ID
  `speckit`, and the recovery path such as `boundline adapter add speckit`
- the host does not silently fall back, skip adapter-owned stages, or prompt
  implicitly

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
- mismatched compatibility metadata is surfaced as a blocked or unsupported
  state before execution starts
- release docs in the Boundline, template, and Speckit repos can point to the
  same compatibility line without requiring lockstep releases