# Contract: Capture and Run CLI

## Purpose

Define the human-facing command contract for starting Synod work from direct text, one or more Markdown briefs, and optional governance intent without requiring users to author internal manifests.

## Command Surface

### `synod capture`

```text
synod capture \
  --workspace <path> \
  [--goal <text>] \
  [--brief <path>]... \
  [--governance <local|canon>] \
  [--risk <value>] \
  [--zone <value>] \
  [--owner <value>]
```

- At least one of `--goal` or `--brief` is required.
- `--brief` may be repeated to preserve explicit source order.
- Governance flags are optional and express business intent, not stage wiring.
- On success, Synod persists the normalized human input into the active session and returns the next recommended command.

### `synod run`

```text
synod run \
  --workspace <path> \
  [--goal <text>] \
  [--brief <path>]... \
  [--governance <local|canon>] \
  [--risk <value>] \
  [--zone <value>] \
  [--owner <value>]
```

- With no human-input flags, `run` preserves the current behavior and resumes the active planned task.
- With any human-input flag present, `run` must use the same normalization path as `capture` before planning or continuing execution.
- If the accepted input is planning-ready, `run` may proceed through the normal plan and execution flow.
- If the accepted input requires clarification, `run` must stop explicitly and return the clarification state instead of guessing.

## Resolution Rules

- Explicit `--brief` inputs are resolved first in the order they are supplied.
- Markdown paths mentioned in `--goal` text are resolved second in first-mention order.
- Only `.md` and `.markdown` files within the active workspace boundary are accepted in the first slice.
- Repeated file inputs are deduplicated by canonical workspace-relative path while preserving the earliest accepted precedence.
- Direct text always remains visible as its own input source even when it references workspace documents.

## Validation Rules

- `capture` fails with invalid invocation if neither `--goal` nor `--brief` is supplied.
- A brief path that is missing, unreadable, outside the workspace boundary, or not Markdown fails the command before planning begins.
- Governance business fields must never require the user to supply stage IDs, Canon modes, packet references, or manifest keys.
- If governance is requested but required business fields are missing, Synod raises a targeted clarification instead of inventing defaults silently.
- If the accepted input is too broad or conflicting to produce a bounded task, Synod records one explicit clarification and does not continue to planning.

## Compatibility Rules

- Existing goal-only usage remains valid for both `capture` and `run`.
- Existing manifest-driven automation remains valid and is not replaced by this contract.
- Assistant command packs and chat-driven entry points must map their inline text and selected files into this same normalization contract rather than creating a second human-input schema.