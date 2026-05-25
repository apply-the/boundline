# Command: /boundline-workflow-resume

Shared guidance: `assistant/README.md`

## Intent
Resume the active named workflow on the same bounded Boundline path.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- workflow resume --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin boundline -- workflow resume --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, any `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, any `follow_through_guidance`, any governance wait-or-block guidance, and the CLI-reported `next_command`. Preserve `effective_routing`, `assistant_bindings`, `runtime_capabilities`, and `slot_effort_policies` when surfaced, and treat delegated continuity output as a stop condition.

## Next-Step Routing
Prefer the CLI-reported `next_command`; route to `/boundline-workflow-inspect` when the workflow reaches terminal or inspect-only follow-through.
Allowed follow-up commands: `/boundline-workflow-status`, `/boundline-workflow-inspect`, `/boundline-workflow-resume`, `/boundline-status`, `/boundline-inspect`.