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

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`
`cargo run --bin boundline -- plan --workspace <workspace> --input <path> --json`

Ask only for missing workspace or missing goal source. Reuse confirmed brief paths instead of asking for them again.

Canonical session-native backend mapping reference:

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`cargo run --bin boundline -- plan --workspace <workspace> --input <path> --json`

## Chat-Only Path
Ask only for the missing workspace or goal source, then provide the matching exact copyable orchestrator command:

`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream`
`cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...] --until phase-request --json-stream`
`cargo run --bin boundline -- plan --workspace <workspace> --input <path> --json`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
For Claude surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:run`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, stop on `phase_request`, and surface `goal`, `authored_input_summary` or `authored_input_sources`, `execution_condition`, planning summary, key artifacts, `latest_status`, the exact raw `resume_command` when present, and the latest `next_command` only when no `resume_command` is available. Only surface raw `context_provenance`, `route_config_projection`, or guidance source dumps when the user explicitly asks for deeper detail.

Summarize the recorded goal or `authored_input_summary`, `authored_input_sources`, any requested governance intent, the resulting plan state, any proposed, confirmed, skipped, or absent `flow_state`, any `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, the emitted planning `phase_request` with its `request_id`, `stage_key`, `question`, `expected_answer.type`, `instruction`, `artifact.artifact_ref`, the exact raw `resume_command` when present, and the latest `next_command` only when no `resume_command` is available. When planning also reports `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, or `context_staleness_reason`, preserve those fields exactly. When those context fields include domain-template selection, winning standards source, or external-input status, preserve that wording exactly and treat missing or stale required domain inputs as a real stop condition. If that context is Canon-grounded, also preserve governed artifact refs and stale-memory wording exactly and treat non-credible context as a real stop condition.

If plan output contains a structured gate (`phase_request`, approval-required, or blocked state), switch to interactive mode: ask the emitted question or the minimum follow-up question needed, wait for user input, and use the raw `resume_command` or CLI `next_command` only for the actual shell continuation. Treat legacy clarification fields as a compatibility fallback only when no `phase_request` is present.

## Next-Step Routing
Prefer an emitted `resume_command` when a `phase_request` is present; otherwise prefer the CLI-reported `next_command`.
Allowed follow-up commands: `/boundline:step`, `/boundline:run`, `/boundline:plan`, `/boundline:goal`.
