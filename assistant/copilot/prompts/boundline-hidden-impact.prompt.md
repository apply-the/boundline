---
description: "Inspect indirect impact from the active Boundline runtime state"
---

# Command: /boundline-hidden-impact

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Surface likely indirect effects from the active Boundline runtime state.
Use structured retrieval by default and disclose when higher-order impact inference falls back.

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
Summarize `hidden_impact_summary` first, then preserve any `hidden_impact_affected_domains`, `hidden_impact_affected_systems`, `hidden_impact_missing_tests`, `hidden_impact_missing_evidence`, `hidden_impact_required_reviewers`, `hidden_impact_fallback_disclosure`, `challenge_required_review`, and `next_command` lines verbatim. Keep governance boundaries visible instead of collapsing them into generic impact advice.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If inspect reports a session error, route to `/boundline-goal`.
Allowed follow-up commands: `/boundline-challenge`, `/boundline-explain-plan`, `/boundline-evidence`, `/boundline-inspect`, `/boundline-status`, `/boundline-goal`.
