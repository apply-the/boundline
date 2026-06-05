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
If the workspace is known, prefer the session-native backend mapping exactly once:

`boundline orchestrate --workspace <workspace> --until terminal --json-stream`

If the active session has no captured goal or planned task, route to `/boundline-plan` or `/boundline-goal` instead of inventing a new run command. If the user explicitly asks for direct manifest-backed compatibility behavior without relying on active session state, route them to the direct compatibility workflow in the shared guidance instead of reinterpreting `/boundline-run`.

## Governed Continuation
Treat `/boundline-run` as the Boundline-first continuation surface. Canon-default governed shorthand may still appear as `boundline run --mode <mode>` when the operator explicitly requests a Canon-default governed mode, but do not rewrite it into `/boundline-run <mode>` or any per-mode `/boundline-<mode>` alias. If the CLI emits governance selection, approval wait, or missing governed input, preserve `governance_runtime`, `mode_selection_preference`, `selected_mode`, `approval_state`, and `next_action` exactly, then follow the emitted `resume_command`, `next_command`, or route to `/boundline-govern`.

## Backlog Quality Gate
Canon backlog is governed source material, while Boundline validates execution readiness. Preserve `backlog_quality_state`, `backlog_quality_findings`, `backlog_task_count`, `backlog_mvp_scope`, and `backlog_unmapped_items` when they appear in status or run output. If `backlog_quality_state` is `blocked` or `clarification_required`, do not route to `/boundline:run`; route to `/boundline:plan` or the emitted planning `phase_request` continuation instead.

## Planning Analysis Gate
Planning analysis is the final Boundline planning gate before execution. Preserve `planning_analysis_state`, `planning_analysis_findings`, and `planning_analysis_coverage` when they appear in status or run output. If `planning_analysis_state` is `blocked`, do not route to `/boundline:run`; route to `/boundline:plan` or the emitted planning continuation instead.

When run output also reports `repository_map_state`, `snapshot_cache_state`, `context_pack_entries`, `omission_findings`, or `patch_safe_edit_attempts`, preserve them exactly. Treat blocking omission findings, unsafe oversized-read refusal, stale tracked cache, or patch-safe edit drift as planning repairs, not as permission to continue execution.
When run output also reports `capability_provider_status`, `capability_provider_id`, `capability_provider_activation_state`, `capability_provider_validation_disposition`, `capability_provider_failure_class`, `capability_provider_accepted_evidence_refs`, `capability_provider_rejected_evidence_refs`, or `capability_provider_limitations`, preserve them exactly. Treat readiness, permission, execution, or post-execution validation failures as real bounded stops rather than assistant-only retry signals.

## Chat-Only Path
If shell execution is unavailable, ask only for missing workspace context and then provide this exact copyable command:

`boundline orchestrate --workspace <workspace> --until terminal --json-stream`

Wait for the user to paste the output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
For Antigravity surfaces, render host-native actions using `/boundline:*` command ids (for example `/boundline:inspect`).
Reply as a compact operator brief by default: preserve the ordered NDJSON event sequence, `goal` when present, `authored_input_summary` or `authored_input_sources`, `routing` or `route_owner`, `execution_condition`, a concise execution summary, key artifacts such as `trace` and checkpoint refs, governance blockers or `governance_next_action`, `latest_status`, any emitted `resume_command`, and the emitted `next_command`. Only surface raw `route_config_projection`, `context_provenance`, guidance-source dumps, or long decision timelines when the user explicitly asks for deeper detail or wants the CLI `--verbose` view. Preserve the returned trace reference for later `/boundline-inspect` use. Preserve any governance wait-or-block guidance exactly when the CLI surfaces it, and do not paraphrase governance waits into permission to continue. If the stream emits a `phase_request`, stop there, preserve its `request_id`, `stage_key`, `question`, `expected_answer.type`, `instruction`, `artifact.artifact_ref`, and the exact raw `resume_command` including any `--planning-stage-complete <stage_key>` or `--request-id <request_id>` marker for shell execution only, and do not continue execution in chat.

If run output reports clarification or any gate boundary (approval wait, blocked state, recovery requirement, or `phase_request`), move to interactive follow-up. Ask the user for the required decision or missing input, then use the CLI-reported raw command path only for shell execution.

## Next-Step Routing
Surface host-native action links using `/boundline:*` command ids.
Prefer `assistant_resume_command` when a `phase_request` is present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`. Default to `/boundline:inspect` after a run completes; route to `/boundline:run` only when delegation is pending or the step is incomplete and re-run is appropriate.
Allowed follow-up commands: `/boundline:inspect`, `/boundline:status`, `/boundline:continue`, `/boundline:run`, `/boundline:plan`, `/boundline:goal`.
