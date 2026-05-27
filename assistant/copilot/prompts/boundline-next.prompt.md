---
description: "Recommend the next bounded Boundline action"
---

# Command: /boundline-next

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Ask Boundline for the next recommended session command.

Compatibility follow-up means the user previously chose `boundline run --compatibility ...`; plain direct `run --goal` is native-first in `0.43.0`.

## Required Context
- `workspace_ref`
- Latest known outcome when already available

## Shell-Enabled Path
If the workspace is known, run `boundline next --workspace <workspace> --json` exactly once and use the reported recommendation.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`boundline next --workspace <workspace> --json`

Wait for pasted output and then recommend exactly one next command.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Summarize `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `latest_status`, `latest_selection_headline`, `latest_selection_reason`, `latest_checkpoint_id`, `latest_checkpoint_scope`, `latest_checkpoint_restore_command`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `explanation`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, `governance_next_action`, and the CLI-reported `next_command`. Preserve `latest_trace_ref` when present so `/boundline-inspect` can reuse it, and keep any `effective_routing`, `assistant_bindings`, `runtime_capabilities`, or `slot_effort_policies` values surfaced inside `route_config_projection`. When the context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. When the context or governance fields are Canon-grounded, preserve governed artifact refs, credibility, and stale-memory wording exactly and treat non-credible governed memory as a real stop condition. When checkpoint fields appear, preserve them exactly and prefer the reported restore command over generic restart advice.

## Next-Step Routing
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): render the CLI-reported `next_command` as the matching clickable `command:github.copilot.chat.execute` link. For common continuations, use:
[▶ Run /boundline-step](command:github.copilot.chat.execute?%5B%22%2Fboundline-step%22%5D)
[▶ Run /boundline-inspect](command:github.copilot.chat.execute?%5B%22%2Fboundline-inspect%22%5D)
[▶ Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)

**Secondary** (shown only when context needs review before acting, or `continuity_authority: compatibility_trace` is present): render this clickable link:
[▶ Run /boundline-status](command:github.copilot.chat.execute?%5B%22%2Fboundline-status%22%5D)

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`. Render whichever assistant-safe route wins using `command:github.copilot.chat.execute`.
Route to `/boundline-goal` only when the CLI reports no active session and no compatibility follow-up.

Allowed follow-up commands: `/boundline-step`, `/boundline-inspect`, `/boundline-status`, `/boundline-plan`, `/boundline-goal`.
