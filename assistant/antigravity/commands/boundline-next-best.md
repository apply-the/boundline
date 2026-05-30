# Command: /boundline-next-best

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Recommend the next best bounded action from authoritative session state without inventing a new workflow.
Prefer the CLI-reported next command over assistant-authored guesswork.

## Required Context
- `workspace_ref`
- Preserve any confirmed `latest_trace_ref` when it already anchors the current session discussion

## Shell-Enabled Path
If `workspace_ref` is known, run `boundline status --workspace <workspace> --json` exactly once.
If the assistant is already anchored in the workspace, run `boundline status --json` exactly once.
If status reports inspect-only continuity, route to `/boundline-inspect` instead of inventing a new action.

## Chat-Only Path
Ask only for the missing `workspace_ref`, then provide this exact copyable command:

`boundline status --workspace <workspace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Summarize `latest_status`, `execution_condition`, `continuity_authority`, `current_step_id`, `latest_validation_status`, `latest_trace_ref`, `governance_next_action`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and `next_command`. Preserve `context_summary`, `context_credibility`, `authored_input_summary`, and Canon-governed blockage or approval wording exactly. If the next safe action is blocked on missing Canon or setup evidence, say that explicitly and keep the recommended action bounded.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if status reports no active session, route to `/boundline-goal`.
Allowed follow-up commands: `/boundline-status`, `/boundline-inspect`, `/boundline-why`, `/boundline-risk`, `/boundline-evidence`, `/boundline-goal`.
