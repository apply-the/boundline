# Contract: Guided Init CLI Surfaces

## Purpose

Define the operator-facing contract for `boundline --version` and the guided and
non-interactive `boundline init` surfaces in feature 046.

## Top-Level Version Surface

The CLI MUST support both of these invocations:

- `boundline --version`
- `boundline -V`

Rules:

- The command MUST print the current Boundline version.
- The command MUST exit successfully without requiring a subcommand.
- The command MUST not print unrelated bootstrap guidance.

## Guided Init Surface

The default human bootstrap path is:

```bash
boundline init
```

Guided init MUST provide bounded terminal interaction for these decisions:

1. Canon approval mode
2. Assistant surface selection
3. Route review and per-slot route editing
4. Final summary confirmation

### Guided Interaction Rules

- Canon approval mode MUST be a closed selection surface with one visible default.
- Assistant surfaces MUST be a multi-select surface rather than comma-separated text entry.
- Route management MUST expose the current route table and allow the operator to:
  - accept defaults
  - edit one slot
  - clear all routes
- Slot editing MUST separate runtime choice from model choice.
- Bundled catalog source metadata MUST be visible before final confirmation.
- Custom model identifiers MUST remain allowed and MUST be labeled unverified.
- All four route slots (`planning`, `implementation`, `verification`, `review`) MUST be visible during route review, even when some are unset.
- Validation failures MUST keep the operator in the current step.
- Canceling before final confirmation MUST write nothing.
- When guided interaction is unavailable because a usable TTY is missing, the command MUST fail with guidance to use `--non-interactive` unless all required automation inputs were already provided.
- The no-TTY guidance text SHOULD use this message shape: `Terminal interaction is unavailable. Rerun with --non-interactive and explicit flags.`

## Non-Interactive Init Surface

Automation MUST remain available through explicit flags:

```bash
boundline init \
  --non-interactive \
  --canon-mode-selection <manual|auto-confirm|auto> \
  --assistant <runtime> \
  --assistant <runtime> \
  --route <slot=runtime:model>
```

Rules:

- `--non-interactive` MUST suppress guided prompts.
- Missing required values in non-interactive mode MUST fail with explicit validation errors.
- Repeated `--assistant` and `--route` flags MUST remain supported.
- Non-interactive mode MUST use the same stored configuration model as guided init.
- Non-interactive output MUST not attempt to render guided prompt controls.

## Progress Feedback

- Long-running init steps MUST expose progress feedback.
- Interactive terminals MAY use a spinner or equivalent single-line activity indicator.
- Non-interactive output MUST use stable text lines without spinner frame artifacts.
- Progress feedback MUST terminate cleanly on success, failure, or cancellation.

## Summary Contract

Before writes occur, guided init MUST display a summary that includes:

- Canon approval mode
- Selected assistant surfaces
- Route table with slot names
- Catalog source metadata
- Warnings for custom or unset routes
- Planned file or asset changes

The summary MUST offer an explicit confirm/cancel decision.

## Assistant Asset Scaffolding

- Selected assistant surfaces MUST scaffold or refresh the existing repository-managed assets under `assistant/` into the target workspace using the same relative paths.
- Summary or completion output MUST report assistant asset results grouped by surface using this shape: `surface: <created> created, <updated> updated, <unchanged> unchanged`.
- The bundled catalog asset itself is read-only input to init and MUST NOT be copied into the workspace.

## Failure Semantics

- Invalid guided selections MUST not abort the entire session; they must return the operator to the active step.
- Write failures MUST end the command with explicit error text and no hidden retries.
- Preview mode caused by existing files and missing `--force` MUST remain explicit and must still surface planned changes and next steps.
