# Command: /synod-inspect

Shared guidance: `assistant/README.md`

## Intent
Inspect a specific or session-resolved Synod trace and summarize the outcome.

If the resolved workspace trace reports compatibility ownership, keep that explicit: it now means the prior direct run opted into `--compatibility`, not that plain `run --goal` defaults there.

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
Summarize `inspection_target`, `trace`, `routing_summary`, `route_owner`, `route_config_projection`, `execution_condition`, `goal_plan_summary`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `decision_timeline`, `failure_evidence`, `changed_files`, validation summaries, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, `terminal_status`, `terminal_reason`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources` when present, governance runtime, mode, run-ref, packet provenance, `governance_next_action` when present, `corrected_command` on failures, and the CLI-reported `next_command`. Preserve `effective_routing`, `assistant_bindings`, `runtime_capabilities`, and `slot_effort_policies` when the route projection includes the persisted execution snapshot. When the context or governance fields are Canon-grounded, preserve governed artifact refs, credibility, and stale-memory wording exactly.

## Next-Step Routing
If workspace-based inspect reports a session error, route to `/synod-start`. Otherwise prefer the CLI-reported `next_command`.
Allowed follow-up commands: `/synod-next`, `/synod-run`, `/synod-step`, `/synod-status`, `/synod-start`.