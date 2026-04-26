# Contract: Adaptive Execution Profile

## Purpose

Define the workspace-local execution manifest shape that enables adaptive execution without fixed pre-authored attempts.

## Location

- Embedded under `<workspace>/.synod/execution.json`

## Preferred JSON shape

```json
{
  "name": "adaptive-red-to-green",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "adaptive": {
    "max_selected_targets": 1,
    "max_generated_attempts": 4,
    "path_preferences": ["src/", "tests/"],
    "allowed_change_kinds": ["arithmetic_swap", "comparison_flip", "boolean_flip"]
  }
}
```

## Required behavior

- `name` MUST be present and non-empty.
- `validation_command.program` MUST be present and non-empty.
- `read_targets` MUST contain at least one workspace-relative path when `adaptive` is present.
- At least one of `attempts` or `adaptive` MUST be present.
- `adaptive.max_selected_targets` MUST be greater than zero.
- `adaptive.max_generated_attempts` MUST be greater than zero.
- `allowed_change_kinds`, when present, MUST use only the bounded vocabulary supported by the runtime.
- Adaptive execution MAY coexist with an existing `review` profile.
- Adaptive configuration MUST be rejected if any configured path escapes the workspace boundary.

## Omission behavior

- If `adaptive` is absent, the existing attempt-based execution profile remains valid.
- If `attempts` is absent but `adaptive` is present, the runtime MUST still be able to create an initial bounded delivery plan.
