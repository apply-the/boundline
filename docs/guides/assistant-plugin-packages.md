# Assistant Plugin Packages

Boundline includes repository-local package surfaces for assistant hosts that can consume plugin metadata, command bindings, or prompt packs. These packages present Boundline as a local delivery orchestrator for bounded engineering work: plan, act, verify, trace.

`.boundline/session.json remains authoritative` for active session state. Host chat history is never the source of truth for session state, current step, recovery status, or next action. Use CLI output, especially `next_command`, `corrected_command`, trace references, and checkpoint restore commands, before recommending follow-up work.

Canon governance is conditional. It should appear only when the workspace is configured for governed delivery or when a user explicitly asks to inspect governance setup.

## Supported Host Packages

| Host | Package Folder | Contents | Install Shape |
|------|----------------|----------|---------------|
| Claude Code | `.claude-plugin/` | `manifest.json` plus command bindings | Copy or link the folder into the package location expected by Claude Code. |
| Codex | `.codex-plugin/` | `plugin.json` with interface metadata, default prompts, capabilities, and paths | Use the folder as the Codex plugin package root for this repository. |
| Cursor | `.cursor-plugin/` | `manifest.json` plus command bindings | Copy or link the folder into the package location expected by Cursor. |
| Copilot | `.copilot-prompts/` and `assistant/prompts/copilot-command-pack.md` | Prompt-pack metadata and prompt guidance | Use the prompt pack directly; this repository does not claim a universal Copilot plugin format. |

## Shared Boundline Sources

Host packages reference shared Boundline-owned files instead of copying behavior into every package:

- `assistant/plugin-metadata.json`
- `assistant/commands/session-workflow.json`
- `assistant/prompts/starter-prompts.md`
- `assistant/prompts/copilot-command-pack.md`
- `assistant/claude/commands/`
- `assistant/codex/commands/`
- `assistant/copilot/prompts/`

## Required Commands

Every supported package exposes or documents these namespaced commands:

| Chat Command | Runtime Surface | Notes |
|--------------|-----------------|-------|
| `/boundline:start` | `boundline start --workspace <workspace> --json` | Opens or resets the active session. |
| `/boundline:capture` | `boundline capture --workspace <workspace> --goal ... --json` | Persists goal or brief input into the active session. |
| `/boundline:plan` | `boundline plan --workspace <workspace> --json` | Produces or reports a bounded plan proposal; confirmation remains explicit. |
| `/boundline:run` | `boundline run --workspace <workspace> --json` | Runs the next bounded action through the real runtime. |
| `/boundline:status` | `boundline status --workspace <workspace> --json` | Reports current state and `next_command`. |
| `/boundline:inspect` | `boundline inspect --workspace <workspace> --json` | Reads authoritative trace and session evidence. |
| `/boundline:recover` | `boundline status --workspace <workspace> --json` then CLI-reported recovery command | Starts from runtime state and uses `next_command`, `corrected_command`, or checkpoint restore guidance. |
| `/boundline:govern` | `boundline config show --workspace <workspace> --scope workspace --json` | Conditional Canon governance only. |

## State Handling

Commands must surface blocked, clarification-required, failed, exhausted, and terminal states explicitly. They must not continue from chat-only assumptions. When shell access is unavailable, provide the exact CLI command with `--json`, wait for pasted output, and interpret that output rather than summarizing from memory.

## Validation

Run package validation from the repository root:

```bash
bash scripts/validate-assistant-plugins.sh
```

The validation checks JSON manifest syntax, required fields, Boundline version alignment, referenced paths, required command surfaces, unsupported capability claims, and prohibited package positioning.
