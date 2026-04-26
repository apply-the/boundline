# Contract: Workspace Execution Profile

## Purpose

Define the workspace-local contract that powers the execution engine.

## Location

- Preferred: `<workspace>/.synod/execution.json`
- Legacy fallback: `<workspace>/.synod/fixture.json`

## Preferred JSON shape

```json
{
  "name": "red-to-green-execution",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "limits": {
    "max_replans": 1
  },
  "attempts": [
    {
      "attempt_id": "fix-add",
      "summary": "Replace subtraction with addition",
      "failure_mode": "replan",
      "changes": [
        {
          "path": "src/lib.rs",
          "find": "left - right",
          "replace": "left + right"
        }
      ]
    }
  ]
}
```

## Required behavior

- `name` MUST be non-empty.
- `validation_command.program` MUST be non-empty.
- `attempts` MUST contain at least one attempt.
- Every `path` MUST be relative to the workspace root.
- Every `find` field MUST be non-empty.
- `limits` MAY be omitted entirely, and partial `limits` objects MUST inherit defaults for omitted fields.
- `failure_mode` MUST map to an explicit recovery behavior supported by the orchestrator.
- Legacy fixture manifests MUST be accepted and converted into the execution-profile model.

## Legacy fixture compatibility

Accepted legacy shape:

```json
{
  "name": "red-to-green",
  "test_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "file_patches": [
    {
      "path": "src/lib.rs",
      "find": "left - right",
      "replace": "left + right"
    }
  ]
}
```

Legacy conversion rules:

- `test_command` maps to `validation_command`.
- `file_patches` become a single attempt with `attempt_id = "legacy-attempt-1"`.
- The runtime MUST preserve the observable behavior of the previous fixture-backed slice for the same workspace.