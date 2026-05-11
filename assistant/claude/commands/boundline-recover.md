# Command: /boundline-recover

Shared guidance: `assistant/README.md`

## Intent
Recover from a Boundline session that is blocked, clarification-required, failed, exhausted, or terminal by asking the real runtime for current state first.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
Run `cargo run --bin boundline -- status --workspace <workspace> --json` exactly once. If the output reports a `latest_checkpoint_restore_command`, `corrected_command`, or `next_command`, use that command as the recovery path. If status is insufficient, run `cargo run --bin boundline -- inspect --workspace <workspace> --json` exactly once and preserve its guidance.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin boundline -- status --workspace <workspace> --json`

Wait for pasted output before recommending recovery.

## Output Interpretation
`.boundline/session.json` remains authoritative. Do not infer recovery from chat history. Preserve `next_command`, checkpoint restore guidance, and explicit blocked, clarification-required, failed, exhausted, or terminal wording.
