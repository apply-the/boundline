# Command: /synod-step

Shared guidance: `assistant/README.md`

## Intent
Advance an active Synod workflow by choosing one explicit next action from the current context.

## Required Context
- Confirmed workflow goal, or pasted inspection output that reveals the current state
- Workspace reference when the next action depends on the latest recorded trace

## Shell-Enabled Path
No direct CLI invocation is required by default. Use the confirmed context to choose exactly one next command. If the current state is unclear but the workspace is known, route to `/synod-status` or `/synod-next` instead of inventing state.

## Chat-Only Path
Ask only for the missing workflow context or for pasted output from `cargo run --bin synod -- inspect --workspace <workspace>`. Once enough evidence is available, recommend one concrete next command.

## Output Interpretation
Summarize the current state in one short paragraph, then state the single next bounded action.

## Next-Step Routing
Allowed follow-up commands: `/synod-run`, `/synod-status`, `/synod-next`, `/synod-inspect`, `/synod-start`.