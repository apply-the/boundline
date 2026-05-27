# Command: /boundline-plan

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Plan the active session into a bounded proposal from an already captured goal.

`/boundline-plan` must not capture or refine the goal. If the user is introducing a new goal, changing the current goal, or supplying brief files that should become goal evidence, route to `/boundline-goal` instead.

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
For Antigravity surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:run`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on `phase_request`, and surface `goal`, `authored_input_summary` or `authored_input_sources`, `execution_condition`, planning summary, key artifacts, `latest_status`, the exact raw `resume_command` when present, and the latest `next_command` only when no `resume_command` is available. Only surface raw `context_provenance`, `route_config_projection`, or guidance source dumps when the user explicitly asks for deeper detail.

Summarize the recorded goal or `authored_input_summary`, `authored_input_sources`, any requested governance intent, the resulting plan state, any proposed, confirmed, skipped, or absent `flow_state`, any `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, the emitted planning `phase_request` with its `request_id`, `stage_key`, `question`, `expected_answer.type`, `instruction`, `artifact.artifact_ref`, the exact raw `resume_command` when present, and the latest `next_command` only when no `resume_command` is available. When planning also reports `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, or `context_staleness_reason`, preserve those fields exactly. When those context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. If that context is Canon-grounded, also preserve governed artifact refs and stale-memory wording exactly and treat non-credible context as a real stop condition.

If plan output contains a structured gate (`phase_request`, approval-required, or blocked state), switch to interactive mode: ask the emitted question or the minimum follow-up question needed, wait for user input, and use the raw `resume_command` or CLI `next_command` only for the actual shell continuation. When `expected_answer.type` is `suggested_choice`, list `expected_answer.options` as suggested answers and state that custom text or a reference path is allowed. Treat legacy clarification fields as a compatibility fallback only when no `phase_request` is present.

## Next-Step Routing
Prefer an emitted `assistant_resume_command` when a `phase_request` is present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`.
For continuity docs and fallback copy, preserve slash-style assistant routes such as `/boundline-step` and `/boundline-run`; render them as host-native `/boundline:*` actions such as `/boundline:step` or `/boundline:run` on Antigravity surfaces. Route to `/boundline:plan` only when planning is blocked, context is non-credible, or brief authoring is still required.
Allowed follow-up commands: `/boundline:step`, `/boundline:run`, `/boundline:plan`, `/boundline:goal`.
