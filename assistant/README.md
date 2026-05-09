# Assistant Command Packs

This directory contains Markdown-based commands to run `boundline` from various AI assistants (Claude, Codex, Copilot, Gemini CLI).

The primary delivery surface is session-native: `start -> capture -> plan -> run -> status -> next -> inspect` against `<workspace>/.boundline/session.json` and `<workspace>/.boundline/traces/`.

In `0.46.0`, shell-enabled assistant flows should prefer `--json` for the session-native lifecycle commands plus `run`, `status`, `next`, and `inspect`. Treat `command_name`, `exit_status`, `rendered_output`, `trace_location`, `session_status`, and `trace_summary` as the authoritative host envelope when those fields are present, and use `rendered_output` only as the human-readable companion.

In `0.44.0`, assistants should treat installation verification as the first
boundary in a new environment: prefer the README quick path, run
`boundline doctor --install` before workspace commands, and only then move into the
session-native workflow.

Keep the product boundary explicit in assistant narration:

- Boundline owns orchestration, planning, execution, validation, and session state.
- Canon is the optional governed companion runtime, not the product entrypoint.
- If a user only needs the fast path, point them to README plus
	`docs/getting-started.md`; use `docs/architecture.md` only for the second
	read level.

In `0.44.0`, workflows and direct runs are primary surfaces of the same Boundline
product story, while compatibility remains explicit and subordinate.

In `0.44.0`, direct `run --goal` still bootstraps that native session path by
default, while `run --compatibility --goal ...` remains the explicit
execution-profile route. `capture` persists `negotiation_goal_summary`,
`negotiation_resolution`, and `negotiation_acceptance_boundary` before
planning. Default `plan` now persists one evidence-driven proposal, and
`plan --confirm` confirms that proposal before native execution can continue.
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

`boundline init` still scaffolds `<workspace>/.boundline/execution.json` plus local routing config, but that manifest is now an explicit compatibility/bootstrap surface rather than the default product story. When operators pass `--assistant claude|copilot|codex|gemini`, preserve the reported `route_setup`, including seeded routes, explicit overrides, `inspect_or_edit`, and any `fallback-from=<runtime>-unavailable` wording. When init reports `assistant_setup`, `workspace_hygiene`, or `next_steps`, preserve created, updated, unchanged, skipped, provenance, and follow-up wording exactly; those lines explain which bounded assistant and hygiene defaults were applied without overwriting local rules.

In the same release, `boundline doctor` now groups output into `summary`, `checks`, and `actions`. Preserve those section labels and follow-up commands exactly instead of paraphrasing them away, because they are now the first-run recovery surface for install and workspace readiness.

When a user asks for direct `run --goal`, assistants should prefer the native
route by default. Add `--compatibility` only when the user explicitly wants the
manifest-backed compatibility path.

When an explicit compatibility run leaves no resumable session, assistants
should treat `continuity_authority: compatibility_trace` as an inspect-only
follow-up state rather than a reason to restart from `boundline start`.

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

- `cargo run --bin boundline -- start --cluster <primary-workspace>`
- `cargo run --bin boundline -- capture --cluster <primary-workspace> --goal "<goal>"`
- `cargo run --bin boundline -- plan --cluster <primary-workspace>`
- `cargo run --bin boundline -- plan --cluster <primary-workspace> --confirm`
- `cargo run --bin boundline -- run --cluster <primary-workspace>`
- `cargo run --bin boundline -- status --cluster <primary-workspace>`
- `cargo run --bin boundline -- next --cluster <primary-workspace>`
- `cargo run --bin boundline -- inspect --cluster <primary-workspace>`

When a workspace defines `.boundline/workflows.toml`, assistants may also use the
bounded named-workflow surface: `workflow list -> workflow run -> workflow
status -> workflow resume -> workflow inspect`. Those commands reuse the same session and trace
story instead of opening a second runtime, including governed `bug-fix:investigate`
approval waits, blocked outcomes, and later packet reuse toward governed verify.

Canon is default for governed mode shorthand. When the user names one Canon mode,
assistants may use `/boundline-run <mode>` or the direct alias for that mode; both
map to `boundline run --mode <mode>` and must preserve `governance_runtime`,
`mode_selection_preference`, `selected_mode`, `approval_state`, and
`next_action` from CLI output. Supported aliases are `/boundline-requirements`,
`/boundline-discovery`, `/boundline-system-shaping`, `/boundline-architecture`,
`/boundline-backlog`, `/boundline-change`, `/boundline-implementation`,
`/boundline-refactor`, `/boundline-review`, `/boundline-verification`,
`/boundline-incident`, `/boundline-security-assessment`,
`/boundline-system-assessment`, `/boundline-migration`, and
`/boundline-supply-chain-analysis`.

Canon-default setup and config commands are also first-class assistant surfaces:
`/boundline-init` maps to `boundline init`, `/boundline-doctor` maps to
`boundline doctor --install`, `/boundline-config-show` maps to
`boundline config show --scope workspace`, `/boundline-config-set-canon` maps to
`boundline config set-canon --mode-selection <manual|auto-confirm|auto>`, and
`/boundline-capture` maps to `boundline capture --goal ... --brief ...`.
Assistants should collect missing chat answers first, then run or provide the
same CLI commands without asking operators to edit manifests manually.

## Directory Structure
- **Claude**: `claude/commands/`
- **Codex**: `codex/commands/`
- **Copilot**: `copilot/prompts/`

## Installation & Registration
Each AI assistant has its own local or remote configuration. Currently, all command packs must be registered as local file references.

- **Copilot**: Copy `./assistant/copilot/prompts/*.prompt.md` to `.github/prompts/` or reference via `#file`.
- **Claude**: Load the respective `.md` files as projects or upload as attachments to the context window.
- **Codex**: Import into the corresponding workbench.
- **Gemini CLI**: Reference the command docs from this directory and run the mapped Boundline CLI commands locally.

Gemini remains an explicit CLI fallback in this release. Claude, Codex, and
Copilot command packs should follow the active route slot binding instead of
assuming one hard-wired backend.

## Fallback Conventions
Since an assistant may be executed in a context *without* shell access (e.g., standard chat window), each command must gracefully degrade.

If the shell/terminal is *not* available:
1. Provide the user with the correct CLI command. For session-native lifecycle commands, `run`, `status`, `next`, and `inspect`, prefer the same command with `--json` so the pasted output stays structured.
2. Provide a brief explanation of what the command does.
3. Tell the user to run it manually, wait for it to finish, and paste the output.

If the shell/terminal *is* available:
1. Run the mapped CLI command directly from the repository root with `cargo run --bin boundline -- ...`. For session-native lifecycle commands, `run`, `status`, `next`, and `inspect`, append `--json` unless the user explicitly asked for plain text.
2. Do not explain syntax.
3. Prefer CLI-reported `next_command` or `corrected_command` when present instead of inventing a follow-up.

## Workflows

### Starting a Workflow (User Story 1)
- `/boundline-init`: Runs `cargo run --bin boundline -- init --workspace <workspace>` before first use or when workspace setup is missing. Add `--template <change|delivery>` only when the user explicitly wants a different starting profile than the default `bug-fix`. Use `--force` when replacing an existing generated profile.
- `/boundline-start`: Confirms the workspace and runs `cargo run --bin boundline -- start --workspace <workspace>` to initialize the active session.
- `/boundline-plan`: Captures human-authored input into the active session, then runs `cargo run --bin boundline -- plan --workspace <workspace>`. When the user gives direct text, use `cargo run --bin boundline -- capture --workspace <workspace> --goal "<goal>"`. When the user provides Markdown brief files, use `cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`. When both are present, pass both `--goal` and repeated `--brief` flags in the same capture command. Summaries should preserve proposed, confirmed, skipped, or absent flow state, `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, the negotiated delivery fields, and any CLI-reported confirm or clarification guidance.

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
condition until the CLI points to `boundline plan --confirm`.

When the user asks to tune defaults for planning, verification, or review roles,
assistants should use `cargo run --bin boundline -- config show|set|unset ...`
instead of asking users to edit config files manually.
When the user asks to tune domain families, layered standards, or supporting
external inputs, assistants should use `cargo run --bin boundline -- config show`,
`config set-domain`, `config unset-domain`, `config bind-context`, and
`config unbind-context` instead of editing `.boundline/config.toml` directly.

If the user explicitly selects a built-in flow, assistants should run `cargo run --bin boundline -- flow <bug-fix|change|delivery> --workspace <workspace>` after capture and before plan. There is no separate assistant command pack for `flow`; use the raw CLI subcommand directly.

### Continuing a Workflow (User Story 2)
- `/boundline-step`: Executes `cargo run --bin boundline -- step --workspace <workspace>` and summarizes `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, `next_command`, and flow-stage fields when present.
- `/boundline-run`: Executes `cargo run --bin boundline -- run --workspace <workspace>` and summarizes `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `execution_path`, `flow_state`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `terminal_status`, `terminal_reason`, `changed_files`, validation summaries, `trace`, `next_command`, and any flow/stage lifecycle events. When adaptive execution is active, also summarize `workspace_slice`, `candidate_family`, `selection_headline`, `selection_reason`, `rejected_candidates`, explicit adaptive exhaustion when present, and `attempt_lineage`. When review is configured, also summarize `review_trigger`, reviewer findings, `review_vote`, and `review_outcome`. When governance is active, also summarize `latest_governance_stage`, `latest_governance_runtime`, `latest_governance_mode`, `latest_governance_run_ref`, packet provenance including `latest_governance_packet_ref` and any binding reason, approval state, any packet rejection or blocked rationale, and `governance_next_action` when present.
- `/boundline-status`: Executes `cargo run --bin boundline -- status --workspace <workspace>` and summarizes the active session state or latest compatibility follow-up for the current workspace, including `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `compatibility_follow_up_command`, `execution_path`, `flow_state`, `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, `latest_decision_status`, `latest_decision_target`, `active_flow`, `current_stage`, `stage_progress`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `latest_changed_files`, `latest_workspace_slice`, `latest_selection_headline`, `latest_candidate_family`, `latest_selection_reason`, `latest_rejected_candidates`, `latest_attempt_lineage`, `latest_validation_status`, `latest_exhaustion_reason`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and the latest review fields when available. When governance is active, surface `latest_governance_stage`, `latest_governance_state`, `latest_governance_mode`, `latest_governance_run_ref`, `latest_governance_packet_ref`, any packet binding reason, autopilot candidates, and `governance_next_action` so the operator knows whether to wait for approval or resolve a blocker instead of continuing execution.
- `/boundline-next`: Executes `cargo run --bin boundline -- next --workspace <workspace>` and summarizes `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `latest_status`, `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, `verification_strategy`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `explanation`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and the CLI-reported `next_command`, plus flow-stage context, the latest adaptive slice, `candidate_family`, selection reason, exhaustion reason when present, validation state, and the latest review outcome when present.

When `status`, `next`, or `inspect` surface compatibility follow-up, treat it as
evidence that the user previously chose `run --compatibility`; do not infer that
plain `run --goal` should continue to use the compatibility route.

### Named Workflow Layer (Workflow Slice)
- Use `/boundline-workflow-list`, `/boundline-workflow-run`, `/boundline-workflow-status`, `/boundline-workflow-resume`, and `/boundline-workflow-inspect` as the assistant-native entrypoints when a workspace provides `.boundline/workflows.toml`.
- `cargo run --bin boundline -- workflow list --workspace <workspace>` should summarize the available workflow names, any shipped summary or `recommended_when` guidance, the declared phase chain, and the exact `workflow run` command to start each one.
- `cargo run --bin boundline -- workflow run <name> --workspace <workspace> [--goal "<goal>"]` should summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, and `next_command`, including actionable paused or blocked states when bounded `review` or `govern` follow-through cannot complete yet.
- `cargo run --bin boundline -- workflow status --workspace <workspace>` should report the same session story as `status`, with workflow identity, active phase, route projection, and any workflow-owned next action added.
- `cargo run --bin boundline -- workflow resume --workspace <workspace>` should be preferred over inventing a phase-specific follow-up when the CLI reports it as `next_command`, especially for bounded `review` or `govern` follow-through.
- `cargo run --bin boundline -- workflow inspect --workspace <workspace>` should combine workflow projection with trace inspection when a trace exists and should preserve the same primary-versus-subordinate product story.

### Inspecting Prior Runs (User Story 3)
- `/boundline-inspect`: Executes `cargo run --bin boundline -- inspect --trace <trace>` for an explicit trace or `cargo run --bin boundline -- inspect --workspace <workspace>` for the workspace-selected trace. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.
- Successful inspection summaries must expose `inspection_target`, `trace`, `routing_summary`, `route_owner`, `route_config_projection` when present, `execution_condition`, `goal_plan_summary`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `delegation_mode`, `delegation_packet_id`, `delegation_packet_kind`, `delegation_packet_state`, `delegation_target_owner`, `delegation_headline`, `delegation_evidence_summary`, `decision_timeline`, `failure_evidence`, `adaptive_evidence`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, `terminal_status`, `terminal_reason`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources` when present, changed-file headlines, validation headlines, adaptive slice and lineage evidence when present, `candidate_family`, selection reason, rejected candidates, explicit exhaustion when present, review trigger, reviewer findings, vote summary, review outcome, governance runtime/mode/run-ref evidence, governance packet provenance when present, `governance_next_action` when present, and `next_command` so assistants can continue routing without dumping raw logs.
- Trace-read failures must expose `terminal_reason`, `next_command: /boundline-inspect`, and a `corrected_command` that tells the user how to retry with a corrected trace reference or workspace. Workspace-based inspect session errors should route back to `/boundline-start`.

For the current adaptive execution manifest shape, broader bounded mutation-family vocabulary, and validation-guided bounded replanning behavior, see [`docs/adaptive-execution.md`](../docs/adaptive-execution.md).

For the current review manifest shape and vote semantics, see [`docs/review-voting.md`](../docs/review-voting.md).

## Continuity Rules
- Preserve confirmed `workspace_ref`, captured goal, confirmed brief paths, authored input summary, and latest trace reference across assistant turns.
- Preserve `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` across assistant turns once capture or planning reports them.
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
- When `status` or `next` reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/boundline-inspect` instead of `/boundline-start`.
- When CLI output includes `corrected_command`, reuse it instead of inventing a replacement inspect invocation.
- When governance output reports `awaiting_approval` or `blocked`, do not suggest an ungoverned bypass; prefer `status` or `inspect` exactly as the CLI recommends and surface `governance_next_action` when the CLI provides it.
- When a user is operating on a registered cluster, keep the primary workspace authoritative and prefer the CLI-reported `--cluster <primary-workspace>` follow-up instead of replacing it with member-scoped `--workspace` commands.
