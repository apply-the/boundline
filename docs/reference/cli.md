# Reference

This page collects compact operator references. Use the task pages for guided workflows.

## Command Reference

### Install And Readiness

```bash
boundline doctor --install
boundline doctor --workspace <workspace>
```

### Workspace Setup

```bash
boundline init --workspace <workspace>
boundline init --workspace <workspace> --assistant codex
boundline assistant install --host codex --scope user
```

### Optional Preflight

```bash
boundline models auth login --provider github-copilot
boundline models auth status
boundline probe --workspace <workspace>
```

### Session Runtime

```bash
boundline goal --workspace <workspace> --goal "<goal>"
boundline plan --workspace <workspace>
boundline plan --workspace <workspace> --confirm
boundline run --workspace <workspace>
boundline status --workspace <workspace>
boundline next --workspace <workspace>
boundline inspect --workspace <workspace>
```

### Configuration

```bash
boundline config show --workspace <workspace> --scope effective
boundline config show --workspace <workspace> --scope workspace
boundline config set-canon --workspace <workspace> --mode-selection auto-confirm
boundline adapter add speckit --workspace <workspace>
boundline adapter show --workspace <workspace> --json
boundline adapter remove --workspace <workspace>
```

### Workflows

```bash
boundline workflow list --workspace <workspace>
boundline workflow status --workspace <workspace>
boundline workflow inspect --workspace <workspace>
```

### Recovery

```bash
boundline checkpoint list --workspace <workspace>
boundline checkpoint restore <id> --workspace <workspace>
```

Prefer the restore command reported by `status`, `next`, or `inspect`.

## File And Directory Reference

```text
.boundline/config.toml        workspace config
.boundline/session.json       active session state
.boundline/traces/            trace records
.boundline/checkpoints/       rollback manifests
.boundline/context-intelligence/ derived retrieval DB, manifest, and local snapshot cache state
.boundline/guidance/          workspace guidance overrides
.boundline/guardians/         workspace guardian overrides
.boundline/workflows.toml     optional workflow registry
.claude-plugin/               repo-local Claude package
.codex-plugin/                repo-local Codex package
.cursor-plugin/               repo-local Cursor package
.copilot-prompts/             repo-local Copilot prompt package
.github/prompts/              VS Code Copilot prompt discovery projection
assistant/packs/              bundled pack sources in the Boundline repo
```

## Manifest Reference

Assistant and pack manifests should be treated as package metadata, not as runtime state. Runtime state comes from `.boundline/session.json` and CLI output.

Common manifest surfaces:

- `.codex-plugin/plugin.json`
- `.claude-plugin/manifest.json`
- `.cursor-plugin/manifest.json`
- `.copilot-prompts/`
- `assistant/packs/*`
- `assistant/packs/guidance-catalog/pack.toml`
- `assistant/packs/guidance-catalog/catalog/guidance-index.toml`
- `assistant/packs/guidance-catalog/catalog/guardian-index.toml`

## Trace Reference

Important trace and status fields can include:

- `command_name`
- `exit_status`
- `rendered_output`
- `session_status`
- `trace_location`
- `trace_summary`
- `next_command`
- `corrected_command`
- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance`
- `repository_map_state`
- `snapshot_cache_state`
- `context_pack_entries`
- `omission_findings`
- `patch_safe_edit_attempts`
- `goal_plan_state`
- `planning_rationale`
- `verification_strategy`
- `route_owner`
- `route_config_projection`
- `framework_adapter_status`
- `framework_adapter_execution_source`
- `framework_adapter_supported_transports`
- `framework_adapter_config_state`
- `loaded_packs`
- `skipped_packs`
- `catalog_validation_findings`
- `guardian_timeline`
- `guardian_findings_summary`
- `guardian_blocking_outcome`
- `latest_checkpoint_restore_command`

## Glossary

- **Bounded session**: one declared delivery boundary with persisted runtime state.
- **Canon**: governed knowledge, packets, evidence, approvals, lineage, and project memory companion.
- **Context pack**: selected evidence used to plan or continue bounded work.
- **Expert pack**: reusable expertise for a stack, domain, role, or delivery phase.
- **Guidance**: rules that shape work before or during action.
- **Guardian**: checker that emits structured findings after action or at quality boundaries.
- **Finding**: structured validation or review result with evidence and disposition.
- **Trace**: record of runtime decisions, evidence, actions, checks, and next steps.
- **Route**: mapping from a delivery slot to an assistant runtime and model.
- **Checkpoint**: local rollback manifest for a mutating bounded run.

## Dashboard Commands

```bash
boundline-dashboard [--workspace <path>] [--no-color] [--snapshot-json]
boundline dashboard [--workspace <path>] [--no-color]
```

`boundline-dashboard` is the dedicated terminal dashboard. `--snapshot-json` emits the typed dashboard snapshot for validation. `boundline dashboard` is the normal CLI launcher and fallback surface.
