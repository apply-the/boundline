# Contract: Run Command Output

## Purpose

Define the minimum observable behavior of `boundline run` when the execution engine is active.

## Invocation

```bash
cargo run --bin boundline -- run --goal "<goal>"
```

or, after a planned session exists:

```bash
cargo run --bin boundline -- run
```

## Success contract

The terminal output MUST include, at minimum:

- `goal: <goal>`
- one or more step lines showing analysis, change application, and validation
- `terminal_status: succeeded`
- `terminal_reason: <reason>`
- `trace: <absolute trace path>`
- `next_command: /boundline-status` or equivalent post-success guidance

If change evidence is available, the output SHOULD also include:

- one or more changed-file lines
- the latest validation outcome

## Non-success contract

When the run fails or exhausts its limits, the terminal output MUST include:

- the same `goal`, `trace`, and `terminal_reason` fields
- `terminal_status: failed` or `terminal_status: exhausted`
- visible evidence that the latest delivery attempt failed, including either change evidence, validation evidence, or both
- `next_command: /boundline-next` or equivalent recovery guidance

## Exit status contract

- Successful delivery returns process exit code `0`.
- Failed or exhausted delivery returns process exit code `1`.
- Invalid invocation returns process exit code `2`.
- Trace-read failures continue to use process exit code `3` when inspection output cannot be loaded.