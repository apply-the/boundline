---
description: "Inspect a Boundline trace and summarize outcome and recovery signals"
---

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

`cargo run --bin boundline -- inspect --trace <trace> --audit --json`

or

`cargo run --bin boundline -- inspect --workspace <workspace> --audit --json`

Otherwise, if `trace_ref` is known, run `cargo run --bin boundline -- inspect --trace <trace> --json`. Otherwise, if `workspace_ref` is known, run `cargo run --bin boundline -- inspect --workspace <workspace> --json`. If the assistant is already anchored in the target workspace and neither field is missing, run `cargo run --bin boundline -- inspect --json` exactly once. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.

## Chat-Only Path
Ask only for the missing `trace_ref` or `workspace_ref`, then provide one exact copyable command:

`cargo run --bin boundline -- inspect --trace <trace> --json`

or, for audit-specific inspection,

`cargo run --bin boundline -- inspect --trace <trace> --audit --json`

or

`cargo run --bin boundline -- inspect --workspace <workspace> --json`

or

`cargo run --bin boundline -- inspect --workspace <workspace> --audit --json`

Wait for pasted output before continuing. If workspace-based inspect reports a session error, route to `/boundline-goal`. If trace reading fails, ask for a corrected trace reference or workspace and provide the replacement inspect command.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Reply as a compact operator brief by default: preserve `inspection_target` when present, `goal`, `authored_input_summary` or `authored_input_sources`, `routing` or `route_owner`, `execution_condition`, a concise inspection summary, key artifacts such as `trace` and checkpoint refs, governance blockers or `governance_next_action`, `latest_status`, and the CLI-reported `next_command`. Only surface raw `route_config_projection`, `context_provenance`, decision timelines, failure evidence dumps, or other deep trace detail when the user explicitly asks for deeper detail or wants the CLI `--verbose` view. Preserve `latest_trace_ref`, `authored_input_deduplicated_sources`, `governance_next_action`, `follow_through_guidance`, `follow_through_evidence_source`, `changed_files`, `validation`, and `corrected_command` exactly when present. Preserve `corrected_command` on failures, and keep delegated continuity, domain-template gaps, and Canon-governed credibility or stale-memory wording as real stop conditions.
When `--audit` is used, treat the audit projection as the primary payload: preserve `audit_entry_count`, `audit_session_ref`, `audit_latest`, and the ordered `audit_timeline`. If any entry includes `participant_routes` or `mixed_routes`, keep that multi-route attribution explicit.

## Next-Step Routing
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): the CLI-reported `next_command` (typically `/boundline-step` or `/boundline-run`).
**Secondary** (shown only when the session needs reset or no active session exists): `/boundline-goal` — start a new goal.

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present — it overrides the primary.

Allowed follow-up commands: `/boundline-next`, `/boundline-run`, `/boundline-step`, `/boundline-status`, `/boundline-goal`.