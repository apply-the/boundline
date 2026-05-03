# Quickstart: Broaden Bounded Adaptive Repair

## Goal

Exercise the complete `0.23.0` adaptive compatibility story: broader bounded mutation families, explicit candidate credibility, and explicit exhaustion follow-up.

## Prerequisites

- A workspace with `.boundline/execution.json` configured for adaptive compatibility execution.
- `read_targets` that include at least one source file and one test or validation-relevant file.
- A failing validation command that can be repaired by a bounded local edit rather than an open-ended refactor.

## Example Adaptive Manifest

```json
{
  "name": "adaptive-bounded-repair",
  "read_targets": ["src/lib.rs", "tests/adaptive_failure.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "attempts": [],
  "adaptive": {
    "max_selected_targets": 1,
    "max_generated_attempts": 6,
    "path_preferences": ["src/"],
    "allowed_change_kinds": [
      "arithmetic_swap",
      "comparison_flip",
      "boolean_flip"
    ]
  }
}
```

## Flow

1. Start from the main operator story and confirm the compatibility route is intentional.
2. Run the adaptive manifest-backed execution.
3. Inspect the selected workspace slice, credibility reason, and validation result.
4. If validation fails, continue until the bounded adaptive planner either selects a new credible candidate or reaches explicit exhaustion.
5. Verify that `status`, `next`, and `inspect` all describe the same latest adaptive state.

## Expected CLI Behavior

### Initial adaptive run

- `boundline run` reports compatibility routing, `execution_condition`, selected workspace slice, and the reason the first bounded candidate was credible.
- The trace persists the chosen mutation family, candidate signature, validation result, and attempt lineage.

### Replanned adaptive run

- A failed validation can shift the selected target or mutation family when the latest validation guidance makes another bounded candidate more credible.
- `boundline status` and `boundline next` both show the latest selection headline, the candidate credibility rationale, and the authoritative compatibility follow-up state when no active session is resumable.

### Explicit exhaustion

- When no remaining bounded candidate is credible or allowed, `boundline inspect` reports an explicit failed or exhausted terminal reason.
- `boundline next` points to the correct inspect-oriented compatibility follow-up rather than suggesting a hidden retry.

## Validation Checklist

- Broader bounded mutation families are visible in the adaptive configuration and trace summaries.
- Candidate credibility and rejection reasons remain visible across `run`, `status`, `next`, and `inspect`.
- Explicit exhaustion is distinguishable from plain validation failure on the last selected candidate.
- Session-native continuity remains the primary route, while adaptive repair remains explicit compatibility behavior.