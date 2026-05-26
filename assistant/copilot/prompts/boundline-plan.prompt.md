---
description: "Plan a Boundline workflow"
---

# Command: /boundline-plan

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Define or refine the active session goal from the user's goal text and/or supporting brief, then plan the active session into a bounded proposal.

If the active session already has a captured goal and the user is only supplying a planning brief, treat that file as planning input, not as a new goal capture.

## Required Context
- `workspace_ref`
- At least one goal source: bounded goal text and/or workspace-relative Markdown brief path(s)

## Shell-Enabled Path
If the workspace and at least one goal source are known, prefer the orchestrator command exactly once:

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- plan --workspace <workspace> --input <path> --json`

Ask only for missing workspace or missing goal source. Reuse confirmed brief paths instead of asking for them again.

Canonical backend mapping reference:

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`cargo run --bin boundline -- plan --workspace <workspace> --input <path> --json`

For Copilot execution, append `--assistant-host copilot` to the backend mapping.
Treat those backend mappings as execution detail. When the resulting continuation is the run surface, do not present `boundline orchestrate --assistant-host ...` as a parallel user-facing next step.

## Chat-Only Path
Ask only for the missing workspace or goal source, then provide the matching exact copyable orchestrator command:

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --assistant-host copilot --until phase-request --json-stream`
`cargo run --bin boundline -- plan --workspace <workspace> --input <path> --json`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on `phase_request`, and surface `goal`, `authored_input_summary` or `authored_input_sources`, `execution_condition`, planning summary, key artifacts, `latest_status`, and exactly one emitted assistant-safe route: prefer `assistant_resume_command` when present, otherwise `assistant_next_command`. Keep the raw `resume_command` only as the hidden shell continuation when needed, and preserve the latest `next_command` only when no assistant-safe route is present. When the selected assistant-safe route is `/boundline-run`, present only that route to the user and keep any equivalent `boundline run` or `boundline orchestrate --assistant-host ...` mapping out of the visible next-step text unless the user explicitly asks for shell or backend details. When the planning handoff is stage-specific, treat the emitted `phase_request` as the next single planning stage to complete and preserve the exact raw `resume_command`, including any `--planning-stage-complete <stage_key>` marker, for shell execution only. Only surface raw `context_provenance`, `route_config_projection`, or guidance source dumps when the user explicitly asks for deeper detail.
When an event carries `audit`, use it as the primary attribution block for the planning summary: prefer `audit.event`, `audit.algorithm`, `audit.actor`, `audit.outcome`, and `audit.message` over legacy flat actor fields. Preserve `audit.actor.participant_routes` and `audit.actor.mixed_routes` exactly when planning governance or review used multiple reviewer routes.

Summarize the recorded goal or `authored_input_summary`, `authored_input_sources`, any requested governance intent, the resulting plan state, any proposed, confirmed, skipped, or absent `flow_state`, any `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, the emitted planning `phase_request` with its `stage_key`, `instruction`, `artifact.artifact_ref`, the single assistant-safe route chosen for the user, the exact raw `resume_command` when present, and the latest `next_command` only when no assistant-safe route is available. When planning also reports `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, or `context_staleness_reason`, preserve those fields exactly. When those context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. If that context is Canon-grounded, also preserve governed artifact refs, packet refs, and stale-memory wording exactly and treat non-credible context as a real stop condition.

If plan output contains clarification fields or a gate (`phase_request`,
approval-required, or blocked state), switch to interactive mode: ask the
minimum follow-up question needed, wait for user input, present exactly one
chat-facing route (prefer `assistant_resume_command`, otherwise `assistant_next_command`),
and use the raw `resume_command` or CLI `next_command` only for the actual shell continuation. Do not present a chat-facing route and its backend `boundline orchestrate --assistant-host ...` equivalent as alternatives in the same sentence.

## Planning Stage Authoring (incomplete Canon packets)

When a planning `phase_request` has `kind: "clarification"` and its `instruction` directs you to author placeholder sections:

0. **Show context gaps first.** If the `instruction` contains a numbered list of context gaps (lines like "1. validation_target requires..."), present them as a brief summary to the user BEFORE showing the choice options. Explain each gap in plain language so the user understands what information is missing and why.
1. Open the referenced artifact (`artifact.artifact_ref` or the path named in the `instruction`).
2. Identify all placeholder markers: lines containing only `TODO`, `TBD`, `N/A`, `[TODO]`, `[TBD]`, `missing-authored-body`, or headings with no substantive body beneath them.
3. Replace each placeholder section with substantive authored content derived from:
   - The captured goal and `authored_input_summary`
   - Project context already gathered (domain, stack, constraints)
   - Any reference file or folder the user provides in their answer
4. When the user's answer is a file or folder path (starts with `/`, `./`, `~`, or contains path separators), read that path's content and use it as the primary source material for filling the placeholder sections.
5. When the user selects **"fill from best practices"**, fill placeholder sections using:
   - Established conventions for the detected stack (e.g., `cargo test` for Rust, REST for HTTP APIs, repository pattern for persistence)
   - Standard architectural patterns appropriate for the domain (e.g., hexagonal architecture, CQRS, event sourcing)
   - If web search or browsing tools are available, research current best practices for the specific technology and domain
   - Do NOT ask for confirmation before filling; apply best practices directly, then resume orchestration.
6. Once all placeholder sections are filled with real content, run the `resume_command` (with `--planning-stage-complete <stage_key>`) to advance orchestration.

When the `phase_request` has `kind: "review"` and `expected_answer.type: "confirmation"`, the packet is already substantively authored; present the confirmation gate to the user and wait for their decision.

## Next-Step Routing (MANDATORY FORMAT)
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): render as a clickable link:
[▶ Run /boundline-step](command:copilot.chat.execute?%5B%22%2Fboundline-step%22%5D) or [▶ Run /boundline-run](command:copilot.chat.execute?%5B%22%2Fboundline-run%22%5D)

**Secondary** (shown only when governance is blocked, context is non-credible, or brief needs authoring): render as:
[▶ Run /boundline-plan](command:copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present — it overrides the primary. Render it using the same clickable format.

Allowed follow-up commands: `/boundline-step`, `/boundline-run`, `/boundline-plan`, `/boundline-goal`.
