# Command: /boundline-workflow-inspect

Shared guidance: `assistant/README.md`

## Intent
Inspect the current named workflow summary and any associated trace-backed
evidence without leaving the Boundline workflow surface.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- workflow inspect --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin boundline -- workflow inspect --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `inspection_target`, `trace`, any `continuity_authority`, any `compatibility_follow_up`, any `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, any `corrected_command`, any governance wait-or-block guidance, and the CLI-reported `next_command`. Preserve `effective_routing`, `assistant_bindings`, `runtime_capabilities`, and `slot_effort_policies` when surfaced.

## Next-Step Routing
Prefer the CLI-reported `next_command`; when workflow follow-through remains bounded, route to `/boundline-workflow-resume`.
Allowed follow-up commands: `/boundline-workflow-resume`, `/boundline-workflow-status`, `/boundline-inspect`, `/boundline-status`.