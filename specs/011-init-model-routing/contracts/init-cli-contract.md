# Contract: Init CLI

## Purpose

Define the user-facing contract for bootstrapping a Boundline workspace without
hand-authoring internal JSON and while optionally preparing assistant runtime
support for the repository.

## Command Surface

### `boundline init`

```text
boundline init \
  --workspace <path> \
  [--template <bug-fix|change|delivery>] \
  [--assistant <claude|codex|copilot|gemini>]... \
  [--route <slot>=<runtime>:<model>]... \
  [--reviewer-route <role>=<runtime>:<model>]... \
  [--adjudicator-route <runtime>:<model>] \
  [--yes] \
  [--force]
```

- `--workspace` is required.
- When template or routing flags are omitted, `boundline init` may guide the user
  interactively or apply documented defaults, but it must still preview the
  resulting changes before writing files.
- `--assistant` may be repeated to enable one or more supported assistant
  surfaces for repository-local setup.
- `--route` accepts user-facing delivery slots such as `planning`,
  `implementation`, `verification`, or `review`.
- `--reviewer-route` applies to one named review role.
- `--adjudicator-route` applies only to the adjudicator slot.
- `--yes` confirms non-destructive defaults and previews without another prompt.
- `--force` is required for destructive replacement after the preview indicates
  an overwrite.

## Required Behavior

- Init MUST create or update the bounded workspace execution files needed by the
  normal Boundline workflow.
- Init MUST detect supported runtime capability before applying routing choices.
- Init MUST preview all proposed file mutations and identify destructive changes.
- Init MUST offer repository-local assistant setup only for supported runtimes
  the user selected.
- Init MUST never write outside the active workspace root.

## Validation Rules

- Unknown template kinds are invalid invocation errors.
- Unknown runtime identifiers are invalid invocation errors.
- Invalid route slots or malformed `runtime:model` pairs are invalid invocation
  errors.
- Gemini routes are valid only as CLI-backed runtime choices in this slice.
- If a selected runtime is unavailable, init must stop or skip that route with a
  clear operator-facing explanation; it must not silently rewrite the route to a
  different runtime.
- If init would overwrite existing config or assistant files, it must require an
  explicit destructive confirmation.

## Compatibility Rules

- Repositories that already use manual `.boundline/execution.json` remain valid.
- Init must be rerunnable so repositories can adopt new config or assistant
  setup later without deleting their working bounded execution profile.
- Init output must name the next useful command, such as `boundline doctor`,
  `boundline config show`, or `boundline start`.