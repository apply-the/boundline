# Boundline Init and Update

`boundline init` creates the workspace-local Boundline surface for a repository.
`boundline update` refreshes managed scaffold artifacts that were created by
init, or previews what would change before writing.

## Init

Run init from the target workspace, or pass `--workspace <path>` when targeting
another repository:

```bash
boundline init --assistant copilot
```

Init can write:

- `.boundline/config.toml` for workspace-local routing and optional Canon
  preferences.
- `.boundline/execution.json` when a compatibility execution template is
  selected.
- Repo-local assistant packs selected with `--assistant <host>`.
- Optional docs export under `docs/boundline/` with `--export-docs`.
- Optional IDE setup selected with `--ide <ide>`.
- Merge-only workspace hygiene defaults such as ignore-file entries.

Assistant packs and IDE setup are independent. Use `--assistant` when you want
repo-local chat command surfaces, and use `--ide` when you want editor or IDE
setup guidance.

## IDE Setup

IDE setup is opt-in. Boundline does not write IDE files unless at least one
`--ide` flag is provided.

| IDE | Managed Output | Terminal Auto-Approval |
| --- | --- | --- |
| VS Code | `.vscode/settings.json` | Managed `chat.tools.terminal.autoApprove` entries. |
| Cursor | `.cursor/rules/boundline.md` | Guidance only; no claimed stable JSON schema. |
| Antigravity | `.boundline/ide/antigravity.md` | Guidance only; configure terminal execution policy manually. |
| JetBrains | `.boundline/ide/jetbrains.md` | Guidance only; no stable project-scoped approval schema is generated. |

VS Code supports three auto-approval profiles:

- `read-only` approves low-risk Boundline inspection commands and denies known
  mutating Boundline flows.
- `session-safe` approves the session-native workflow commands `goal`, `plan`,
  and `run`, plus inspect/status-style commands, while denying administrative or
  higher-impact flows such as `init`, `orchestrate`, `workflow run|resume`,
  `config set|unset|bind-context|unbind-context`, and `cluster init`.
- `trusted` broadly approves `boundline` and `canon` terminal commands.

Examples:

```bash
boundline init --assistant copilot --ide vscode --auto-approve read-only
boundline init --assistant copilot --ide vscode --auto-approve session-safe
boundline init --assistant copilot --ide vscode --auto-approve trusted
boundline init --assistant antigravity --ide antigravity
boundline init --ide cursor --ide jetbrains
```

`--auto-approve` requires at least one `--ide`. For non-VS Code IDEs,
Boundline records managed guidance instead of inventing unverified terminal
policy settings.

## Docs Export

Use `--export-docs` when you want generated reference docs in the repository:

```bash
boundline init --assistant copilot --export-docs
```

Docs export is create-only by default. Use `--refresh` to update generated docs
in place, `--diff` to preview changes, `--to <path>` to export elsewhere, or
`--force` when an explicit overwrite is intended.

## Update

`boundline update` reads `.boundline/scaffold-manifest.json` and previews
managed scaffold refreshes:

```bash
boundline update --target ide
```

Apply the preview explicitly:

```bash
boundline update --target ide --apply
boundline update --target assistant --target ide --apply
```

When IDE setup was previously initialized, `boundline update --target ide`
refreshes the same IDE selections from the scaffold manifest. You can also pass
new IDE flags with the IDE target:

```bash
boundline update --target ide --ide vscode --auto-approve trusted
```

If no `--target` is provided, update refreshes the default managed surfaces and
includes IDE setup when the scaffold manifest already tracks IDE artifacts.
