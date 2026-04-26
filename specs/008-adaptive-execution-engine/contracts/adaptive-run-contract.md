# Contract: Adaptive Run Command Output

## Purpose

Define the minimum observable behavior of `synod run` when adaptive execution is active.

## Invocation

```bash
cargo run --bin synod -- run --goal "<goal>" --workspace <workspace>
```

or, after a planned session exists:

```bash
cargo run --bin synod -- run --workspace <workspace>
```

## Success contract

When adaptive execution succeeds, terminal output MUST include, at minimum:

- `goal: <goal>`
- one or more delivery step lines
- `workspace_slice: <summary>`
- `attempt_lineage: <summary>` for any attempt after the first
- `changed_files: <summary>`
- `validation: passed` or equivalent validation summary
- `terminal_status: succeeded`
- `trace: <absolute trace path>`

If review is configured and triggered, the existing review output contract from Spec 007 remains valid.

## Non-success contract

When adaptive execution fails or exhausts its bounded options, terminal output MUST include:

- the same `goal` and `trace` fields
- the latest `workspace_slice` summary
- visible evidence of why the current path failed or why no credible next candidate remained
- `terminal_status: failed` or `terminal_status: exhausted`
- `next_command`

## Omission contract

When adaptive execution is not active for the current profile:

- adaptive-specific fields MUST be omitted cleanly
- the existing attempt-based run output contract remains valid
