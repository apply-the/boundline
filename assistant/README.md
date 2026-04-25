# Assistant Command Packs

This directory contains Markdown-based commands to run `synod` from various AI assistants (Claude, Codex, Copilot).

## Directory Structure
- **Claude**: `claude/commands/`
- **Codex**: `codex/commands/`
- **Copilot**: `copilot/prompts/`

## Installation & Registration
Each AI assistant has its own local or remote configuration. Currently, all command packs must be registered as local file references.

- **Copilot**: Copy `./assistant/copilot/prompts/*.prompt.md` to `.github/prompts/` or reference via `#file`.
- **Claude**: Load the respective `.md` files as projects or upload as attachments to the context window.
- **Codex**: Import into the corresponding workbench.

## Fallback Conventions
Since an assistant may be executed in a context *without* shell access (e.g., standard chat window), each command must gracefully degrade.

If the shell/terminal is *not* available:
1. Provide the user with the correct CLI command.
2. Provide a brief explanation of what the command does.
3. Invite the user to run it manually and paste the output.

If the shell/terminal *is* available:
1. Run the mapped CLI command directly from the repository root with `cargo run --bin synod -- ...`.
2. Do not explain syntax.
3. Observe the output and follow the next logical step silently until user interaction is required.

## Workflows

### Starting a Workflow (User Story 1)
- `/synod-start`: Confirms the workspace and runs `cargo run --bin synod -- doctor --workspace <workspace>` to summarize readiness and missing prerequisites.
- `/synod-plan`: Clarifies a bounded goal and routes to `/synod-run`; no direct CLI invocation is required.

### Continuing a Workflow (User Story 2)
- `/synod-step`: Uses confirmed context or pasted inspection output to recommend one explicit next command without inventing hidden state.
- `/synod-run`: Executes `cargo run --bin synod -- run --workspace <workspace> --goal "<goal>"` and summarizes `terminal_status`, `terminal_reason`, `trace`, and `next_command`.
- `/synod-status`: Executes `cargo run --bin synod -- inspect --workspace <workspace>` and summarizes the latest trace for the current workspace.
- `/synod-next`: Uses the latest inspection evidence to recommend the single most useful next command; when evidence is missing it first routes through `cargo run --bin synod -- inspect --workspace <workspace>`.

### Inspecting Prior Runs (User Story 3)
- `/synod-inspect`: Executes `cargo run --bin synod -- inspect --trace <trace>` for an explicit trace or `cargo run --bin synod -- inspect --workspace <workspace>` for the latest trace in a workspace.
- Successful inspection summaries must expose `inspection_target`, `trace`, `terminal_status`, `terminal_reason`, and `next_command` so assistants can continue routing without dumping raw logs.
- Trace-read failures must expose `terminal_reason`, `next_command: /synod-inspect`, and a `corrected_command` that tells the user how to retry with a corrected trace reference or workspace.

## Continuity Rules
- Preserve confirmed `workspace_ref`, goal, and latest trace reference across assistant turns.
- Ask only for missing fields before recommending or executing a command.
- In chat-only mode, wait for pasted output before updating the workflow state.
- Preserve `inspection_target` when the user is working from an explicit trace instead of the latest workspace trace.
- When CLI output includes `next_command`, prefer that route instead of inventing a follow-up.
- When CLI output includes `corrected_command`, reuse it instead of inventing a replacement inspect invocation.
