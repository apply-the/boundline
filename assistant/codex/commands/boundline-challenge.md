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
If `trace_ref` is known, run `boundline inspect --trace <trace> --json`.
Otherwise, if `workspace_ref` is known, run `boundline inspect --workspace <workspace> --json`.
If the assistant is already anchored in the workspace, run `boundline inspect --json` exactly once.

## Chat-Only Path
Ask only for the missing `workspace_ref` or `trace_ref`, then provide one exact copyable command:

`boundline inspect --workspace <workspace> --json`

or

`boundline inspect --trace <trace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Summarize `challenge_strongest_objection`, `challenge_weakest_assumption`, `challenge_missing_evidence`, `challenge_failure_mode`, `challenge_required_review`, `challenge_council_required`, `reasoning_profile_id`, `reasoning_selection_reason`, `reasoning_contribution`, `reasoning_fallback_disclosure`, `fallback_disclosure`, and `next_command`. Do not soften governance wording or imply that this command can replace required Canon review or council decisions.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If `challenge_required_review` or `challenge_council_required` escalates governance, route to `/boundline-govern` or `/boundline-status` instead of inventing a bypass.
Allowed follow-up commands: `/boundline-govern`, `/boundline-hidden-impact`, `/boundline-explain-plan`, `/boundline-inspect`, `/boundline-status`, `/boundline-goal`.
