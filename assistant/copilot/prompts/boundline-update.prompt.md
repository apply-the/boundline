---
description: "Preview or apply Boundline-managed workspace upgrades"
---

# Command: /boundline-update

Shared guidance: `assistant/README.md`

## Install Boundary
Before relying on repo-local scaffold updates, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Preview or apply Boundline-managed workspace updates so the current project stays aligned with the latest Boundline scaffold, assistant assets, docs, or hygiene surfaces.

## Required Context
- `workspace_ref`
- Optional explicit `--target`, `--status`, `--diff`, `--apply`, `--force`, `--adopt`, `--prune`, or `--template` when the user names them

## Shell-Enabled Path
If `workspace_ref` is known, run `boundline update --workspace <workspace>` exactly once by default, preserving any explicit `--target`, `--status`, `--diff`, `--apply`, `--force`, `--adopt`, `--prune`, or `--template` flags the user requested. Add `--target assistant` when the user explicitly wants only the repo-local assistant package refreshed. Do not add `--apply`, `--force`, `--adopt`, or `--prune` unless the user asked to mutate the workspace or is explicitly following the CLI-reported repair path.

## Chat-Only Path
Ask only for missing `workspace_ref`, then provide one exact copyable command:

`boundline update --workspace <workspace>`

If the user explicitly wants only assistant-pack refreshes, provide:

`boundline update --workspace <workspace> --target assistant`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the update result. Do NOT dump raw CLI sections back verbatim when a concise summary is enough. Reply as a compact operator brief by default: preserve whether the run is preview-only or status-only, the reported `update_status` when present, `targets`, `manifest`, `tracked_artifacts`, summary totals, and the CLI-reported `next_steps`. Only expand `adoptions`, `updates`, `orphaned_artifacts`, `conflicts`, or detailed file-by-file changes when the user explicitly asks for more detail or the CLI reports a blocked repair path. Preserve `--apply`, `--force --apply`, `--adopt --force --apply`, and `--prune --apply` guidance exactly when the CLI emits them.

## Next-Step Routing
Prefer the CLI-reported `next_steps` or follow-up command. If update reports install or workspace health issues, route to the reported doctor or status path instead of inventing a new repair flow.
Allowed follow-up commands: `/boundline-update`, `/boundline-doctor`, `/boundline-config-show`, `/boundline-status`.