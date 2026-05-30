# Quickstart: Canon Governance Adapter

## Prerequisites

- Work from the repository root.
- Use a clean workspace or start with no active `.boundline/session.json`.
- Provide `.boundline/execution.json` in the target workspace.
- Install Canon only for the Canon-backed scenarios; the local-first scenario must still work without it.
- Run commands through `cargo run --bin boundline -- ...` when validating locally.

## Example governed execution profile

Create `.boundline/execution.json` in a small Rust workspace:

```json
{
  "name": "governed-bug-fix",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "attempts": [
    {
      "attempt_id": "fix-add",
      "summary": "Replace subtraction with addition",
      "failure_mode": "terminal",
      "changes": [
        {
          "path": "src/lib.rs",
          "find": "left - right",
          "replace": "left + right"
        }
      ]
    }
  ],
  "governance": {
    "default_runtime": "local",
    "canon": {
      "command": "canon",
      "default_owner": "developer",
      "default_risk": "medium",
      "default_zone": "repo",
      "default_system_context": "existing"
    },
    "stages": [
      {
        "flow_name": "bug-fix",
        "stage_id": "investigate",
        "enabled": true,
        "required": false,
        "autopilot": false,
        "runtime": "canon",
        "canon_mode": "discovery",
        "system_context": "existing",
        "risk": "low",
        "zone": "repo",
        "owner": "developer"
      },
      {
        "flow_name": "bug-fix",
        "stage_id": "implement",
        "enabled": true,
        "required": true,
        "autopilot": true,
        "runtime": "canon",
        "canon_mode": "implementation",
        "system_context": "existing",
        "risk": "medium",
        "zone": "repo",
        "owner": "developer"
      },
      {
        "flow_name": "bug-fix",
        "stage_id": "verify",
        "enabled": true,
        "required": false,
        "autopilot": false,
        "runtime": "canon",
        "canon_mode": "verification",
        "system_context": "existing",
        "risk": "low",
        "zone": "repo",
        "owner": "developer"
      }
    ]
  }
}
```

## Scenario 1: Local-first governance when Canon is unavailable or not selected

1. Start a workspace session:

   ```bash
   cargo run --bin boundline -- start
   cargo run --bin boundline -- goal --goal "Fix the failing add test"
   cargo run --bin boundline -- flow bug-fix
   cargo run --bin boundline -- plan
   cargo run --bin boundline -- step
   cargo run --bin boundline -- status
   ```

2. Configure the current governed stage for `runtime = local` or make Canon unavailable while the stage is not marked `required`.

Expected outcome:

- `status` shows `latest_governance_runtime: local`.
- The governed stage reaches `latest_governance_state: completed` or `governed_ready` without a Canon run reference.
- Boundline continues through the normal local execution path with explicit governance evidence.

## Scenario 2: Canon-backed governance and packet reuse

1. Configure a Canon-backed stage policy such as `delivery:requirements` or `bug-fix:investigate`.
2. Run the stage through the session flow or through a direct custom run.
3. Inspect the results:

   ```bash
   cargo run --bin boundline -- run
   cargo run --bin boundline -- inspect
   ```

Expected outcome:

- Boundline records `latest_governance_runtime: canon` and the selected Canon mode.
- The run or inspect output exposes `latest_governance_run_ref` and `latest_governance_packet_ref`.
- `status` also exposes `latest_governance_packet_source_stage` and `latest_governance_packet_binding_reason` when a later stage reuses the immediate-upstream governed packet.
- The governance timeline shows runtime selection, Canon start, and governed completion.
- Later stages can reuse the governed packet as bounded reasoning input instead of reconstructing stage context.

## Scenario 3: Governance required with autopilot and approval wait

1. Configure a stage such as `bug-fix:implement` with `required: true` and `autopilot: true`.
2. Trigger a run where the chosen Canon path requires explicit approval.

Expected outcome:

- Boundline records one explicit autopilot decision for the stage.
- The governed stage enters `latest_governance_state: awaiting_approval` instead of continuing locally.
- `status` and `inspect` continue to expose `latest_governance_mode`, `latest_governance_run_ref`, and autopilot candidates while approval is still pending.
- A later `status`, `step`, or `run` invocation refreshes approval state and only resumes the stage after the runtime reports `granted`.
- `next` and `inspect` guide the operator toward safe inspection rather than another ungoverned execution step.
- No stage execution continues past the approval boundary until it is explicitly resolved.

## Validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Expected outcome:

- Governance manifest parsing, runtime selection, session projections, trace rendering, and autopilot blocking scenarios all pass.
- Boundline remains executable when Canon is absent.
- Canon-backed scenarios surface governed run references and packet readiness explicitly.