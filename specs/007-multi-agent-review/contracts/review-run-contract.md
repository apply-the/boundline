# Contract: Run Command Review Output

## Purpose

Define the minimum observable behavior of `boundline run` when the bounded review phase is active.

## Invocation

```bash
cargo run --bin boundline -- run --goal "<goal>" --workspace <workspace>
```

or, after a planned session exists:

```bash
cargo run --bin boundline -- run --workspace <workspace>
```

## Success contract

When a review phase runs and the result is accepted, the terminal output MUST include, at minimum:

- `goal: <goal>`
- one or more delivery step lines
- one or more review step lines naming the participating reviewers
- `review_trigger: <trigger>`
- `review_vote: <summary>`
- `review_outcome: accepted`
- `terminal_status: succeeded`
- `trace: <absolute trace path>`

The initial slice runs review at most once after the current run reaches a reviewable terminal delivery result.

If review findings are available, the output SHOULD also include:

- the highest-severity finding headline
- whether adjudication ran
- a next command or follow-up guidance line

## Non-success contract

When a review phase rejects, escalates, or fails, the terminal output MUST include:

- the same `goal`, `trace`, and `review_trigger` fields
- `review_outcome: rejected`, `review_outcome: escalated`, or `review_outcome: failed`
- visible evidence that explains why the result was not accepted
- `terminal_status: failed` or `terminal_status: exhausted` when the delivery run terminates non-successfully
- follow-up guidance through `next_command`

When reviewer participation is incomplete, the output SHOULD make visible which reviewers failed or were omitted.

## Omission contract

When review is not configured or not triggered for the current run:

- review-specific output fields MUST be omitted cleanly
- the existing execution-engine run output contract MUST remain valid
