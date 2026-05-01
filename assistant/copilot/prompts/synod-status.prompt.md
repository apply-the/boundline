---
description: "Summarize the latest known status of a Synod workflow"
---

# Command: /synod-status

Shared guidance: `assistant/README.md`

## Intent
Summarize the active session state or latest compatibility follow-up for a workspace without re-inspecting a trace by default.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
Run `cargo run --bin synod -- status --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin synod -- status --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Summarize `latest_status`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `compatibility_follow_up_command`, `execution_path`, `flow_state`, `latest_decision_status`, `latest_decision_target`, `current_step_id`, any `latest_trace_ref`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, governance mode, run-ref, packet provenance, and `governance_next_action` when present, plus the CLI-reported `next_command`. Preserve `effective_routing` and `assistant_bindings` when they appear inside `route_config_projection`.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if status reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/synod-inspect`. Route to `/synod-start` only when the CLI reports no active session and no compatibility follow-up.
Allowed follow-up commands: `/synod-next`, `/synod-inspect`, `/synod-step`, `/synod-plan`, `/synod-start`, `/synod-status`.