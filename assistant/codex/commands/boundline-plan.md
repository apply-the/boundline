# Command: /boundline-plan

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Plan the active session into a bounded proposal from an already captured goal.

`/boundline-plan` must not capture or refine the goal. If the user is introducing a new goal, changing the current goal, or supplying brief files that should become goal evidence, route to `/boundline-goal` instead.

Do not proceed from chat-only assumptions when goal quality or plan quality is blocked. If status or orchestrate reports `goal_quality_state: clarification_required`, preserve `goal_quality_findings` and route through the emitted goal `phase_request` or `/boundline:goal` instead of inventing planning inputs. If planning reports `plan_quality_state: clarification_required`, preserve `plan_quality_findings` and `plan_quality_assumptions`, then follow the emitted `phase_request` or assistant-safe route before moving to execution.

## User Input

```text
$ARGUMENTS
```

Consider the user input before proceeding. Treat it as planning guidance only when it refines how to plan the already captured goal; if it changes the goal, route back to `/boundline:goal`.

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
Canon backlog is governed source material, while Boundline validates execution readiness. Preserve `backlog_quality_state`, `backlog_quality_findings`, `backlog_task_count`, `backlog_mvp_scope`, and `backlog_unmapped_items` whenever planning reports them. If `backlog_quality_state` is `blocked` or `clarification_required`, do not route to `/boundline:run`; present the emitted `phase_request` or planning-stage resume command as the only next route.

## Planning Analysis Gate
Planning analysis is a Boundline-owned read-only projection over the ready plan and backlog. Preserve `planning_analysis_state`, `planning_analysis_findings`, and `planning_analysis_coverage` whenever planning reports them. If `planning_analysis_state` is `blocked`, do not route to `/boundline:run`; present `/boundline:plan` or the emitted planning continuation as the only next route.

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

`boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`

Canonical session-native backend mapping reference:

`boundline plan --workspace <workspace> --json`
`boundline plan --workspace <workspace> --input <path> --json`

## Chat-Only Path
Ask only for the missing workspace or missing planning input path, then provide the matching exact copyable planning command:

`boundline plan --workspace <workspace> --json`
`boundline plan --workspace <workspace> --input <path> --json`

If the active session has no captured goal or the user wants to change it, route to `/boundline-goal` instead and provide the matching exact copyable orchestrator command:

`boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`boundline orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
For Codex surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:run`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on `phase_request`, and surface `goal`, `authored_input_summary` or `authored_input_sources`, `execution_condition`, planning summary, key artifacts, `latest_status`, the exact raw `resume_command` when present, and the latest `next_command` only when no `resume_command` is available. Only surface raw `context_provenance`, `route_config_projection`, or guidance source dumps when the user explicitly asks for deeper detail.

Summarize the recorded goal or `authored_input_summary`, `authored_input_sources`, any requested governance intent, the resulting plan state, any proposed, confirmed, skipped, or absent `flow_state`, any `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, the emitted planning `phase_request` with its `request_id`, `stage_key`, `question`, `expected_answer.type`, `instruction`, `artifact.artifact_ref`, the exact raw `resume_command` when present, and the latest `next_command` only when no `resume_command` is available. When planning also reports `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, or `context_staleness_reason`, preserve those fields exactly. When those context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. If that context is Canon-grounded, also preserve governed artifact refs and stale-memory wording exactly and treat non-credible context as a real stop condition.

If plan output contains a structured gate (`phase_request`, approval-required, or blocked state), switch to interactive mode: ask the emitted question or the minimum follow-up question needed, wait for user input, and use the raw `resume_command` or CLI `next_command` only for the actual shell continuation. When `expected_answer.type` is `suggested_choice`, list `expected_answer.options` as suggested answers and state that custom text or a reference path is allowed. Treat legacy clarification fields as a compatibility fallback only when no `phase_request` is present.

## Next-Step Routing
Prefer an emitted `assistant_resume_command` when a `phase_request` is present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`.
For continuity docs and fallback copy, preserve slash-style assistant routes such as `/boundline-step` and `/boundline-run`; render them as host-native `/boundline:*` actions such as `/boundline:step` or `/boundline:run` on Codex surfaces. Route to `/boundline:plan` only when planning is blocked, context is non-credible, or brief authoring is still required.
Allowed follow-up commands: `/boundline:step`, `/boundline:run`, `/boundline:plan`, `/boundline:goal`.
