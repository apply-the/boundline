# Command: /boundline-inspect

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Inspect a specific or session-resolved Boundline trace and summarize the outcome.
When the user asks for the full session audit trail, actor attribution, voting lineage, or algorithm outcomes, switch to the dedicated audit surface by appending `--audit`.

If the resolved workspace trace reports compatibility ownership, keep that explicit: it now means the prior direct run opted into `--compatibility`, not that plain `run --goal` defaults there.

## Required Context
- `trace_ref` or `workspace_ref`
- Preserve any confirmed `latest_trace_ref` from prior turns

## Shell-Enabled Path
If the user wants the audit trail specifically, run the matching `inspect` command with `--audit`:

`boundline inspect --trace <trace> --audit --json`

or

`boundline inspect --workspace <workspace> --audit --json`

Otherwise, if `trace_ref` is known, run `boundline inspect --trace <trace> --json`. Otherwise, if `workspace_ref` is known, run `boundline inspect --workspace <workspace> --json`. If the assistant is already anchored in the target workspace and neither field is missing, run `boundline inspect --json` exactly once. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.

## Chat-Only Path
Ask only for the missing `trace_ref` or `workspace_ref`, then provide one exact copyable command:

`boundline inspect --trace <trace> --json`

or, for audit-specific inspection,

`boundline inspect --trace <trace> --audit --json`

or

`boundline inspect --workspace <workspace> --json`

or

`boundline inspect --workspace <workspace> --audit --json`

Wait for pasted output before continuing. If workspace-based inspect reports a session error, route to `/boundline-goal`. If trace reading fails, ask for a corrected trace reference or workspace and provide the replacement inspect command.

## Cross-Surface Consistency
Treat `status`, `inspect`, and assistant-rendered session summaries as projections of the same runtime state. Preserve degraded reasons, diagnostics, governed references, and fallback commands exactly, and never claim assistant-only state exists.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Reply as a compact operator brief by default: preserve `inspection_target` when present, `goal`, `authored_input_summary` or `authored_input_sources`, `routing` or `route_owner`, `execution_condition`, a concise inspection summary, key artifacts such as `trace` and checkpoint refs, governance blockers or `governance_next_action`, `latest_status`, and the CLI-reported `next_command`. Only surface raw `route_config_projection`, `context_provenance`, decision timelines, failure evidence dumps, or other deep trace detail when the user explicitly asks for deeper detail or wants the CLI `--verbose` view. Preserve `latest_trace_ref`, `authored_input_deduplicated_sources`, `governance_next_action`, `follow_through_guidance`, `follow_through_evidence_source`, `changed_files`, `validation`, and `corrected_command` exactly when present. Preserve `corrected_command` on failures, and keep delegated continuity, domain-template gaps, and Canon-governed credibility or stale-memory wording as real stop conditions.
When inspect reports `planning_analysis_state`, `planning_analysis_findings`, or `planning_analysis_coverage`, preserve them exactly. Treat `planning_analysis_state: blocked` as a real execution stop and route back to `/boundline:plan` instead of `/boundline:run`.
When inspect also reports `repository_map_state`, `snapshot_cache_state`, `context_pack_entries`, `omission_findings`, or `patch_safe_edit_attempts`, preserve them exactly. Use those fields to explain digest-backed compaction, archived-context visibility, unsafe oversized-read refusal, stale tracked cache, or patch-safe edit drift without inventing assistant-only state.
When `--audit` is used, treat the audit projection as the primary payload: preserve `audit_entry_count`, `audit_session_ref`, `audit_latest`, and the ordered `audit_timeline`. If any entry includes `participant_routes` or `mixed_routes`, keep that multi-route attribution explicit.

## Next-Step Routing
If workspace-based inspect reports a session error, route to `/boundline-goal`. Otherwise prefer the CLI-reported `next_command`.
Allowed follow-up commands: `/boundline-next`, `/boundline-run`, `/boundline-step`, `/boundline-status`, `/boundline-goal`.
