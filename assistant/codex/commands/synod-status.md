# Command: /synod-status

Shared guidance: `assistant/README.md`

## Intent
Summarize the active session state for a workspace without re-inspecting a trace by default.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
Run `cargo run --bin synod -- status --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin synod -- status --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Summarize `latest_status`, `current_step_id`, any `latest_trace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if the session is missing or invalid, route to `/synod-start`.
Allowed follow-up commands: `/synod-next`, `/synod-inspect`, `/synod-step`, `/synod-plan`, `/synod-start`, `/synod-status`.