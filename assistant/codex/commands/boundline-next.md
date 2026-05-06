# Command: /boundline-next

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Ask Boundline for the next recommended session command.

Compatibility follow-up means the user previously chose `boundline run --compatibility ...`; plain direct `run --goal` is native-first in `0.42.0`.

## Required Context
- `workspace_ref`
- Latest known outcome when already available

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- next --workspace <workspace>` exactly once and use the reported recommendation.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin boundline -- next --workspace <workspace>`

Wait for pasted output and then recommend exactly one next command.

## Output Interpretation
Summarize `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `latest_status`, `latest_selection_headline`, `latest_selection_reason`, `latest_checkpoint_id`, `latest_checkpoint_scope`, `latest_checkpoint_restore_command`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `explanation`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, `governance_next_action`, and the CLI-reported `next_command`. Preserve `latest_trace_ref` when present so `/boundline-inspect` can reuse it, and keep any `effective_routing`, `assistant_bindings`, `runtime_capabilities`, or `slot_effort_policies` values surfaced inside `route_config_projection`. When the context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. When the context or governance fields are Canon-grounded, preserve governed artifact refs, credibility, and stale-memory wording exactly and treat non-credible governed memory as a real stop condition. When checkpoint fields appear, preserve them exactly and prefer the reported restore command over generic restart advice.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if it points to inspect or `continuity_authority: compatibility_trace` is present, route to `/boundline-inspect`. Route to `/boundline-start` only when the CLI reports no active session and no compatibility follow-up.
Allowed follow-up commands: `/boundline-step`, `/boundline-inspect`, `/boundline-status`, `/boundline-plan`, `/boundline-start`.