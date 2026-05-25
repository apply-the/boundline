---
description: "Explain why the current Boundline state looks the way it does"
---

# Command: /boundline-why

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Explain why the current plan, block, or next step is happening from authoritative Boundline runtime state.
Keep Boundline-owned evidence separate from Canon-governed signals and call out any missing Canon input explicitly.

## Required Context
- `workspace_ref` or `trace_ref`
- Preserve any confirmed `latest_trace_ref` from prior turns

## Shell-Enabled Path
If `trace_ref` is known, run `cargo run --bin boundline -- inspect --trace <trace> --json`.
Otherwise, if `workspace_ref` is known, run `cargo run --bin boundline -- inspect --workspace <workspace> --json`.
If the assistant is already anchored in the workspace, run `cargo run --bin boundline -- inspect --json` exactly once.

## Chat-Only Path
Ask only for the missing `workspace_ref` or `trace_ref`, then provide one exact copyable command:

`cargo run --bin boundline -- inspect --workspace <workspace> --json`

or

`cargo run --bin boundline -- inspect --trace <trace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Summarize the current `goal`, `routing_summary`, `goal_plan_summary`, `negotiation_goal_summary`, `decision_timeline`, `failure_evidence`, `terminal_reason`, `governance_next_action`, `reasoning_profile_id`, `reasoning_selection_reason`, `reasoning_contribution`, `reasoning_fallback_disclosure`, `follow_through_guidance`, `follow_through_evidence_source`, and `next_command` when present. Preserve `authored_input_summary`, `context_summary`, `context_credibility`, `context_provenance`, and any Canon-grounded governance or stale-context wording exactly. When Canon-governed input is absent, stale, incompatible, blocked, or downgraded to a reasoning fallback, say that plainly instead of implying agreement.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If inspect reports a session error, route to `/boundline-goal`.
Allowed follow-up commands: `/boundline-next-best`, `/boundline-risk`, `/boundline-evidence`, `/boundline-inspect`, `/boundline-status`, `/boundline-goal`.
