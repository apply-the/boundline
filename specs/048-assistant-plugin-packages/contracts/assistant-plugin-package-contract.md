# Contract: Assistant Plugin Packages

## Required Package Folders

- `.claude-plugin/`
- `.codex-plugin/`
- `.cursor-plugin/`
- `.copilot-prompts/` for prompt-pack metadata without claiming a universal Copilot plugin manifest

## Required Common Metadata

Each JSON package manifest or prompt-pack metadata file must expose:

- `name`
- `displayName`
- `version`
- `description`
- `author`
- `homepage`
- `repository`
- `license`
- `keywords`
- `capabilities`
- `paths`

The `version` value must equal the workspace package version in `Cargo.toml`.

## Required Commands

Every supported package surface must expose or document:

- `/boundline:start`
- `/boundline:goal`
- `/boundline:plan`
- `/boundline:run`
- `/boundline:status`
- `/boundline:inspect`
- `/boundline:recover`

`/boundline:govern` must exist only as a conditional Canon-governance integration. It must not be presented as the default Boundline path when governance is not configured.

## State Contract

Command bindings must preserve these rules:

- `.boundline/session.json` is authoritative for active session state.
- Chat history is never authoritative for session state or next action.
- Commands must call or guide the real Boundline CLI/runtime.
- Commands must surface blocked, clarification-required, failed, exhausted, and terminal states explicitly.
- Commands must prefer CLI-reported `next_command`, `corrected_command`, checkpoint restore command, or state-specific guidance over inferred chat follow-up.

## Positioning Contract

Approved positioning includes:

- "Local delivery orchestrator for bounded engineering work"
- "Plan, act, verify, trace"
- "Turns bounded engineering goals into verified workspace changes"
- "Session-native runtime for AI-assisted software delivery"

Prohibited positioning includes:

- generic agent framework
- debate CLI
- prompt library
- governance runtime
- replacement for Canon
- raw CLI wrapper

## Validation Contract

`bash scripts/validate-assistant-plugins.sh` must fail when:

- JSON manifests are invalid.
- Required metadata fields are missing.
- Referenced paths do not exist.
- Required commands are missing.
- Versions drift from the workspace package version.
- Unsupported host capability claims appear.
- Prohibited positioning appears in package metadata.
