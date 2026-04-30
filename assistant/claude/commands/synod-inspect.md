# Command: /synod-inspect

Shared guidance: `assistant/README.md`

## Intent
Inspect a specific or session-resolved Synod trace and summarize the outcome.

## Required Context
- `trace_ref` or `workspace_ref`
- Preserve any confirmed `latest_trace_ref` from prior turns

## Shell-Enabled Path
If `trace_ref` is known, run `cargo run --bin synod -- inspect --trace <trace>`. Otherwise, if `workspace_ref` is known, run `cargo run --bin synod -- inspect --workspace <workspace>`. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.

## Chat-Only Path
Ask only for the missing `trace_ref` or `workspace_ref`, then provide one exact copyable command:

`cargo run --bin synod -- inspect --trace <trace>`

or

`cargo run --bin synod -- inspect --workspace <workspace>`

Wait for pasted output before continuing. If workspace-based inspect reports a session error, route to `/synod-start`. If trace reading fails, ask for a corrected trace reference or workspace and provide the replacement inspect command.

## Output Interpretation
Summarize `inspection_target`, `trace`, `routing_summary`, `execution_condition`, `goal_plan_summary`, `decision_timeline`, `failure_evidence`, `terminal_status`, `terminal_reason`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources` when present, governance runtime, mode, run-ref, packet provenance, `governance_next_action` when present, `corrected_command` on failures, and the CLI-reported `next_command`.

## Next-Step Routing
If workspace-based inspect reports a session error, route to `/synod-start`. Otherwise prefer the CLI-reported `next_command`.
Allowed follow-up commands: `/synod-next`, `/synod-run`, `/synod-step`, `/synod-status`, `/synod-start`.