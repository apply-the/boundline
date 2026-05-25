---
description: "Challenge the current Boundline plan without hiding governance"
---

# Command: /boundline-challenge

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Challenge the current bounded plan without replacing the formal governance path.
Keep objections, weak assumptions, missing evidence, and required review visible.

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
Summarize `challenge_strongest_objection`, `challenge_weakest_assumption`, `challenge_missing_evidence`, `challenge_failure_mode`, `challenge_required_review`, `challenge_council_required`, `reasoning_profile_id`, `reasoning_selection_reason`, `reasoning_contribution`, `reasoning_fallback_disclosure`, `fallback_disclosure`, and `next_command`. Do not soften governance wording or imply that this command can replace required Canon review or council decisions.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If `challenge_required_review` or `challenge_council_required` escalates governance, route to `/boundline-govern` or `/boundline-status` instead of inventing a bypass.
Allowed follow-up commands: `/boundline-govern`, `/boundline-hidden-impact`, `/boundline-explain-plan`, `/boundline-inspect`, `/boundline-status`, `/boundline-goal`.
