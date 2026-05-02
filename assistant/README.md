# Assistant Command Packs

This directory contains Markdown-based commands to run `synod` from various AI assistants (Claude, Codex, Copilot, Gemini CLI).

The primary delivery surface is session-native: `start -> capture -> plan -> run -> status -> next -> inspect` against `<workspace>/.synod/session.json` and `<workspace>/.synod/traces/`.

In `0.33.0`, workflows and direct runs are primary surfaces of the same Synod
product story, while compatibility remains explicit and subordinate.

In `0.33.0`, direct `run --goal` still bootstraps that native session path by
default, while `run --compatibility --goal ...` remains the explicit
execution-profile route. `capture` persists `negotiation_goal_summary`,
`negotiation_resolution`, and `negotiation_acceptance_boundary` before
planning. Assistants should preserve those fields across `plan`, `run`,
`status`, `next`, and `inspect` instead of paraphrasing them away.

In the same release, native planning also persists `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, and
`context_staleness_reason` when available. Preserve those values exactly: they
explain why planning is bounded enough to continue or why it stopped.

`synod init` still scaffolds `<workspace>/.synod/execution.json` plus local routing config, but that manifest is now an explicit compatibility/bootstrap surface rather than the default product story.

When a user asks for direct `run --goal`, assistants should prefer the native
route by default. Add `--compatibility` only when the user explicitly wants the
manifest-backed compatibility path.

When an explicit compatibility run leaves no resumable session, assistants
should treat `continuity_authority: compatibility_trace` as an inspect-only
follow-up state rather than a reason to restart from `synod start`.

When `run`, `status`, `next`, or `inspect` report `route_owner` and
`route_config_projection`, assistants should preserve those fields in their
working state and use them when explaining why a route or config default is
authoritative.

When `route_config_projection` includes `effective_routing` or
`assistant_bindings`, preserve those values exactly. They now describe the
resolved slot route, its source, the bound assistant family, and the persisted
route snapshot used during execution rather than only the current workspace
config file.

When `status`, `next`, or `inspect` report `follow_through_guidance`,
`follow_through_evidence_source`, `follow_through_next_action`, or
`follow_through_stop_reason`, preserve those values exactly. They describe why
one bounded follow-up action is currently credible and whether the guidance came
from persisted session state or the authoritative trace.

When a native run reports an assistant-binding failure because the active route
requires a runtime outside declared `assistant_runtimes`, treat that as a real
stop condition. Do not silently switch assistant families; tell the user to
change routing or assistant capabilities first.

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

Clustered session-native delivery uses the same CLI surface through the primary
workspace:

- `cargo run --bin synod -- start --cluster <primary-workspace>`
- `cargo run --bin synod -- capture --cluster <primary-workspace> --goal "<goal>"`
- `cargo run --bin synod -- plan --cluster <primary-workspace>`
- `cargo run --bin synod -- run --cluster <primary-workspace>`
- `cargo run --bin synod -- status --cluster <primary-workspace>`
- `cargo run --bin synod -- next --cluster <primary-workspace>`
- `cargo run --bin synod -- inspect --cluster <primary-workspace>`

When a workspace defines `.synod/workflows.toml`, assistants may also use the
bounded named-workflow surface: `workflow list -> workflow run -> workflow
status -> workflow resume -> workflow inspect`. Those commands reuse the same session and trace
story instead of opening a second runtime, including governed `bug-fix:investigate`
approval waits, blocked outcomes, and later packet reuse toward governed verify.

## Directory Structure
- **Claude**: `claude/commands/`
- **Codex**: `codex/commands/`
- **Copilot**: `copilot/prompts/`

## Installation & Registration
Each AI assistant has its own local or remote configuration. Currently, all command packs must be registered as local file references.

- **Copilot**: Copy `./assistant/copilot/prompts/*.prompt.md` to `.github/prompts/` or reference via `#file`.
- **Claude**: Load the respective `.md` files as projects or upload as attachments to the context window.
- **Codex**: Import into the corresponding workbench.
- **Gemini CLI**: Reference the command docs from this directory and run the mapped Synod CLI commands locally.

Gemini remains an explicit CLI fallback in this release. Claude, Codex, and
Copilot command packs should follow the active route slot binding instead of
assuming one hard-wired backend.

## Fallback Conventions
Since an assistant may be executed in a context *without* shell access (e.g., standard chat window), each command must gracefully degrade.

If the shell/terminal is *not* available:
1. Provide the user with the correct CLI command.
2. Provide a brief explanation of what the command does.
3. Tell the user to run it manually, wait for it to finish, and paste the output.

If the shell/terminal *is* available:
1. Run the mapped CLI command directly from the repository root with `cargo run --bin synod -- ...`.
2. Do not explain syntax.
3. Prefer CLI-reported `next_command` or `corrected_command` when present instead of inventing a follow-up.

## Workflows

### Starting a Workflow (User Story 1)
- `/synod-init`: Runs `cargo run --bin synod -- init --workspace <workspace>` before first use or when workspace setup is missing. Add `--template <change|delivery>` only when the user explicitly wants a different starting profile than the default `bug-fix`. Use `--force` when replacing an existing generated profile.
- `/synod-start`: Confirms the workspace and runs `cargo run --bin synod -- start --workspace <workspace>` to initialize the active session.
- `/synod-plan`: Captures human-authored input into the active session, then runs `cargo run --bin synod -- plan --workspace <workspace>`. When the user gives direct text, use `cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`. When the user provides Markdown brief files, use `cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`. When both are present, pass both `--goal` and repeated `--brief` flags in the same capture command. Summaries should preserve proposed, confirmed, skipped, or absent flow state, the negotiated delivery fields, and any CLI-reported confirm, skip, or clarification guidance.

When `plan`, `run`, `status`, `next`, or `inspect` report `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, or
`context_staleness_reason`, assistants should preserve those fields exactly and
surface any explicit non-credible context as a real stop condition.

When the user asks to tune defaults for planning, verification, or review roles,
assistants should use `cargo run --bin synod -- config show|set|unset ...`
instead of asking users to edit config files manually.

If the user explicitly selects a built-in flow, assistants should run `cargo run --bin synod -- flow <bug-fix|change|delivery> --workspace <workspace>` after capture and before plan. There is no separate assistant command pack for `flow`; use the raw CLI subcommand directly.

### Continuing a Workflow (User Story 2)
- `/synod-step`: Executes `cargo run --bin synod -- step --workspace <workspace>` and summarizes `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, `next_command`, and flow-stage fields when present.
- `/synod-run`: Executes `cargo run --bin synod -- run --workspace <workspace>` and summarizes `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `execution_path`, `flow_state`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `terminal_status`, `terminal_reason`, `changed_files`, validation summaries, `trace`, `next_command`, and any flow/stage lifecycle events. When adaptive execution is active, also summarize `workspace_slice`, `candidate_family`, `selection_headline`, `selection_reason`, `rejected_candidates`, explicit adaptive exhaustion when present, and `attempt_lineage`. When review is configured, also summarize `review_trigger`, reviewer findings, `review_vote`, and `review_outcome`. When governance is active, also summarize `latest_governance_stage`, `latest_governance_runtime`, `latest_governance_mode`, `latest_governance_run_ref`, packet provenance including `latest_governance_packet_ref` and any binding reason, approval state, any packet rejection or blocked rationale, and `governance_next_action` when present.
- `/synod-status`: Executes `cargo run --bin synod -- status --workspace <workspace>` and summarizes the active session state or latest compatibility follow-up for the current workspace, including `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `compatibility_follow_up_command`, `execution_path`, `flow_state`, `latest_decision_status`, `latest_decision_target`, `active_flow`, `current_stage`, `stage_progress`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `latest_changed_files`, `latest_workspace_slice`, `latest_selection_headline`, `latest_candidate_family`, `latest_selection_reason`, `latest_rejected_candidates`, `latest_attempt_lineage`, `latest_validation_status`, `latest_exhaustion_reason`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and the latest review fields when available. When governance is active, surface `latest_governance_stage`, `latest_governance_state`, `latest_governance_mode`, `latest_governance_run_ref`, `latest_governance_packet_ref`, any packet binding reason, autopilot candidates, and `governance_next_action` so the operator knows whether to wait for approval or resolve a blocker instead of continuing execution.
- `/synod-next`: Executes `cargo run --bin synod -- next --workspace <workspace>` and summarizes `routing`, `route_owner`, `route_config_projection` when present, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `latest_status`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `explanation`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, and the CLI-reported `next_command`, plus flow-stage context, the latest adaptive slice, `candidate_family`, selection reason, exhaustion reason when present, validation state, and the latest review outcome when present.

When `status`, `next`, or `inspect` surface compatibility follow-up, treat it as
evidence that the user previously chose `run --compatibility`; do not infer that
plain `run --goal` should continue to use the compatibility route.

### Named Workflow Layer (Workflow Slice)
- Use `/synod-workflow-list`, `/synod-workflow-run`, `/synod-workflow-status`, `/synod-workflow-resume`, and `/synod-workflow-inspect` as the assistant-native entrypoints when a workspace provides `.synod/workflows.toml`.
- `cargo run --bin synod -- workflow list --workspace <workspace>` should summarize the available workflow names, any shipped summary or `recommended_when` guidance, the declared phase chain, and the exact `workflow run` command to start each one.
- `cargo run --bin synod -- workflow run <name> --workspace <workspace> [--goal "<goal>"]` should summarize `workflow`, `workflow_phase`, `routing`, `route_owner`, `route_config_projection`, `execution_condition`, `execution_path`, and `next_command`, including actionable paused or blocked states when bounded `review` or `govern` follow-through cannot complete yet.
- `cargo run --bin synod -- workflow status --workspace <workspace>` should report the same session story as `status`, with workflow identity, active phase, route projection, and any workflow-owned next action added.
- `cargo run --bin synod -- workflow resume --workspace <workspace>` should be preferred over inventing a phase-specific follow-up when the CLI reports it as `next_command`, especially for bounded `review` or `govern` follow-through.
- `cargo run --bin synod -- workflow inspect --workspace <workspace>` should combine workflow projection with trace inspection when a trace exists and should preserve the same primary-versus-subordinate product story.

### Inspecting Prior Runs (User Story 3)
- `/synod-inspect`: Executes `cargo run --bin synod -- inspect --trace <trace>` for an explicit trace or `cargo run --bin synod -- inspect --workspace <workspace>` for the workspace-selected trace. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.
- Successful inspection summaries must expose `inspection_target`, `trace`, `routing_summary`, `route_owner`, `route_config_projection` when present, `execution_condition`, `goal_plan_summary`, `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, `decision_timeline`, `failure_evidence`, `adaptive_evidence`, `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, `follow_through_stop_reason`, `terminal_status`, `terminal_reason`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources` when present, changed-file headlines, validation headlines, adaptive slice and lineage evidence when present, `candidate_family`, selection reason, rejected candidates, explicit exhaustion when present, review trigger, reviewer findings, vote summary, review outcome, governance runtime/mode/run-ref evidence, governance packet provenance when present, `governance_next_action` when present, and `next_command` so assistants can continue routing without dumping raw logs.
- Trace-read failures must expose `terminal_reason`, `next_command: /synod-inspect`, and a `corrected_command` that tells the user how to retry with a corrected trace reference or workspace. Workspace-based inspect session errors should route back to `/synod-start`.

For the current adaptive execution manifest shape, broader bounded mutation-family vocabulary, and validation-guided bounded replanning behavior, see [`docs/adaptive-execution.md`](../docs/adaptive-execution.md).

For the current review manifest shape and vote semantics, see [`docs/review-voting.md`](../docs/review-voting.md).

## Continuity Rules
- Preserve confirmed `workspace_ref`, captured goal, confirmed brief paths, authored input summary, and latest trace reference across assistant turns.
- Preserve `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` across assistant turns once capture or planning reports them.
- Preserve `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, and `context_staleness_reason` across assistant turns once planning or follow-through reports them.
- Preserve `continuity_authority`, `compatibility_trace_ref`, and `compatibility_follow_up_command` when the CLI reports them after an explicit compatibility run.
- Preserve `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, and `follow_through_stop_reason` when the CLI reports them on `status`, `next`, or `inspect`.
- Preserve `cluster_id`, `cluster_route_owner`, `cluster_authoritative_workspace`, `cluster_execution_condition`, `cluster_participating_workspaces`, and `cluster_blocking_workspace` when the CLI reports them during clustered delivery.
- Preserve `latest_candidate_family`, `latest_selection_reason`, and `latest_exhaustion_reason` when adaptive compatibility output includes them.
- Preserve the selected flow name when the user has committed to `bug-fix`, `change`, or `delivery`.
- Ask only for missing fields before recommending or executing a command.
- In chat-only mode, always provide exact copyable commands, wait for the user to run them, and update the workflow state only after pasted output.
- Preserve `inspection_target` when the user is working from an explicit trace instead of the latest workspace trace.
- When CLI output includes `next_command`, prefer that route instead of inventing a follow-up.
- When `status` or `next` reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/synod-inspect` instead of `/synod-start`.
- When CLI output includes `corrected_command`, reuse it instead of inventing a replacement inspect invocation.
- When governance output reports `awaiting_approval` or `blocked`, do not suggest an ungoverned bypass; prefer `status` or `inspect` exactly as the CLI recommends and surface `governance_next_action` when the CLI provides it.
- When a user is operating on a registered cluster, keep the primary workspace authoritative and prefer the CLI-reported `--cluster <primary-workspace>` follow-up instead of replacing it with member-scoped `--workspace` commands.
