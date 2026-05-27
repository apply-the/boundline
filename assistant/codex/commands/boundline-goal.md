# Command: /boundline-goal

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Capture or refine the active session goal through the orchestrator so runtime-owned clarification gates can stop the flow before planning.

## Required Context
- `workspace_ref`
- At least one goal source: bounded goal text and/or workspace-relative Markdown brief path(s)

## Shell-Enabled Path
If the workspace and at least one goal source are known, prefer the orchestrator command exactly once:

`boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`

Ask only for the missing workspace or missing goal source. Reuse confirmed brief paths instead of asking for them again. The raw `boundline goal` command remains the non-interactive capture primitive for direct shell use; assistant-host interactive flows should stay on `orchestrate`.

## Chat-Only Path
If shell execution is unavailable, ask only for the missing workspace or goal source, then provide one exact copyable orchestrator command:

`boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`

Wait for pasted output before continuing.

## Host Capabilities
Codex CLI operates as a terminal and conversational surface. No native interactive question API is available. Phase requests are rendered using the universal fallback only.

## Phase Request Rendering
When the stream contains a `phase_request` event, stop and display this template exactly:

---
Boundline needs one answer before it can continue.

Question: `<phase_request.question>`
Reason:   `<phase_request.reason>`
Options:  `<list options when expected_answer.type is suggested_choice, confirmation, single_choice, or multi_choice; for suggested_choice, state that custom text is also allowed; omit for free_text>`
Resume:   `<resume_command with --answer "<your_answer>" substituted>`

---

Hard-stop rules:
- Treat `phase_request` as a full stop. Do not continue past it without collecting an answer.
- Show exactly one question at a time. Do not batch multiple clarification questions.
- For `suggested_choice`, list `expected_answer.options` as suggested answers and accept either an option value or custom text.
- After the user answers, run or suggest the resume command with the user's answer substituted in.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
For Codex surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:plan`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on a structured goal `phase_request`, and surface the recorded goal or `authored_input_summary`, `authored_input_sources`, the latest status, and exactly one valid follow-up route. When the stream emits a structured goal `phase_request`, explain `phase_request.reason` in one concise line, ask exactly `phase_request.question`, preserve `phase_request.request_id`, `phase_request.expected_answer`, and the raw `resume_command` including any `--answer "<answer>"` placeholder for shell continuation only. Treat legacy clarification fields as compatibility fallback only when no structured `phase_request` is present.

## Next-Step Routing
Surface exactly one host-native action link using `/boundline:*` command ids.
If the CLI emits a structured goal `phase_request` or reports legacy clarification fields, route only to `/boundline:goal`.
Otherwise prefer `assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`, which is typically `/boundline:plan`.
Allowed follow-up commands: `/boundline:plan`, `/boundline:goal`.
