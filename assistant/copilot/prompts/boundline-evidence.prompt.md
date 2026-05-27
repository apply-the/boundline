---
description: "Show the evidence currently supporting the active Boundline answer"
---

# Command: /boundline-evidence

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Show which runtime, workspace, and Canon-governed signals currently support the active Boundline answer.
Do not collapse Boundline evidence, Canon evidence, and missing evidence into one undifferentiated summary.

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
Summarize `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `context_summary`, `context_primary_inputs`, `context_provenance`, `decision_timeline`, `failure_evidence`, `adaptive_evidence`, `governance_timeline`, `governance_approval_provenance`, `review_timeline`, and `next_command`. Preserve Canon-grounded provenance, contract-line, packet, approval, readiness, security, audit, or promotion wording exactly when present. If Canon evidence is missing, stale, incompatible, or blocked, list that as a real evidence gap.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if more state is needed, route to `/boundline-inspect`.
Allowed follow-up commands: `/boundline-why`, `/boundline-risk`, `/boundline-next-best`, `/boundline-inspect`, `/boundline-status`, `/boundline-goal`.
