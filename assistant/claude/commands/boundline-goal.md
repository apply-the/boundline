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

## User Input
Use the text after `/boundline-goal` as the goal source when present. If the user supplies only brief paths, treat the referenced Markdown as the goal source. Do not ask the user to repeat non-empty input.

## Pre-Execution Checks
Confirm only that the workspace is known and that at least one goal source is available. Do not read `.specify/extensions.yml` or run generic pre/post hooks; Boundline uses runtime `phase_request`, `assistant_resume_command`, and `assistant_next_command` for handoff.

## Execution Flow
1. Run `boundline orchestrate` once with the known workspace and goal or brief sources.
2. Stop on the first structured `phase_request`.
3. If no `phase_request` is emitted, summarize the captured goal, quality state, and next assistant-safe route.

## Goal Quality Validation
Boundline runtime owns validation. Surface `goal_quality_state`, `goal_quality_findings`, and `goal_quality_assumptions` when present, translating them into plain language. A blocked quality state means planning must wait for the emitted question.

The quality rubric checks for a bounded outcome and scope boundary, actors/actions/data or affected artifact, intended outcome, validation target, measurable success criteria, assumptions/defaults, and security/privacy/auth clarification only when materially relevant. Clarifications are prioritized as scope > security/privacy > user experience > technical details. Maximum 3 quality clarification questions may be reported by the runtime, but host interaction still asks exactly one `phase_request.question` at a time.

## Quick Guidelines
Focus on what the user needs and why. Avoid implementation details unless the user supplied them as constraints or validation evidence. Think like a tester: vague requirements should become testable outcomes or a runtime clarification.

## Reasonable Defaults
Do not ask about low-impact omissions. Accept and surface runtime assumptions such as no new auth/privacy boundary, no new persistence boundary, and scope limited to the stated goal and supplied briefs when those are reasonable.

## Success Criteria Guidelines
Prefer measurable, user- or business-facing outcomes. Good examples: "users complete checkout in under 3 minutes", "95% of searches return results in under 1 second", or "task completion improves by 40%". Avoid framework, database, cache, or internal implementation metrics unless the user provided them as validation evidence.

## Done When
- The goal source has been passed to `boundline orchestrate`.
- Any `phase_request.question` has been asked exactly as emitted.
- `goal_quality_state`, `goal_quality_findings`, and `goal_quality_assumptions` are surfaced when present.
- The next route is taken from `assistant_resume_command`, `assistant_next_command`, or the CLI-reported route.

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
Claude Code operates as a terminal and conversational surface. No native interactive question API is available. Phase requests are rendered using the universal fallback only.

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
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
For Claude surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:plan`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on a structured goal `phase_request`, and surface the recorded goal or `authored_input_summary`, `authored_input_sources`, the latest status, and exactly one valid follow-up route. When the stream emits a structured goal `phase_request`, explain `phase_request.reason` in one concise line, ask exactly `phase_request.question`, preserve `phase_request.request_id`, `phase_request.expected_answer`, and the raw `resume_command` including any `--answer "<answer>"` placeholder for shell continuation only. Treat legacy clarification fields as compatibility fallback only when no structured `phase_request` is present.

## Next-Step Routing
Surface exactly one host-native action link using `/boundline:*` command ids.
If the CLI emits a structured goal `phase_request` or reports legacy clarification fields, route only to `/boundline:goal`.
Otherwise prefer `assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`, which is typically `/boundline:plan`.
Allowed follow-up commands: `/boundline:plan`, `/boundline:goal`.
