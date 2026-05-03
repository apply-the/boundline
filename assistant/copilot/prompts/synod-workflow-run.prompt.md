---
description: "Start a named Synod workflow"
---

# Command: /synod-workflow-run

Shared guidance: `assistant/README.md`

## Intent
Start a named workflow on the primary Synod session-native path and keep the
follow-through bounded.

## Required Context
- `workspace_ref`
- `workflow_name`
- Optional `goal`

## Shell-Enabled Path
If the workflow name and workspace are known, run `cargo run --bin synod -- workflow run <name> --workspace <workspace>` exactly once. Add `--goal "<goal>"` when the operator supplied fresh goal text for workflow start. If the user explicitly asks for compatibility behavior, keep that explicit instead of reinterpreting `/synod-workflow-run` as a compatibility command.

## Chat-Only Path
Ask only for the missing `workspace_ref` or `workflow_name`, then provide this exact copyable command:

`cargo run --bin synod -- workflow run <name> --workspace <workspace>`

Wait for pasted output before continuing.

## Output Interpretation
Summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, any `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, any `follow_through_guidance`, any governance wait-or-block guidance, and the CLI-reported `next_command`. Preserve `effective_routing`, `assistant_bindings`, `runtime_capabilities`, and `slot_effort_policies` when they appear inside `route_config_projection`. If the CLI reports delegated continuity, treat it as a real stop condition.

## Next-Step Routing
Prefer the CLI-reported `next_command`; when the workflow remains active, route to `/synod-workflow-resume`, and when the next bounded action is inspection, route to `/synod-workflow-inspect`.
Allowed follow-up commands: `/synod-workflow-resume`, `/synod-workflow-status`, `/synod-workflow-inspect`, `/synod-status`, `/synod-inspect`.