# Boundline Copilot Prompt Pack

This folder describes Boundline support for Copilot-style prompt environments. It does not claim a universal Copilot plugin format.

Use `assistant/prompts/copilot-command-pack.md` and the prompt files under `assistant/copilot/prompts/` to guide Copilot into Boundline's real CLI/runtime. Copilot narration is advisory; the Rust CLI/runtime still performs orchestration, planning, execution, and state transitions. When you scaffold Copilot support with `boundline init --assistant copilot`, Boundline writes this prompt-pack metadata and mirrors the generated prompt files into `.github/prompts/` for VS Code prompt discovery. `assistant/copilot/prompts/` remains the Boundline-owned source copy, and you can still reference the generated prompt files directly via `#file`. `.boundline/session.json` remains authoritative for session state, and CLI output remains authoritative for `next_command`, blocked, clarification-required, failed, exhausted, and terminal states.

Canon governance is conditional. Do not expose `/boundline:govern` as the normal delivery path unless the workspace is configured for Canon governance.
