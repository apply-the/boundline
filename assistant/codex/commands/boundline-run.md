# Command: /boundline-run

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Resume the active Boundline session through the selected runtime route until it reaches a terminal outcome.

## Required Context
- `workspace_ref`
- Active session state; do not ask for a new goal or brief path when authored input is already captured

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- run --workspace <workspace>` exactly once. If the active session has no captured goal, authored brief, or planned task, route to `/boundline-plan` or `/boundline-start` instead of inventing a new run command. If the user explicitly asks for direct manifest-backed compatibility behavior without relying on active session state, route them to the direct compatibility workflow in the shared guidance instead of reinterpreting `/boundline-run`.

## Chat-Only Path
If shell execution is unavailable, ask only for missing workspace context and then provide this exact copyable command:

`cargo run --bin boundline -- run --workspace <workspace>`

Wait for the user to paste the output before continuing.

## Output Interpretation
Summarize `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, `flow_state`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `terminal_status`, `terminal_reason`, `changed_files`, validation summaries, `trace`, `next_command`, any latest decision summary when surfaced, and any governance wait-or-block guidance, including mode, run-ref, packet provenance, and `governance_next_action` when present. Preserve the returned trace reference for later `/boundline-inspect` use. When `route_config_projection` includes `effective_routing`, `assistant_bindings`, `runtime_capabilities`, or `slot_effort_policies`, preserve those values exactly. If the CLI reports delegated continuity, treat it as a stop condition instead of retrying with another assistant family. When the context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. When the context or governance fields are Canon-grounded, preserve governed artifact refs, credibility, and stale-memory wording exactly.

## Next-Step Routing
Prefer the CLI-reported `next_command`; when inspection is needed, route to `/boundline-inspect`.
Allowed follow-up commands: `/boundline-inspect`, `/boundline-status`, `/boundline-next`, `/boundline-run`, `/boundline-plan`, `/boundline-start`.