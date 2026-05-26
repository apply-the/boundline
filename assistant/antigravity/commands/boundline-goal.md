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

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`

Ask only for the missing workspace or missing goal source. Reuse confirmed brief paths instead of asking for them again. The raw `boundline goal` command remains the non-interactive capture primitive for direct shell use; assistant-host interactive flows should stay on `orchestrate`.

## Chat-Only Path
If shell execution is unavailable, ask only for the missing workspace or goal source, then provide one exact copyable orchestrator command:

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`

Wait for pasted output before continuing.

## Host Capabilities
Antigravity: detect native question or form API availability at runtime.
- If a native API is available: map `phase_request` to it using the answer-type table below.
- If no native API is detected: use the universal fallback template below.

The plain-text fallback is mandatory; native enhancement is optional.

Answer-type mapping:
- `free_text` → open text input
- `confirmation` → yes/no choice (options: `["Yes", "No"]`)
- `single_choice` → single selection from `expected_answer.options`
- `multi_choice` → multi-selection from `expected_answer.options` (or numbered list if unavailable)

## Phase Request Rendering
When the stream contains a `phase_request` event, stop and render using the detected surface. Fall back to this template when no native API is available:

---
Boundline needs one answer before it can continue.

Question: `<phase_request.question>`
Reason:   `<phase_request.reason>`
Options:  `<list options when expected_answer.type is confirmation, single_choice, or multi_choice; omit for free_text>`
Resume:   `<resume_command with --answer "<your_answer>" substituted>`

---

Hard-stop rules:
- Treat `phase_request` as a full stop. Do not continue past it without collecting an answer.
- Show exactly one question at a time. Do not batch multiple clarification questions.
- After the user answers, run or suggest the resume command with the user's answer substituted in.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
For Antigravity surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:plan`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on a structured goal `phase_request`, and surface the recorded goal or `authored_input_summary`, `authored_input_sources`, the latest status, and exactly one valid follow-up route. When the stream emits a structured goal `phase_request`, explain `phase_request.reason` in one concise line, ask exactly `phase_request.question`, preserve `phase_request.request_id`, `phase_request.expected_answer`, and the raw `resume_command` including any `--answer "<answer>"` placeholder for shell continuation only. Treat legacy clarification fields as compatibility fallback only when no structured `phase_request` is present.

## Next-Step Routing
Surface exactly one action link.
If the CLI emits a structured goal `phase_request` or reports legacy clarification fields, route only to `/boundline:goal`.
Otherwise follow the CLI-reported `next_command`, which is typically `/boundline:plan`.
Allowed follow-up commands: `/boundline:plan`, `/boundline:goal`.
