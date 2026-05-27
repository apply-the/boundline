# Assistant Command Packs

This directory contains Markdown-based commands to run `boundline` from various AI assistants (Claude, Codex, Copilot, and Antigravity package surfaces).

The primary delivery surface is session-native: `goal -> plan -> run -> status -> next -> inspect` against `<workspace>/.boundline/session.json` and `<workspace>/.boundline/traces/`.

Assistant narration and any optional LLM chat remain advisory. They can explain the current session and suggest the next command, but they do not replace the Rust runtime or create authoritative state.

When a user asks for a consolidated state view, prefer `boundline status --workspace <workspace>` for a compact summary or `boundline inspect --workspace <workspace>` for the detailed runtime projection. Repo-local assistant command packs bootstrapped by `boundline init --assistant <host>` and refreshed by `boundline update --workspace <workspace> --target assistant --apply` should surface that same session-native truth in chat. Do not infer state from chat history or treat any assistant surface as a second workflow engine.

Human-readable Boundline answers should stay compact by default across the CLI and assistant chat surfaces. Prefer a short operator brief that preserves `goal`, `authored_input_summary` or `authored_input_sources`, `routing`, `execution_condition`, a concise bounded summary, key artifacts, `latest_status`, and `next_command`. Low-signal dumps such as `route_config_projection`, `context_provenance`, guidance source lists, or raw retrieval candidate lists belong in `inspect`, `status --verbose`, trace refs, or governed artifacts unless the operator explicitly asks for deeper detail.

**IMPORTANT FORMATTING RULES:**
1. You MUST translate all state information into natural, conversational language. DO NOT use raw JSON keys, snake_case strings, or bulleted lists of machine fields (e.g., do not output `next_command: boundline plan` or `latest_status: goal_captured`). Integrate the information naturally into your reply.
2. For follow-up actions (like the next command), DO NOT output plain text like "Prossimo step consigliato: `/boundline:plan`". Instead, provide a clickable Markdown action for the active host.
3. Follow host-specific action rendering rules for `assistant_resume_command` or `assistant_next_command`:
   - Copilot prompt surfaces: use `command:github.copilot.chat.execute` links, for example `[▶ Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)`.
   - Codex, Claude, and Antigravity package surfaces: render host-native slash command actions using `/boundline:*` command ids (for example `/boundline:plan`), and do not emit Copilot-specific URIs.
   - Cursor copy-ready surfaces: render deterministic fallback actions as slash command text or CLI follow-up guidance; do not claim verified clickable URI parity.
4. When both an assistant-safe route (`assistant_resume_command` or `assistant_next_command`) and a raw CLI/backend continuation (`resume_command` or `next_command`) are available, show only the assistant-safe route to the user. Keep the raw CLI command for hidden shell execution only. Do not present equivalent continuations such as `/boundline-run`, `boundline run`, and `boundline orchestrate --assistant-host ...` as parallel user-facing alternatives unless the user explicitly asks for backend details.

The runtime also persists the latest compact operator briefs under `.boundline/briefs/goal.md`, `.boundline/briefs/plan.md`, and `.boundline/briefs/run.md`. Treat those files as runtime-owned latest summaries for the active session. They complement, but do not replace, stage-specific governed artifacts such as `.boundline/governance/planning/<stage>/brief.md` and Canon packet documents.

The first-response assistant surfaces are part of that same session-native
runtime rather than a parallel report. Preserve
`why_summary`, `risk_summary`, `evidence_summary`, `source_attribution`,
`fallback_disclosure`, `confidence_level`, and `next_best_action` exactly when
they appear on `status` or `inspect`. These lines are the authoritative backing
for `/boundline:why`, `/boundline:risk`, `/boundline:evidence`, and
`/boundline:next-best`, including partial-setup answers where Canon is not yet
available.

The same release adds deeper cognitive lens lines for `/boundline:assumptions`,
`/boundline:hidden-impact`, `/boundline:challenge`, and
`/boundline:explain-plan`. Preserve `assumptions_summary`, every
`assumption_group`, `hidden_impact_summary`, all surfaced `hidden_impact_*`
detail lines, `hidden_impact_fallback_disclosure`, every `challenge_*` line,
and every `explain_plan_*` line exactly. When `challenge_required_review` or
`challenge_council_required` appears, do not paraphrase that governance
boundary away or imply the assistant can bypass it.

## Interactive Clarification And Gate Contract

Across all session steps (`start`, `goal`, `plan`, `run`, `status`, `next`,
`inspect`, and workflow variants), assistants should treat clarification and
approval gates as interactive boundaries, not as static summaries.

When CLI output includes a structured `phase_request`:

1. Treat it as the authoritative interactive boundary.
2. Explain `phase_request.reason` in one concise line.
3. Ask exactly the emitted `phase_request.question`.
4. Preserve `phase_request.request_id`, `phase_request.expected_answer`, any
	`assistant_resume_command` or `assistant_next_command`, and the raw
	`resume_command` for the next valid continuation.
5. Wait for the user's answer or external artifact update before issuing the
	next mutating command.

Answer-type rendering:

| `expected_answer.type` | Rendering contract |
|---|---|
| `free_text` | Ask for an open text answer; no predefined options are meaningful. |
| `suggested_choice` | Render `expected_answer.options` as selectable suggestions or a visible list, but keep a custom/free-text answer path. Selecting a suggestion sends that option's `value`. |
| `single_choice` | Require one selection from `expected_answer.options`; selecting an option sends that option's `value`. |
| `confirmation` | Render a yes/no review gate. Use this for approval boundaries, not open clarification. |
| `multi_choice` | Render multiple selections from `expected_answer.options`; if the host lacks multi-select, show a numbered list and collect text. |

Legacy clarification fields (`clarification_prompt`,
`clarification_missing_fields`, and `clarification_questions`) are
compatibility fallback only when no `phase_request` is emitted:

1. Ask follow-up questions directly in chat.
2. Ask one bounded question at a time unless the user explicitly asks for a
	full checklist.
3. Wait for user answers before issuing the next mutating command.
4. Re-run the mapped Boundline command with updated inputs after collecting
	missing answers.

When CLI output includes other gate conditions (for example governance waits,
approval-required states, blocked states, or checkpoint restore boundaries):

1. Stop progression at that gate.
2. Explain the gate in one concise line.
3. Ask the minimum interactive question needed to choose the next valid path.
4. Prefer any assistant-safe `assistant_resume_command` or
	`assistant_next_command` when present for the chat-facing route.
5. Use the raw CLI-reported `resume_command`, `next_command`, or restore command
	only for the actual shell continuation path instead of inventing a new command path.

Do not continue silently past clarification-required or approval-required
states, and do not treat a list of questions as sufficient completion.

Shell-enabled assistant flows should preserve advanced-context projection
fields exactly when they appear on `plan`, `status`, and `inspect`:
`retrieval_mode`, `retrieval_state`, `retrieval_index_state`, selected
evidence, relationship lines, impact findings, and any explicit disabled or
degraded reason. These fields explain the baseline local SQLite + FTS5
retrieval path and must not be paraphrased away.

The active semantic-acceleration path extends those same fields rather than
creating another report. Preserve `semantic_policy_state`,
`semantic_capability_state`, `hybrid_outcome`, `semantic_selected_count`,
`semantic_rejected_count`, candidate `match_origin`, and any surfaced
`rejected_candidate:` lines exactly when they appear. They explain whether the
local semantic path expanded or reranked the V1 set, or why Boundline stayed
on the lexical baseline.

In `0.56.0`, shell-enabled assistant flows should prefer `--json` for the session-native lifecycle commands plus `run`, `status`, `next`, and `inspect`. Treat `command_name`, `exit_status`, `rendered_output`, `trace_location`, `session_status`, and `trace_summary` as the authoritative host envelope when those fields are present, and use `rendered_output` only as the human-readable companion.

In `0.56.0`, assistant plugin packages expose Boundline through `.claude-plugin/`,
`.codex-plugin/`, `.cursor-plugin/`, and `.copilot-prompts/`. Package commands
use `/boundline:*` names and must preserve `.boundline/session.json`,
CLI-reported `next_command`, and explicit blocked, clarification-required,
failed, exhausted, and terminal states. `/boundline:govern` is conditional:
Canon governance is only visible when the workspace is configured for it or the
user explicitly asks for governed delivery.
Canon is the optional governed companion runtime.

The same release adds explicit review-council projection fields such as
`latest_review_council_profile`, `latest_review_independence_state`, and
`latest_review_stop_semantics`. Preserve them exactly when present: they are the
operator-facing explanation for why a bounded review can proceed, wait, or stop.

In the same release, bundled guidance and guardian standards can arrive through
directory-based catalog packs such as `assistant/packs/guidance-catalog/`.
When `plan`, `status`, or `inspect` report `loaded_packs`, `skipped_packs`, or
`catalog_validation_findings`, preserve those fields exactly: they explain why
catalog content did or did not participate in the bounded run.

Treat the chat surface in three layers:

- global bootstrap commands such as `/boundline:init`, `/boundline:doctor`, `/boundline:help`, `/boundline:status`, and `/boundline:continue` for install, readiness, and repo setup before `.boundline/session.json` exists
- repo-local runtime commands such as `/boundline:goal`, `/boundline:plan`, `/boundline:run`, `/boundline:status`, `/boundline:next`, `/boundline:inspect`, and `/boundline:recover` for the active session and trace-backed runtime state
- guided delivery-intent commands such as workflow entrypoints or governed mode shorthands when the operator wants a bounded delivery phase surfaced directly

Large work is supported by decomposition, not by unbounded autonomy.

In `0.44.0`, assistants should treat installation verification as the first
boundary in a new environment: prefer the README quick path, run
`boundline doctor --install` before workspace commands, and only then move into the
session-native workflow.

Keep the product boundary explicit in assistant narration:

- Boundline pilots the work through orchestration, decomposition, planning, execution, validation, and session state.
- Canon governs packets, approvals, and governed artifacts when a delivery boundary requires it; it is not the product entrypoint.
- If a user only needs the fast path, point them to README plus
	`docs/getting-started.md`; use `docs/architecture.md` only for the second
	read level.

In `0.44.0`, workflows and direct runs are primary surfaces of the same Boundline
product story, while compatibility remains explicit and subordinate.

In `0.44.0`, direct `run --goal` still bootstraps that native session path by
default, while `run --compatibility --goal ...` remains the explicit
execution-profile route. `goal` persists `negotiation_goal_summary`,
`negotiation_resolution`, and `negotiation_acceptance_boundary` before
planning. Default `plan` now persists one evidence-driven proposal, and
session-native `run` applies that approval when execution is ready to continue.
Assistants should preserve those fields across `plan`, `run`, `status`,
`next`, and `inspect` instead of paraphrasing them away.

In the same release, native execution also keeps explicit selector-driven
guidance visible on the read-side surfaces. Preserve `latest_selection_headline`,
`latest_selection_reason`, and inspect `selector:` lines exactly when they
appear: they explain which bounded action Boundline chose next and why.

In the same release, native planning also persists `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, and
`context_staleness_reason` when available. Preserve those values exactly: they
explain why planning is bounded enough to continue or why it stopped.

Treat authored brief file refs, recent changed files after failed validation,
and other explicit evidence anchors as causal reasons the planner selected a
target. Do not paraphrase them into generic keyword matches.

In the same release, those context fields can also carry the selected domain
family, the winning standards source, and any used, stale, unavailable, or
required external-input status for active domain templates. Preserve that
domain wording exactly and treat missing or stale required domain inputs as
real stop conditions rather than optional hints.

In the same release, Canon capability snapshots and compact Canon-grounded
memory can also populate those context fields and `governance_next_action`
across `plan`, `run`, `status`, `next`, and `inspect`. Preserve that governed
memory exactly, including artifact-backed provenance and stale-memory wording:
it can be the authoritative reason Boundline stops instead of continuing.

Native planning now also persists `goal_plan_state`, `goal_plan_revision`,
`planning_rationale`, and `verification_strategy` when available. Preserve
those values exactly: they explain whether the current proposal is still
waiting for confirmation, what changed across revisions, and how Boundline expects
to validate the bounded plan.

`boundline assistant install --host <host> --scope user` reports global
bootstrap assets from `assistant/global/` for `/boundline:init`,
`/boundline:doctor`, `/boundline:help`, `/boundline:status`, and
`/boundline:continue`. These commands must detect whether the workspace is
initialized and must fall back to exact CLI commands when the host cannot run a
shell. `boundline continue` must not infer state from chat history when
`.boundline/session.json` is absent.
The CLI runtime remains authoritative. Host chat history is not authoritative;
chat history is not authoritative.

`boundline init` still scaffolds `<workspace>/.boundline/execution.json` plus local routing config, but that manifest is now an explicit compatibility/bootstrap surface rather than the default product story. When operators pass `--assistant claude|copilot|codex|antigravity`, preserve the reported `route_setup`, including seeded routes, explicit overrides, `inspect_or_edit`, `assistant_package_scope: repo-local`, and any `fallback-from=<runtime>-unavailable` wording. Antigravity scaffolds the repo-local package surface without declaring a provider runtime by itself; explicit provider routing still belongs on `--route SLOT=RUNTIME:MODEL`, where Gemini remains valid as a model runtime. When operators also pass `--export-docs`, Boundline mirrors a stable Canon reference plus the selected assistant reference files under `<workspace>/docs/boundline/` by default or another root via `--to`; that export is create-only unless the operator explicitly asks for `--refresh` or `--force`, and `--diff` previews changes without writing. When init reports `assistant_setup`, `docs_export`, `workspace_hygiene`, or `next_steps`, preserve created, updated, unchanged, skipped, provenance, and follow-up wording exactly; those lines explain which bounded assistant and hygiene defaults were applied without overwriting local rules.

In the same release, `boundline doctor` now groups output into `summary`, `checks`, and `actions`. Preserve those section labels and follow-up commands exactly instead of paraphrasing them away, because they are now the first-run recovery surface for install and workspace readiness.

When a user asks for direct `run --goal`, assistants should prefer the native
route by default. Add `--compatibility` only when the user explicitly wants the
manifest-backed compatibility path.

When an explicit compatibility run leaves no resumable session, assistants
should treat `continuity_authority: compatibility_trace` as an inspect-only
follow-up state rather than a reason to restart from `boundline goal`.

When `run`, `status`, `next`, or `inspect` report `route_owner` and
`route_config_projection`, assistants should preserve those fields in their
working state and use them when explaining why a route or config default is
authoritative.

When `route_config_projection` includes `effective_routing`,
`assistant_bindings`, `runtime_capabilities`, or `slot_effort_policies`,
preserve those values exactly. They now describe the resolved slot route, its
source, the bound assistant family, the effective capability or effort policy,
and the persisted route snapshot used during execution rather than only the
current workspace config file.

When `status`, `next`, or `inspect` report `follow_through_guidance`,
`follow_through_evidence_source`, `follow_through_next_action`, or
`follow_through_stop_reason`, preserve those values exactly. They describe why
one bounded follow-up action is currently credible and whether the guidance came
from persisted session state or the authoritative trace.

When a native run reports `delegation_mode`, `delegation_packet_kind`, or other
`delegation_*` fields because the active route cannot continue inside declared
`assistant_runtimes`, assistant availability, or runtime capability policy,
treat that as a real stop condition. Do not silently switch assistant
families; use the reported packet, target owner, evidence summary, and
`next_command` to explain whether the user should hand off, inspect, replan,
or resolve the blocking route policy first.

When those same commands report negotiated delivery fields, assistants should
keep the active acceptance boundary explicit and treat
`pending_clarification`, `conflicting`, or `blocked` negotiation states as
real stop conditions rather than as hints to proceed anyway.

When bounded `bug-fix` or `change` work reaches a terminal state, assistants
should treat missing `latest_changed_files` or a non-`passed`
`latest_validation_status` as a real failed-delivery outcome rather than as a
successful completion. When governed delivery succeeds, preserve
`latest_changed_files`, `latest_validation_status`, and any governed packet
lineage fields exactly.

When a bounded delivery story spans a registered cluster, assistants should use
`--cluster <primary-workspace>` for the session-native commands instead of
switching ownership to a member workspace. Preserve `cluster_route_owner`,
`cluster_authoritative_workspace`, `cluster_execution_condition`,
`cluster_participating_workspaces`, and `cluster_blocking_workspace` when those
fields appear in CLI output.

In the same release, mutating `run` and `step` create local rollback manifests.
Preserve `latest_checkpoint_id`, `latest_checkpoint_scope`, and
`latest_checkpoint_restore_command` exactly when they appear on `run`, `status`,
`next`, or `inspect`, and prefer the reported restore command over generic
restart advice when a bounded change is blocked or failed.

Clustered session-native delivery uses the same CLI surface through the primary
workspace:

- `boundline goal --cluster <primary-workspace> --goal "<goal>"`
- `boundline plan --cluster <primary-workspace>`
- `boundline plan --cluster <primary-workspace> --confirm`
- `boundline run --cluster <primary-workspace>`
- `boundline status --cluster <primary-workspace>`
- `boundline next --cluster <primary-workspace>`
- `boundline inspect --cluster <primary-workspace>`

When a workspace defines `.boundline/workflows.toml`, assistants may invoke the
bounded named-workflow CLI surface directly: `workflow list -> workflow run -> workflow
status -> workflow resume -> workflow inspect`. Those commands reuse the same session and trace
story instead of opening a second runtime, including governed `bug-fix:investigate`
approval waits, blocked outcomes, and later packet reuse toward governed verify.
Do not expose dedicated `/boundline-workflow-*` prompt surfaces; use the workflow
CLI only when the operator explicitly asks for named-workflow discovery or execution,
or when the CLI-reported `next_command` points there.

Canon-governed work stays behind Boundline's primary workflow surface.
Assistants should use `/boundline-run` for normal continuation and preserve
`governance_runtime`, `mode_selection_preference`, `selected_mode`,
`approval_state`, and `next_action` exactly when the CLI emits them. Do not
promote `/boundline-run <mode>` or per-mode `/boundline-<mode>` aliases as the
primary UX.

Workspace maintenance and config commands are also first-class assistant surfaces.
Repo bootstrap itself remains CLI-only through `boundline init --assistant <host>`.
After bootstrap, `/boundline-update` maps to `boundline update`, `/boundline-doctor` maps to
`boundline doctor --install`, `/boundline-config-show` maps to
`boundline config show --scope workspace`, `/boundline-config-set-canon` maps to
`boundline config set-canon --mode-selection <manual|auto-confirm|auto>`,
`/boundline-goal` maps to `boundline orchestrate --goal ... --brief ... --until phase-request --json-stream` for assistant-host interactive capture, while direct `boundline goal --goal ... --brief ...` remains the raw non-interactive primitive,
`/boundline-recover` maps to `boundline status --json` followed by the
CLI-reported recovery command, and `/boundline-govern` maps to
`boundline govern --workspace <workspace> --json`, optionally with
`--mode <mode>` only when the user explicitly supplied one.
Assistants should collect missing chat answers first, then run or provide the
same CLI commands without asking operators to edit manifests manually.

The cognitive follow-up commands are also first-class assistant surfaces:
`/boundline-why`, `/boundline-risk`, `/boundline-evidence`, `/boundline-next-best`,
`/boundline-assumptions`, `/boundline-hidden-impact`, `/boundline-challenge`, and
`/boundline-explain-plan` each run the matching `boundline` subcommand against the
active session and surface the result without inferring missing context from chat
history. `/boundline-doctor-context` maps to `boundline doctor --workspace <workspace>`
and must summarize `boundline_config`, `canon_project_memory`, `expert_pack_inputs`,
`provider_readiness`, `advanced_context_index`, and `session_evidence`, plus the
CLI-reported fix commands. Keep advisory gaps explicit.

## Directory Structure
- **Claude**: `claude/commands/`
- **Codex**: `codex/commands/`
- **Copilot**: `copilot/prompts/`

## Installation & Registration
Each AI assistant has its own local or remote configuration. Currently, all command packs must be registered as local file references.

- **Copilot**: `boundline init --assistant copilot` scaffolds `./assistant/copilot/prompts/`, `.copilot-prompts/`, and mirrors the generated prompts into `.github/prompts/` for VS Code discovery. Use `#file` when you want ad hoc prompt references instead of repo-local discovery.
- **Claude**: `boundline init --assistant claude` scaffolds `.claude-plugin/` plus `./assistant/claude/commands/`; complete any host-specific registration from that repo-local package root.
- **Codex**: `boundline init --assistant codex` scaffolds `.codex-plugin/` plus `./assistant/codex/commands/`; import or register that repo-local package in the host.
- **Antigravity**: `boundline init --assistant antigravity` scaffolds `.antigravity-plugin/` plus `./assistant/antigravity/commands/`; complete any host-specific registration from that repo-local package root.
- **Cursor**: `boundline assistant install --host cursor --scope user` emits copy-ready bootstrap assets and guidance. Treat Cursor as `copy-ready-assets`: installation details remain host-specific and the CLI stays authoritative.

After the first bootstrap, refresh repo-local assistant assets with `boundline update --workspace <workspace> --target assistant --apply` instead of rerouting through a dedicated init prompt.

For pre-init host chat bootstrap, use `boundline assistant install --host
<claude|codex|cursor|copilot|antigravity> --scope user`. Copilot and Antigravity output
manual fallback guidance instead of claiming a universal global plugin install.

Repo-local support modes in this release are explicit: Claude, Codex, and
Copilot are `repo-local-full`; Antigravity is `repo-local-full`; Cursor is `copy-ready-assets`. Assistants should not claim stronger parity than those
declared support modes, and all hosts must treat CLI output plus
`.boundline/session.json` as authoritative.

## Fallback Conventions
Since an assistant may be executed in a context *without* shell access (e.g., standard chat window), each command must gracefully degrade.

If the shell/terminal is *not* available:
1. Provide the user with the correct CLI command. For session-native lifecycle commands, `run`, `status`, `next`, and `inspect`, prefer the same command with `--json` so the pasted output stays structured.
2. Provide a brief explanation of what the command does.
3. Tell the user to run it manually, wait for it to finish, and paste the output.

If the shell/terminal *is* available:
1. Run the mapped CLI command directly from the repository root with `boundline ...`. For session-native lifecycle commands, `run`, `status`, `next`, and `inspect`, append `--json` unless the user explicitly asked for plain text.
2. Do not explain syntax.
3. Prefer CLI-reported `next_command` or `corrected_command` when present instead of inventing a follow-up.

## Workflows

### Starting a Workflow (User Story 1)
- `/boundline-update`: Runs `boundline update --workspace <workspace>` by default to preview Boundline-managed workspace drift. Add `--target assistant` when the user explicitly wants only the repo-local assistant package refreshed. Add `--apply`, `--force`, `--adopt`, or `--prune` only when the user explicitly wants to mutate the workspace or is following the CLI-reported repair path. Summaries should preserve `update_status`, `targets`, manifest and tracked-artifact state, and the CLI-reported `next_steps` exactly.
- `/boundline-goal`: Confirms the workspace and runs `boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream` for direct goal text, `boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream` for brief-only capture, or the combined `--goal` plus repeated `--brief` variant when both are present. Treat this as the canonical assistant-host interactive path for goal capture and clarification. Direct `boundline goal ...` remains the non-interactive capture primitive for raw shell workflows.
- `/boundline-plan`: Plans the active session from the already captured goal. Use `boundline plan --workspace <workspace> --json` by default and `boundline plan --workspace <workspace> --input <path> --json` only when the user is supplying planning-only input for that same goal. If the user is introducing or changing the goal, route to `/boundline-goal` instead so runtime-owned goal clarification stays on the goal surface. Summaries should preserve proposed, confirmed, skipped, or absent flow state, `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, the negotiated delivery fields, and any CLI-reported confirm or clarification guidance.

When `plan`, `run`, `status`, `next`, or `inspect` report `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, or
`context_staleness_reason`, assistants should preserve those fields exactly and
surface any explicit non-credible context as a real stop condition.

When those same commands surface Canon-grounded memory through the context or
governance fields, assistants should preserve `governance_next_action`, packet
or artifact refs, and stale-memory wording exactly rather than paraphrasing it
into a generic suggestion.

When those same commands report `goal_plan_state`, `goal_plan_revision`,
`planning_rationale`, or `verification_strategy`, assistants should preserve
those fields exactly and treat an unconfirmed proposal as a real stop
condition until the CLI points to `boundline run` or another runtime-issued
continuation command.

When the user asks to tune defaults for planning, verification, or review roles,
assistants should use `boundline config show|set|unset ...`
instead of asking users to edit config files manually.
When the user asks to tune domain families, layered standards, or supporting
external inputs, assistants should use `boundline config show`,
`config set-domain`, `config unset-domain`, `config bind-context`, and
`config unbind-context` instead of editing `.boundline/config.toml` directly.

If the user explicitly selects a built-in flow, assistants should run `boundline flow <bug-fix|change|delivery> --workspace <workspace>` after recording the goal and before plan. There is no separate assistant command pack for `flow`; use the raw CLI subcommand directly.

### Continuing a Workflow (User Story 2)
- `/boundline-step`: Executes `boundline step --workspace <workspace>` and summarizes `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, `next_command`, and flow-stage fields when present.
- `/boundline-run`: Executes `boundline run --workspace <workspace>` and summarizes `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `execution_path`, `flow_state`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `terminal_status`, `terminal_reason`, `changed_files`, validation summaries, `trace`, `next_command`, and any flow/stage lifecycle events. When adaptive execution is active, also summarize `workspace_slice`, `candidate_family`, `selection_headline`, `selection_reason`, `rejected_candidates`, explicit adaptive exhaustion when present, and `attempt_lineage`. When review is configured, also summarize `review_trigger`, reviewer findings, `review_vote`, and `review_outcome`. When governance is active, also summarize `latest_governance_stage`, `latest_governance_runtime`, `latest_governance_mode`, `latest_governance_run_ref`, packet provenance including `latest_governance_packet_ref` and any binding reason, approval state, any packet rejection or blocked rationale, and `governance_next_action` when present.
- `/boundline-status`: Executes `boundline status --workspace <workspace>` and summarizes the active session state or latest compatibility follow-up for the current workspace, including `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `compatibility_follow_up_command`, `execution_path`, `flow_state`, `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, `latest_decision_status`, `latest_decision_target`, `active_flow`, `current_stage`, `stage_progress`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `latest_changed_files`, `latest_workspace_slice`, `latest_selection_headline`, `latest_candidate_family`, `latest_selection_reason`, `latest_rejected_candidates`, `latest_attempt_lineage`, `latest_validation_status`, `latest_exhaustion_reason`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and the latest review fields when available. When governance is active, surface `latest_governance_stage`, `latest_governance_state`, `latest_governance_mode`, `latest_governance_run_ref`, `latest_governance_packet_ref`, any packet binding reason, autopilot candidates, and `governance_next_action` so the operator knows whether to wait for approval or resolve a blocker instead of continuing execution.
- `/boundline-next`: Executes `boundline next --workspace <workspace>` and summarizes `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `latest_status`, `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `explanation`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and the CLI-reported `next_command`, plus flow-stage context, the latest adaptive slice, `candidate_family`, selection reason, exhaustion reason when present, validation state, and the latest review outcome when present.

When `status`, `next`, or `inspect` surface compatibility follow-up, treat it as
evidence that the user previously chose `run --compatibility`; do not infer that
plain `run --goal` should continue to use the compatibility route.

### Named Workflow Layer (Workflow Slice)
- Named workflows remain CLI-only for assistants. When a workspace provides `.boundline/workflows.toml`, assistants may run `boundline workflow list --workspace <workspace>`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` directly instead of exposing dedicated slash-command prompts.
- `boundline workflow list --workspace <workspace>` should summarize the available workflow names, any shipped summary or `recommended_when` guidance, the declared phase chain, and the exact `workflow run` command to start each one.
- `boundline workflow run <name> --workspace <workspace> [--goal "<goal>"]` should summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, and `next_command`, including actionable paused or blocked states when bounded `review` or `govern` follow-through cannot complete yet.
- `boundline workflow status --workspace <workspace>` should report the same session story as `status`, with workflow identity, active phase, route projection, and any workflow-owned next action added.
- `boundline workflow resume --workspace <workspace>` should be preferred over inventing a phase-specific follow-up when the CLI reports it as `next_command`, especially for bounded `review` or `govern` follow-through.
- `boundline workflow inspect --workspace <workspace>` should combine workflow projection with trace inspection when a trace exists and should preserve the same primary-versus-subordinate product story.

### Inspecting Prior Runs (User Story 3)
- `/boundline-inspect`: Executes `boundline inspect --trace <trace>` for an explicit trace or `boundline inspect --workspace <workspace>` for the workspace-selected trace. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.
- Successful inspection summaries must expose `inspection_target`, `trace`, `routing_summary`, `route_owner`, `route_config_projection` when present, `execution_condition`, `goal_plan_summary`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `decision_timeline`, `failure_evidence`, `adaptive_evidence`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, `terminal_status`, `terminal_reason`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources` when present, changed-file headlines, validation headlines, adaptive slice and lineage evidence when present, `candidate_family`, selection reason, rejected candidates, explicit exhaustion when present, review trigger, reviewer findings, vote summary, review outcome, governance runtime/mode/run-ref evidence, governance packet provenance when present, `governance_next_action` when present, and `next_command` so assistants can continue routing without dumping raw logs.
- Trace-read failures must expose `terminal_reason`, `next_command: /boundline-inspect`, and a `corrected_command` that tells the user how to retry with a corrected trace reference or workspace. Workspace-based inspect session errors should route back to `/boundline-goal`.

For the current adaptive execution manifest shape, broader bounded mutation-family vocabulary, and validation-guided bounded replanning behavior, see [`docs/adaptive-execution.md`](../docs/adaptive-execution.md).

For the current review manifest shape and vote semantics, see [`docs/review-voting.md`](../docs/review-voting.md).

## Continuity Rules
- Preserve confirmed `workspace_ref`, recorded goal, confirmed brief paths, authored input summary, and latest trace reference across assistant turns.
- Preserve `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` across assistant turns once goal intake or planning reports them.
- Preserve `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, and `context_staleness_reason` across assistant turns once planning or follow-through reports them.
- Preserve Canon-grounded `governance_next_action`, governed artifact refs, and any stale-memory wording once `plan`, `run`, `status`, `next`, or `inspect` reports them.
- Preserve `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, and `verification_strategy` across assistant turns once planning or follow-through reports them.
- Preserve `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, and `delegation_evidence_summary` across assistant turns once native follow-through reports them.
- Preserve `continuity_authority`, `compatibility_trace_ref`, and `compatibility_follow_up_command` when the CLI reports them after an explicit compatibility run.
- Preserve `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, and `follow_through_stop_reason` when the CLI reports them on `status`, `next`, or `inspect`.
- Preserve `cluster_id`, `cluster_route_owner`, `cluster_authoritative_workspace`, `cluster_execution_condition`, `cluster_participating_workspaces`, and `cluster_blocking_workspace` when the CLI reports them during clustered delivery.
- Preserve `latest_candidate_family`, `latest_selection_reason`, and `latest_exhaustion_reason` when adaptive compatibility output includes them.
- Preserve the selected flow name when the user has committed to `bug-fix`, `change`, or `delivery`.
- Ask only for missing fields before recommending or executing a command.
- In chat-only mode, always provide exact copyable commands, wait for the user to run them, and update the workflow state only after pasted output.
- Preserve `inspection_target` when the user is working from an explicit trace instead of the latest workspace trace.
- When CLI output includes `next_command`, prefer that route instead of inventing a follow-up.
- When `status` or `next` reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/boundline-inspect` instead of `/boundline-goal`.
- When CLI output includes `corrected_command`, reuse it instead of inventing a replacement inspect invocation.
- When governance output reports `awaiting_approval` or `blocked`, do not suggest an ungoverned bypass; prefer `status` or `inspect` exactly as the CLI recommends and surface `governance_next_action` when the CLI provides it.
- When a user is operating on a registered cluster, keep the primary workspace authoritative and prefer the CLI-reported `--cluster <primary-workspace>` follow-up instead of replacing it with member-scoped `--workspace` commands.
