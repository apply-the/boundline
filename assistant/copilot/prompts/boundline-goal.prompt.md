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

## User Input
Use the text after `/boundline-goal` as the goal source when present. If the user supplies only brief paths, treat the referenced Markdown as the goal source. Do not ask the user to repeat non-empty input.

## Pre-Execution Checks
Confirm only that the workspace is known and that at least one goal source is available. Do not read `.specify/extensions.yml` or run generic pre/post hooks; Boundline uses runtime `phase_request`, `assistant_resume_command`, and `assistant_next_command` for handoff.

## Execution Flow
1. Derive a concise 2-4 word kebab-case slug from the goal when opening a new session.
2. Run `boundline orchestrate` once with the known workspace and goal or brief sources.
3. Stop on the first structured `phase_request`.
4. If no `phase_request` is emitted, summarize the captured goal, quality state, and next assistant-safe route.

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
If the workspace and at least one goal source are known, prefer the orchestrator command exactly once.

Before running `orchestrate`, derive a 2-4 word kebab-case slug from the goal (action-noun format, e.g. `rust-user-service`, `fix-payment-timeout`, `oauth2-api-gateway`; preserve technical acronyms). Pass it as `--slug <derived-slug>` in calls that open a new session.

`boundline orchestrate --workspace <workspace> --goal "<goal>" --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`

Ask only for the missing workspace or missing goal source. Reuse confirmed brief paths instead of asking for them again. The raw `boundline goal` command remains the non-interactive capture primitive for direct shell use; assistant-host interactive flows should stay on `orchestrate`.

## Chat-Only Path
If shell execution is unavailable, ask only for the missing workspace or goal source, then provide one exact copyable orchestrator command.

Derive a 2-4 word kebab-case slug from the goal (action-noun format, e.g. `rust-user-service`, `fix-payment-timeout`, `oauth2-api-gateway`; preserve technical acronyms) and include it as `--slug <derived-slug>`.

`boundline orchestrate --workspace <workspace> --goal "<goal>" --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on a structured goal `phase_request`, and surface the recorded goal or `authored_input_summary`, `authored_input_sources`, the latest status, and exactly one valid follow-up route. When the stream emits a structured goal `phase_request`, explain `phase_request.reason` in one concise line, ask exactly `phase_request.question`, preserve `phase_request.request_id`, `phase_request.expected_answer`, any `assistant_resume_command`, and the raw `resume_command` including any `--answer "<answer>"` placeholder for shell continuation only. Treat legacy clarification fields (`clarification_prompt`, `clarification_missing_fields`, or `clarification_questions`) as compatibility fallback only when no structured `phase_request` is present.
When an NDJSON event also carries `audit`, use that projection as the authoritative explanation of who acted and what happened: preserve `audit.event`, `audit.algorithm`, `audit.actor.display_name` or `audit.actor.id`, `audit.outcome.status`, and `audit.message`. If `audit.actor.participant_routes` or `audit.actor.mixed_routes` is present, keep that multi-route attribution explicit instead of flattening it to one route.

**Critical rule — inline question required:** When the stream contains a `phase_request` with a non-null `question` field, you MUST ask that exact question as your direct reply. Do NOT render an action link or button in place of the question — asking the question inline IS the next action. Only AFTER the user answers should you construct the next shell run substituting the answer into `resume_command`'s `--answer "<answer>"` placeholder.

## Next-Step Routing (MANDATORY FORMAT)
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): render as a clickable link:
[▶ Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)

**Secondary** (shown only when a `phase_request.question` is active or clarification is still pending): render as:
[▶ Run /boundline-goal](command:github.copilot.chat.execute?%5B%22%2Fboundline-goal%22%5D)

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`. Render whichever assistant-safe route wins using the same clickable format: `[▶ Run /command-name](command:github.copilot.chat.execute?%5B%22%2Fcommand-name%22%5D)`.

Allowed follow-up commands: `/boundline-plan`, `/boundline-goal`.

## Agent Mode Override
This prompt runs as a VS Code Copilot chat participant. In Agent Mode, where tool calls are available, an interactive rendering override applies to `phase_request` events.

When tools are available and the stream contains a `phase_request`:
1. Invoke `vscode_askQuestions` with the question text from `phase_request.question`.
2. Map `expected_answer.type` to the question format:
   - `free_text` → free-text input (no fixed options)
   - `suggested_choice` → selectable suggestions from `expected_answer.options` plus a custom/free-text fallback; selecting a suggestion sends that option's `value`
   - `confirmation` → options `["Yes", "No"]`
   - `single_choice` → options from `expected_answer.options`
   - `multi_choice` → multi-select from `expected_answer.options`; if multi-select is unavailable, present as a numbered list with free-text input
3. Do NOT also print the question as plain prose. Choose one rendering path — interactive tool or inline text — but not both.
4. After the user answers, substitute the answer into `resume_command`'s `--answer "<user_answer>"` placeholder and run or surface the resulting command.

When tools are unavailable, fall back to the inline question rule in `## Output Interpretation`.
