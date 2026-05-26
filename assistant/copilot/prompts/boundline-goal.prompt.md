---
description: "Define or refine the active Boundline session goal"
---

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
If the workspace and at least one goal source are known, prefer the orchestrator command exactly once.

Before running `orchestrate`, derive a 2-4 word kebab-case slug from the goal (action-noun format, e.g. `rust-user-service`, `fix-payment-timeout`, `oauth2-api-gateway`; preserve technical acronyms). Pass it as `--slug <derived-slug>` in calls that open a new session.

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`

Ask only for the missing workspace or missing goal source. Reuse confirmed brief paths instead of asking for them again. The raw `boundline goal` command remains the non-interactive capture primitive for direct shell use; assistant-host interactive flows should stay on `orchestrate`.

## Chat-Only Path
If shell execution is unavailable, ask only for the missing workspace or goal source, then provide one exact copyable orchestrator command.

Derive a 2-4 word kebab-case slug from the goal (action-noun format, e.g. `rust-user-service`, `fix-payment-timeout`, `oauth2-api-gateway`; preserve technical acronyms) and include it as `--slug <derived-slug>`.

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on a structured goal `phase_request`, and surface the recorded goal or `authored_input_summary`, `authored_input_sources`, the latest status, and exactly one valid follow-up route. When the stream emits a structured goal `phase_request`, explain `phase_request.reason` in one concise line, ask exactly `phase_request.question`, preserve `phase_request.request_id`, `phase_request.expected_answer`, any `assistant_resume_command`, and the raw `resume_command` including any `--answer "<answer>"` placeholder for shell continuation only. Treat legacy clarification fields (`clarification_prompt`, `clarification_missing_fields`, or `clarification_questions`) as compatibility fallback only when no structured `phase_request` is present.
When an NDJSON event also carries `audit`, use that projection as the authoritative explanation of who acted and what happened: preserve `audit.event`, `audit.algorithm`, `audit.actor.display_name` or `audit.actor.id`, `audit.outcome.status`, and `audit.message`. If `audit.actor.participant_routes` or `audit.actor.mixed_routes` is present, keep that multi-route attribution explicit instead of flattening it to one route.

**Critical rule — inline question required:** When the stream contains a `phase_request` with a non-null `question` field, you MUST ask that exact question as your direct reply. Do NOT render an action link or button in place of the question — asking the question inline IS the next action. Only AFTER the user answers should you construct the next shell run substituting the answer into `resume_command`'s `--answer "<answer>"` placeholder.

## Next-Step Routing (MANDATORY FORMAT)
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): render as a clickable link:
[▶ Run /boundline-plan](command:copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)

**Secondary** (shown only when a `phase_request.question` is active or clarification is still pending): render as:
[▶ Run /boundline-goal](command:copilot.chat.execute?%5B%22%2Fboundline-goal%22%5D)

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present — it overrides the primary. Render it using the same clickable format: `[▶ Run /command-name](command:copilot.chat.execute?%5B%22%2Fcommand-name%22%5D)`.

Allowed follow-up commands: `/boundline-plan`, `/boundline-goal`.

## Agent Mode Override
This prompt runs as a VS Code Copilot chat participant. In Agent Mode, where tool calls are available, an interactive rendering override applies to `phase_request` events.

When tools are available and the stream contains a `phase_request`:
1. Invoke `vscode_askQuestions` with the question text from `phase_request.question`.
2. Map `expected_answer.type` to the question format:
   - `free_text` → free-text input (no fixed options)
   - `confirmation` → options `["Yes", "No"]`
   - `single_choice` → options from `expected_answer.options`
   - `multi_choice` → multi-select from `expected_answer.options`; if multi-select is unavailable, present as a numbered list with free-text input
3. Do NOT also print the question as plain prose. Choose one rendering path — interactive tool or inline text — but not both.
4. After the user answers, substitute the answer into `resume_command`'s `--answer "<user_answer>"` placeholder and run or surface the resulting command.

When tools are unavailable, fall back to the inline question rule in `## Output Interpretation`.
