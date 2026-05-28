---
description: "Plan a Boundline workflow"
---

# Command: /boundline-plan

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Plan the active session into a bounded proposal from an already captured goal.

`/boundline-plan` must not capture or refine the goal. If the user is introducing a new goal, changing the current goal, or supplying brief files that should become goal evidence, route to `/boundline-goal` instead.

Do not proceed from chat-only assumptions when goal quality or plan quality is blocked. If status or orchestrate reports `goal_quality_state: clarification_required`, preserve `goal_quality_findings` and route through the emitted goal `phase_request` or `/boundline-goal` instead of inventing planning inputs. If planning reports `plan_quality_state: clarification_required`, preserve `plan_quality_findings` and `plan_quality_assumptions`, then follow the emitted `phase_request` or assistant-safe route before moving to execution.

## User Input

```text
$ARGUMENTS
```

Consider the user input before proceeding. Treat it as planning guidance only when it refines how to plan the already captured goal; if it changes the goal, route back to `/boundline-goal`.

## Pre-Execution Checks
- Confirm the workspace is known and an active session already has a captured goal.
- Check status/orchestrate output for `goal_quality_state` and do not plan from chat-only assumptions while goal quality is blocked.
- Do not read `.specify/extensions.yml` or run Speckit-style hooks for this command; Boundline uses runtime `phase_request`, `assistant_resume_command`, and `assistant_next_command` handoffs.

## Execution Flow
1. Run the planning command once with the active workspace and any explicit planning input path.
2. Parse the structured output and preserve the ordered event sequence.
3. Surface `planning_rationale`, `verification_strategy`, `plan_quality_state`, `plan_quality_findings`, and `plan_quality_assumptions` when present.
4. Stop on `phase_request` or a blocked/non-credible planning state; do not advance to run/step until the runtime reports that planning can continue.

## Plan Quality Validation
Planning is ready only when the runtime provides a bounded plan with a rationale for selected targets and a verification strategy. Treat missing rationale, missing verification strategy, non-credible context, or unresolved stage authoring as a real gate. Do not fill those gaps from chat-only guesses unless the runtime explicitly asks for authored content through `phase_request`.

## Backlog Quality Gate
Canon backlog is governed source material, while Boundline validates execution readiness. Preserve `backlog_quality_state`, `backlog_quality_findings`, `backlog_task_count`, `backlog_mvp_scope`, and `backlog_unmapped_items` whenever planning reports them. If `backlog_quality_state` is `blocked` or `clarification_required`, do not route to `/boundline-run`; present the emitted `phase_request` or planning-stage resume command as the only next route.

## Planning Analysis Gate
Planning analysis is a Boundline-owned read-only projection over the ready plan and backlog. Preserve `planning_analysis_state`, `planning_analysis_findings`, and `planning_analysis_coverage` whenever planning reports them. If `planning_analysis_state` is `blocked`, do not route to `/boundline-run`; present `/boundline-plan` or the emitted planning continuation as the only next route.

## Gate Handling
When a planning gate appears, ask the emitted question or present the emitted suggested choices, wait for the user's answer, and resume with the raw `resume_command` or assistant-safe route. Preserve `request_id`, `stage_key`, `expected_answer.type`, `artifact.artifact_ref`, and the exact raw continuation for shell execution.

## Reasonable Defaults
- Preserve runtime-reported planning gates verbatim instead of inventing missing coverage or hidden mappings.
- Treat `planning_analysis_findings` as read-only execution evidence; do not rewrite the plan or backlog in chat unless the runtime asks for new planning input.
- When `planning_analysis_state` is `findings`, summarize the gaps before offering the next route.

## Quick Guidelines
- Focus on WHY the plan is safe to execute and how it will be validated.
- Keep implementation details tied to the runtime plan; do not invent hidden architecture.
- Treat context credibility and stale Canon memory as blocking until the runtime clears them.

## Success Criteria Guidelines
Summarize success criteria as verifiable outcomes. Prefer user/business outcomes and explicit validation commands already supplied by the runtime or user. Avoid replacing missing criteria with vague terms like fast, robust, or scalable.

## Done When
- Planning output has been summarized with plan state, rationale, verification strategy, and plan quality.
- Any `phase_request` has been asked or routed without advancing prematurely.
- Exactly one valid next route is offered from the runtime-reported assistant-safe command.

## Required Context
- `workspace_ref`
- Active session with a captured goal
- Optional workspace-relative Markdown planning input path(s)

## Shell-Enabled Path
If the workspace is known and the active session already has a captured goal, prefer the planning command exactly once:

`boundline plan --workspace <workspace> --json`
`boundline plan --workspace <workspace> --input <path> --json`

Ask only for missing workspace or missing planning input path. Reuse confirmed planning brief paths instead of asking for them again.

If the user needs to create or change the goal, route to `/boundline-goal` and use:

`boundline orchestrate --workspace <workspace> --goal "<goal>" --assistant-host copilot --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --assistant-host copilot --until phase-request --json-stream`

Canonical backend mapping reference:

`boundline plan --workspace <workspace> --json`
`boundline plan --workspace <workspace> --input <path> --json`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`

For Copilot execution, append `--assistant-host copilot` to the goal-capture backend mapping.
Treat those backend mappings as execution detail. When the resulting continuation is the run surface, do not present `boundline orchestrate --assistant-host ...` as a parallel user-facing next step.

## Chat-Only Path
Ask only for the missing workspace or missing planning input path, then provide the matching exact copyable planning command:

`boundline plan --workspace <workspace> --json`
`boundline plan --workspace <workspace> --input <path> --json`

If the active session has no captured goal or the user wants to change it, route to `/boundline-goal` instead and provide the matching exact copyable orchestrator command:

`boundline orchestrate --workspace <workspace> --goal "<goal>" --assistant-host copilot --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --assistant-host copilot --until phase-request --json-stream`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
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

If `expected_answer.type` is `suggested_choice`, present `expected_answer.options` as suggested answers first and keep a custom answer or reference-path path available.
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
Surface exactly one action link matching the actual required next step.

Do NOT always propose `/boundline-run`. If the session is blocked (e.g., by governance, non-credible context, or missing input), propose the command that resolves the block as reported by the CLI (e.g., `/boundline-inspect`, `/boundline-plan`, or `/boundline-goal`).
Only propose `/boundline-run` or `/boundline-step` if the CLI output explicitly permits execution to advance.

Before the action link, include one brief natural-language sentence summarizing why this action is offered.
Prefer an emitted `phase_request.assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`. Render whichever assistant-safe route wins using the format: `[▶ Run /command-name](command:github.copilot.chat.execute?%5B%22%2Fcommand-name%22%5D)`.

Allowed follow-up commands: `/boundline-step`, `/boundline-run`, `/boundline-plan`, `/boundline-goal`, `/boundline-inspect`.
