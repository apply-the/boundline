# Command: /boundline-status

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Summarize the active session state or latest compatibility follow-up for a workspace without re-inspecting a trace by default.

Compatibility follow-up means the user previously chose `boundline run --compatibility ...`; plain direct `run --goal` is native-first in `0.43.0`.

## Required Context
- `workspace_ref`

## Pre-Execution Checks
When workspace or session readiness is uncertain, run `boundline probe --workspace <workspace> --json` for a fast preflight snapshot. If the probe recommends `boundline init` and omits an assistant handoff, stop and surface the host bootstrap CLI path instead of inventing a repo-local handoff. If the probe recommends doctor, redirect to `/boundline-doctor` before relying on `status` to infer session state.

## Shell-Enabled Path
Run `boundline status --workspace <workspace> --json` exactly once.
If the user asks for a richer state view than `status`, route to `/boundline-inspect` instead of switching to the removed terminal UI surface.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`boundline status --workspace <workspace> --json`

Then wait for pasted output.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Reply as a compact operator brief by default: preserve `goal` when present, `authored_input_summary` or `authored_input_sources`, `routing`, `execution_condition`, a concise bounded summary, key artifacts, `latest_status`, and the CLI-reported `next_command`. Only surface raw `route_config_projection`, `context_provenance`, or guidance source dumps when the user explicitly asks for deeper detail.

Summarize `latest_status`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `compatibility_follow_up_command`, `execution_path`, `flow_state`, `latest_decision_status`, `latest_decision_target`, `latest_selection_headline`, `latest_selection_reason`, `current_step_id`, `latest_changed_files`, `latest_validation_status`, `latest_checkpoint_id`, `latest_checkpoint_scope`, `latest_checkpoint_restore_command`, any `latest_trace_ref`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, governance mode, run-ref, packet provenance, `governance_next_action`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, and `follow_through_stop_reason` when present, plus the CLI-reported `next_command`. Preserve `effective_routing`, `assistant_bindings`, `runtime_capabilities`, and `slot_effort_policies` when they appear inside `route_config_projection`. When the context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. When the context or governance fields are Canon-grounded, preserve governed artifact refs, credibility, and stale-memory wording exactly and treat non-credible governed memory as a real stop condition. When checkpoint fields appear, preserve them exactly and prefer the reported restore command over generic restart advice.
When status reports `planning_analysis_state`, `planning_analysis_findings`, or `planning_analysis_coverage`, preserve them exactly. Treat `planning_analysis_state: blocked` as a real execution stop and route back to `/boundline:plan` instead of `/boundline:run`.
When status also reports `repository_map_state`, `snapshot_cache_state`, `context_pack_entries`, `omission_findings`, or `patch_safe_edit_attempts`, preserve them exactly. Treat blocking omission findings, unsafe oversized-read refusal, stale tracked cache, or patch-safe edit drift as real planning stops rather than advisory context notes.
When status also reports `capability_provider_status`, `capability_provider_id`, `capability_provider_activation_state`, `capability_provider_capability_ids`, `capability_provider_setup_requirements`, or `capability_provider_summary`, preserve them exactly. Treat incomplete setup, blocked readiness, or a degraded provider summary as a real bounded stop rather than assistant-only retry advice.
When status reports `completion_verification_state`, `completion_claim_kind`, `completion_claim_source`, `completion_blocked_claims`, `completion_evidence_refs`, `completion_verification_finding`, or `completion_verification_required_action`, preserve them exactly. If completion verification is not `ready`, do not describe the session as successful. Call out the blocked claim, the proving command or rerun action, and that Boundline owns proof execution while Canon may later consume the emitted evidence refs.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if status reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/boundline-inspect`. Route to `/boundline-goal` only when the CLI reports no active session and no compatibility follow-up.
Allowed follow-up commands: `/boundline-next`, `/boundline-inspect`, `/boundline-step`, `/boundline-plan`, `/boundline-goal`, `/boundline-status`.
