---
description: "Summarize an active Boundline workflow"
---

# Command: /boundline-workflow-status

Shared guidance: `assistant/README.md`

## Intent
Summarize the active named workflow without leaving the primary Boundline product
surface.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- workflow status --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin boundline -- workflow status --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Summarize `workflow`, `workflow_phase`, `workflow_next_action`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, `continuity_authority`, any `compatibility_follow_up`, any `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, any governance wait-or-block guidance, and the CLI-reported `next_command`. Preserve `effective_routing`, `assistant_bindings`, `runtime_capabilities`, and `slot_effort_policies` when surfaced.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if the workflow remains active, route to `/boundline-workflow-resume`, and if the workflow is terminal or inspect-only, route to `/boundline-workflow-inspect`.
Allowed follow-up commands: `/boundline-workflow-resume`, `/boundline-workflow-inspect`, `/boundline-workflow-run`, `/boundline-status`, `/boundline-inspect`.