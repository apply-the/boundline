---
description: "Execute a bounded Boundline workflow"
---

# Command: /boundline-run

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Resume the active Boundline session through the selected runtime route until it reaches a terminal outcome.

## Required Context
- `workspace_ref`
- Captured goal or authored brief in the active session; do not ask for new input when it is already stored

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- run --workspace <workspace>` exactly once. If the host or wrapper needs the session-native event stream, treat `cargo run --bin boundline -- orchestrate --workspace <workspace> --assistant-host copilot --until terminal --json-stream` as the internal backend mapping for the same continuation. Do not mention that backend mapping in user-facing next-step copy when `/boundline-run` or `boundline run` already expresses the same action. If the active session has no captured goal or planned task, route to `/boundline-plan` or `/boundline-goal` instead of inventing a new run command. If the user explicitly asks for direct manifest-backed compatibility behavior without relying on active session state, route them to the direct compatibility workflow in the shared guidance instead of reinterpreting `/boundline-run`.

Internal backend mapping reference (keep hidden unless the user asks for backend details):

`cargo run --bin boundline -- orchestrate --workspace <workspace> --until terminal --json-stream`

For Copilot execution, append `--assistant-host copilot` to the backend mapping.

## Governed Continuation
Treat `/boundline-run` as the Boundline-first continuation surface. Canon-default governed shorthand may still appear as `boundline run --mode <mode>` when the operator explicitly requests a Canon-default governed mode, but do not rewrite it into `/boundline-run <mode>` or any per-mode `/boundline-<mode>` alias. If the CLI emits governance selection, approval wait, or missing governed input, preserve `governance_runtime`, `mode_selection_preference`, `selected_mode`, `approval_state`, and `next_action` exactly, then follow the emitted `resume_command`, `next_command`, or route to `/boundline-govern`.

## Chat-Only Path
If shell execution is unavailable, ask only for missing workspace context and then provide this exact copyable command:

`cargo run --bin boundline -- run --workspace <workspace>`

Wait for the user to paste the output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
When `assistant_next_command` or `assistant_resume_command` is available, render only that assistant-safe route in the visible follow-up. Do not describe the same continuation as both `boundline run` and `boundline orchestrate ...`, and do not phrase the next step as "run (or orchestrate)".
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, `goal` when present, `authored_input_summary` or `authored_input_sources`, `routing` or `route_owner`, `execution_condition`, a concise execution summary, key artifacts such as `trace` and checkpoint refs, governance blockers or `governance_next_action`, `latest_status`, any emitted `assistant_next_command`, and the emitted `next_command`. Only surface raw `route_config_projection`, `context_provenance`, guidance-source dumps, or long decision timelines when the user explicitly asks for deeper detail or wants the CLI `--verbose` view. Preserve the returned trace reference for later `/boundline-inspect` use. Preserve any governance wait-or-block guidance exactly when the CLI surfaces it, and do not paraphrase governance waits into permission to continue. If the stream emits a `phase_request`, stop there, preserve its `stage_key`, `instruction`, `artifact.artifact_ref`, any `assistant_resume_command`, and the exact raw `resume_command` including any `--planning-stage-complete <stage_key>` marker for shell execution only, and do not continue execution in chat.

If run output reports clarification or any gate boundary (approval wait,
blocked state, recovery requirement, or `phase_request`), move to interactive
follow-up. Ask the user for the required decision or missing input, present any
`assistant_resume_command` or `assistant_next_command` as the chat-facing path,
then use the CLI-reported raw command path only for shell execution.

## Next-Step Routing
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): `/boundline-inspect` — review what happened.
**Secondary** (shown only when delegation is pending or the step is incomplete and re-run is appropriate): `/boundline-run` — re-run.

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present — it overrides the primary.

Allowed follow-up commands: `/boundline-inspect`, `/boundline-status`, `/boundline-next`, `/boundline-run`, `/boundline-plan`, `/boundline-goal`.
