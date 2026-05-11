# Command: /boundline-govern

Shared guidance: `assistant/README.md`

## Intent
Use optional Canon governance only when the active Boundline workspace is configured for it.

## Required Context
- `workspace_ref`
- Desired governed mode only if the user already provided one

## Shell-Enabled Path
Run `cargo run --bin boundline -- config show --workspace <workspace> --scope workspace --json` exactly once. If Canon governance is configured and the user supplied a mode, use the CLI-supported governed surface such as `cargo run --bin boundline -- run --workspace <workspace> --mode <mode> --json`. If governance is absent, report that Canon governance is conditional and route back to the CLI-reported `next_command`.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin boundline -- config show --workspace <workspace> --scope workspace --json`

Wait for pasted output before discussing governance.

## Output Interpretation
`.boundline/session.json` remains authoritative for delivery state. Canon governance is conditional and must not appear as the normal delivery path when not configured. Preserve `next_command` and blocked, clarification-required, failed, exhausted, or terminal state before invoking governed actions.
