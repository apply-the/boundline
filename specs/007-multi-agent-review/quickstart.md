# Quickstart: Multi-Agent Review & Voting

## Prerequisites

- Work from the repository root.
- Use a clean workspace or start with no active `.synod/session.json`.
- Provide `.synod/execution.json` in the target workspace with a bounded review configuration.
- Run commands through `cargo run --bin synod -- ...` when validating locally.

## Example review configuration

Create `.synod/execution.json` in a small Rust workspace:

```json
{
  "name": "reviewed-delivery",
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
  "review": {
    "triggers": ["pr_ready"],
    "reviewers": [
      {"reviewer_id": "safety", "role": "Safety", "source": "gpt", "weight": 2},
      {"reviewer_id": "maintainability", "role": "Maintainability", "source": "claude", "weight": 1}
    ],
    "vote_rule": {"strategy": "weighted", "reject_on_blocking": true},
    "adjudication": {"enabled": true, "reviewer_id": "lead"},
    "scenarios": [
      {
        "trigger": "pr_ready",
        "findings": [
          {"reviewer_id": "safety", "disposition": "approve", "summary": "No blocking issues"},
          {"reviewer_id": "maintainability", "disposition": "concern", "summary": "Add a follow-up cleanup task"}
        ]
      }
    ]
  }
}
```

## Scenario 1: Run delivery with review acceptance

1. Execute a delivery run against the workspace:

   ```bash
   cargo run --bin synod -- run --goal "Fix the failing add test" --workspace <workspace>
   ```

2. Inspect the latest trace:

   ```bash
   cargo run --bin synod -- inspect --workspace <workspace>
   ```

Expected outcome:

- `run` applies the configured change set and then enters the review phase.
- The terminal output includes reviewer headlines, vote summary, review trigger, review outcome, and the trace reference.
- `inspect` exposes the participating reviewers, findings, vote resolution, and final review decision.

## Scenario 2: Review disagreement resolved by adjudication

1. Configure a review scenario where reviewers disagree.
2. Enable adjudication in the review config.
3. Execute the run again.

Expected outcome:

- Synod records the initial conflicting findings.
- The first vote indicates that adjudication is required.
- One bounded adjudication step runs and produces the final review decision.
- The trace shows both the original vote and the adjudication result.

## Scenario 3: Review escalation after non-credible decision

1. Configure a trigger that produces unresolved disagreement or reviewer failure.
2. Execute the run and then inspect the workspace session.

Expected outcome:

- Synod does not silently approve the result.
- The run terminates with an explicit escalated or failed review outcome.
- `status`, `next`, and `inspect` surface the latest review outcome and the reason it was not accepted.

## Validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected outcome:

- All tests pass.
- Review-specific contract and integration scenarios pass.
- `lcov.info` is regenerated from the same command used in CI.
- The crate version and documentation are updated to reflect the review and voting slice.
