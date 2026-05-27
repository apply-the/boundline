---
description: "Summarize the latest known status of a Boundline workflow"
---

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

## Shell-Enabled Path
Run `boundline status --workspace <workspace> --json` exactly once.

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

## Next-Step Routing
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): the CLI-reported `next_command` (typically `/boundline-step` or `/boundline-plan`).
**Secondary** (shown only when `continuity_authority: compatibility_trace`, `compatibility_follow_up: inspect_only`, or detailed review is needed): `/boundline-inspect` — inspect in detail.

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present — it overrides the primary.
Route to `/boundline-goal` only when the CLI reports no active session and no compatibility follow-up.

Allowed follow-up commands: `/boundline-next`, `/boundline-inspect`, `/boundline-step`, `/boundline-plan`, `/boundline-goal`, `/boundline-status`.