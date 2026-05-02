# Command: /synod-workflow-resume

Shared guidance: `assistant/README.md`

## Intent
Resume the active named workflow on the same bounded Synod path.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin synod -- workflow resume --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin synod -- workflow resume --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, any `follow_through_guidance`, any governance wait-or-block guidance, and the CLI-reported `next_command`. Preserve `effective_routing` and `assistant_bindings` when surfaced, and treat assistant-binding mismatch output as a stop condition.

## Next-Step Routing
Prefer the CLI-reported `next_command`; route to `/synod-workflow-inspect` when the workflow reaches terminal or inspect-only follow-through.
Allowed follow-up commands: `/synod-workflow-status`, `/synod-workflow-inspect`, `/synod-workflow-resume`, `/synod-status`, `/synod-inspect`.