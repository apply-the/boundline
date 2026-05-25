---
description: "Initialize a Boundline workspace with Canon-default preferences"
---

# Command: /boundline-init

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Initialize workspace-local or global Boundline configuration, Canon defaults, and assistant scaffolding without inventing setup state from chat history.

## Required Context
- `workspace_ref` for workspace or both scope
- Optional explicit scope, template, Canon mode-selection preference, assistant runtimes, or route overrides when the user names them

## Shell-Enabled Path
If the user wants workspace or both scope and `workspace_ref` is known, run `cargo run --bin boundline -- init --workspace <workspace> --json` exactly once, preserving any explicit `--scope`, `--template`, `--assistant`, `--route`, `--canon-mode-selection`, `--export-docs`, or `--force` flags they requested. If the user explicitly wants global-only setup, run `cargo run --bin boundline -- init --scope global --json` exactly once.

## Chat-Only Path
Ask only for the missing `workspace_ref` when workspace scope is required, then provide one exact copyable command:

`cargo run --bin boundline -- init --workspace <workspace> --json`

If the user explicitly wants global-only setup, provide:

`cargo run --bin boundline -- init --scope global --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Reply as a compact operator brief by default: preserve `scope`, `template` when present, a concise setup summary, key artifacts such as config and execution-profile paths, Canon readiness fields like `canon_mode_selection`, `canon_bootstrap`, and `canon_surface`, `latest_status`, and the CLI-reported `next_command`. Only surface full scaffold diffs, route-setup dumps, or detailed file-by-file assistant and hygiene listings when the user explicitly asks for deeper detail or wants the CLI `--verbose` view. Preserve blocked preview wording, Canon bootstrap failures, and repair guidance exactly when reported.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If init is blocked on installation or Canon readiness, route to the reported doctor or inspect command instead of inventing a new setup path.
Allowed follow-up commands: `/boundline-doctor`, `/boundline-config-show`, `/boundline-init`, `/boundline-status`.
