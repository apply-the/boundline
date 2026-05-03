# Contract: Governance Execution Profile

## Purpose

Define the workspace manifest shape added to `<workspace>/.boundline/execution.json` so Boundline can configure stage-scoped local or Canon-backed governance without introducing a second workspace manifest.

## JSON Shape

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
      "default_owner": "boundline",
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

## Required Validation Rules

- `governance.stages[*].flow_name` must be a built-in Boundline flow.
- `governance.stages[*].stage_id` must be a valid stage of the selected flow.
- Duplicate `(flow_name, stage_id)` stage policies are invalid.
- `required: true` implies `enabled: true`.
- `autopilot: true` implies `enabled: true`.
- `runtime: "canon"` requires a valid `governance.canon` block and a `canon_mode` that is allowed for the `(flow_name, stage_id)` pair in the first-slice whitelist, unless exactly one compliant mode exists and Boundline can derive it deterministically.
- `system_context: "existing"` means the stage is grounded in the current repository or an earlier governed packet; `system_context: "new"` means the stage is grounded only in a newly authored governed brief.
- For the first slice, `change`, `backlog`, `implementation`, `verification`, and `pr-review` policies must bind `system_context: "existing"`; `requirements`, `discovery`, and `architecture` may bind `new` or `existing` when the selected Canon mode permits it.
- If Canon defaults are omitted, the stage policy must provide the required `owner`, `risk`, `zone`, or `system_context` values before the stage can proceed.

## Runtime Semantics

- If `governance` is absent, Boundline behaves exactly as it does today.
- If `governance` is present and a current stage has no matching stage policy, Boundline uses `default_runtime` only when the implementation explicitly treats the stage as governed; otherwise the stage remains ungoverned.
- If a stage is governed and `required` is `true`, Boundline must block explicitly when no compliant governance path exists.
- If a stage is governed and `required` is `false`, Boundline may fall back to the local runtime only when that fallback is surfaced explicitly in session and trace output.
- Manifest loading must reject invalid stage-to-mode combinations before session execution begins.
- Governed packet reuse is built into Boundline's stage semantics for the first slice: a stage may reuse only the newest reusable packet from the same stage on rerun or from the immediately previous stage in the same built-in flow.