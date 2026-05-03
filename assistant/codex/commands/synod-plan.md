# Command: /synod-plan

Shared guidance: `assistant/README.md`

## Intent
Capture human-authored input and plan the active session into a bounded proposal.

## Required Context
- `workspace_ref`
- At least one authored input source: bounded goal text and/or workspace-relative Markdown brief path(s)

## Shell-Enabled Path
If the workspace and at least one authored input source are known, run the matching capture command exactly once, then run plan:

`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`
`cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- plan --workspace <workspace>`

Ask only for missing workspace or missing authored input. Reuse confirmed brief paths instead of asking for them again.

## Chat-Only Path
Ask only for the missing workspace or authored input, then provide the matching exact copyable capture command followed by plan:

`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`
`cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- plan --workspace <workspace>`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Summarize the captured goal or `authored_input_summary`, `authored_input_sources`, any requested governance intent, the resulting plan state, any proposed, confirmed, skipped, or absent `flow_state`, any `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, any CLI-reported confirm or clarification guidance, and the CLI-reported `next_command`. When planning also reports `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, or `context_staleness_reason`, preserve those fields exactly. If that context is Canon-grounded, also preserve governed artifact refs and stale-memory wording exactly and treat non-credible context as a real stop condition.

## Next-Step Routing
Prefer the CLI-reported `next_command`; when planning is waiting on plan confirmation, follow that CLI route instead of inventing `/synod-run`.
Allowed follow-up commands: `/synod-step`, `/synod-run`, `/synod-plan`, `/synod-start`.
