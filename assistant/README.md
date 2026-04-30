# Assistant Command Packs

This directory contains Markdown-based commands to run `synod` from various AI assistants (Claude, Codex, Copilot, Gemini CLI).

The primary delivery surface is session-native: `start -> capture -> plan -> run -> status -> next -> inspect` against `<workspace>/.synod/session.json` and `<workspace>/.synod/traces/`.

`synod init` still scaffolds `<workspace>/.synod/execution.json` plus local routing config, but that manifest is now an explicit compatibility/bootstrap surface rather than the default product story.

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
- `/synod-plan`: Captures human-authored input into the active session, then runs `cargo run --bin synod -- plan --workspace <workspace>`. When the user gives direct text, use `cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`. When the user provides Markdown brief files, use `cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`. When both are present, pass both `--goal` and repeated `--brief` flags in the same capture command. Summaries should preserve proposed, confirmed, skipped, or absent flow state and any CLI-reported confirm or skip guidance.

When the user asks to tune defaults for planning, verification, or review roles,
assistants should use `cargo run --bin synod -- config show|set|unset ...`
instead of asking users to edit config files manually.

If the user explicitly selects a built-in flow, assistants should run `cargo run --bin synod -- flow <bug-fix|change|delivery> --workspace <workspace>` after capture and before plan. There is no separate assistant command pack for `flow`; use the raw CLI subcommand directly.

### Continuing a Workflow (User Story 2)
- `/synod-step`: Executes `cargo run --bin synod -- step --workspace <workspace>` and summarizes `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, `next_command`, and flow-stage fields when present.
- `/synod-run`: Executes `cargo run --bin synod -- run --workspace <workspace>` and summarizes `routing`, `execution_condition`, `execution_path`, `flow_state`, `terminal_status`, `terminal_reason`, `changed_files`, validation summaries, `trace`, `next_command`, and any flow/stage lifecycle events. When adaptive execution is active, also summarize `workspace_slice` and `attempt_lineage`. When review is configured, also summarize `review_trigger`, reviewer findings, `review_vote`, and `review_outcome`. When governance is active, also summarize `latest_governance_runtime`, `latest_governance_mode`, `latest_governance_run_ref`, packet provenance including `latest_governance_packet_ref` and any binding reason, approval state, any packet rejection or blocked rationale, and `governance_next_action` when present.
- `/synod-status`: Executes `cargo run --bin synod -- status --workspace <workspace>` and summarizes the active session state for the current workspace, including `routing`, `execution_condition`, `execution_path`, `flow_state`, `latest_decision_status`, `latest_decision_target`, `active_flow`, `current_stage`, `stage_progress`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `latest_changed_files`, `latest_workspace_slice`, `latest_selection_headline`, `latest_attempt_lineage`, `latest_validation_status`, and the latest review fields when available. When governance is active, surface `latest_governance_state`, `latest_governance_mode`, `latest_governance_run_ref`, `latest_governance_packet_ref`, any packet binding reason, autopilot candidates, and `governance_next_action` so the operator knows whether to wait for approval instead of continuing execution.
- `/synod-next`: Executes `cargo run --bin synod -- next --workspace <workspace>` and summarizes `routing`, `execution_condition`, `latest_status`, `explanation`, and the CLI-reported `next_command`, plus flow-stage context, the latest adaptive slice and validation state, and the latest review outcome when present.

### Inspecting Prior Runs (User Story 3)
- `/synod-inspect`: Executes `cargo run --bin synod -- inspect --trace <trace>` for an explicit trace or `cargo run --bin synod -- inspect --workspace <workspace>` for the workspace-selected trace. Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.
- Successful inspection summaries must expose `inspection_target`, `trace`, `routing_summary`, `execution_condition`, `goal_plan_summary`, `decision_timeline`, `failure_evidence`, `terminal_status`, `terminal_reason`, `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources` when present, changed-file headlines, validation headlines, adaptive slice and lineage evidence when present, review trigger, reviewer findings, vote summary, review outcome, governance runtime/mode/run-ref evidence, governance packet provenance when present, `governance_next_action` when present, and `next_command` so assistants can continue routing without dumping raw logs.
- Trace-read failures must expose `terminal_reason`, `next_command: /synod-inspect`, and a `corrected_command` that tells the user how to retry with a corrected trace reference or workspace. Workspace-based inspect session errors should route back to `/synod-start`.

For the current adaptive execution manifest shape and bounded replanning behavior, see [`docs/adaptive-execution.md`](../docs/adaptive-execution.md).

For the current review manifest shape and vote semantics, see [`docs/review-voting.md`](../docs/review-voting.md).

## Continuity Rules
- Preserve confirmed `workspace_ref`, captured goal, confirmed brief paths, authored input summary, and latest trace reference across assistant turns.
- Preserve the selected flow name when the user has committed to `bug-fix`, `change`, or `delivery`.
- Ask only for missing fields before recommending or executing a command.
- In chat-only mode, always provide exact copyable commands, wait for the user to run them, and update the workflow state only after pasted output.
- Preserve `inspection_target` when the user is working from an explicit trace instead of the latest workspace trace.
- When CLI output includes `next_command`, prefer that route instead of inventing a follow-up.
- When CLI output includes `corrected_command`, reuse it instead of inventing a replacement inspect invocation.
- When governance output reports `awaiting_approval` or `blocked`, do not suggest an ungoverned bypass; prefer `status` or `inspect` exactly as the CLI recommends and surface `governance_next_action` when the CLI provides it.
