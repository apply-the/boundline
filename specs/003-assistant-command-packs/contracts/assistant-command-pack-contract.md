# Contract: Assistant Command Pack Surface

## Purpose

Defines the repository layout and required coverage for the assistant-native command packs shipped with Boundline.

## Supported Packs

| Assistant | Root | Asset Surface | File Pattern |
|-----------|------|---------------|--------------|
| Claude | `assistant/claude/commands/` | Slash-style command files | `boundline-<command>.md` |
| Codex | `assistant/codex/commands/` | Slash-style command files | `boundline-<command>.md` |
| Copilot | `assistant/copilot/prompts/` | Prompt files | `boundline-<command>.prompt.md` |

## Required Command Coverage

Every assistant pack MUST include exactly one file for each of the following commands:

- `boundline-start`
- `boundline-plan`
- `boundline-step`
- `boundline-run`
- `boundline-status`
- `boundline-next`
- `boundline-inspect`

## Documentation Requirements

- The repository MUST include `assistant/README.md`.
- `assistant/README.md` MUST describe supported assistants, installation or registration steps, shell-enabled behavior, and chat-only fallback behavior.
- Each assistant pack MUST point back to the shared README for environment-specific enablement guidance.

## Cross-Pack Consistency Guarantees

- All three packs MUST expose the same command names and the same user-level intent for each command.
- Differences between packs MAY exist only where required by assistant-specific packaging or prompt syntax.
- No pack may reference a backend command or workflow that is absent from the shared command definition contract.

## Behavioral Guarantees

- A pack file must never assume hidden shell access; both `shell-enabled` and `chat-only` paths must be documented.
- A pack file must never require external services or assistant-specific APIs to perform its core workflow.
- A pack file must prefer explicit workspace or trace context and must not invent run state that has not been confirmed by the user or the CLI backend.