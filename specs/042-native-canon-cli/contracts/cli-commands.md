# CLI Commands Contract: Native Canon CLI Surface

**Feature**: 042-native-canon-cli
**Date**: 2026-05-05

## Overview

This contract defines the CLI commands and flags introduced or modified by
feature 042.  All commands emit JSON to stdout when invoked programmatically
and human-readable text by default.  Assistant command packs map to the same
commands.

---

## Modified Commands

### `boundline init`

**Change**: Add guided Canon mode-selection, assistant/model routing collection,
and Canon surface verification during workspace initialization.

```text
boundline init [--workspace <path>]
    [--canon-mode-selection <manual|auto-confirm|auto>]
    [--assistant <name>...]
    [--route <slot>=<runtime>:<model>...]
    [--risk <risk>]
    [--zone <zone>]
    [--owner <owner>]
```

**Guided mode** (no `--canon-mode-selection` flag):
1. Ask Canon mode-selection preference (`manual`, `auto-confirm`, `auto`)
2. Ask assistant surfaces to configure (multi-select from Copilot, Codex, Claude, Gemini)
3. Ask model routes for available routing slots
4. Verify Canon surface if Canon binary detected
5. Write `.boundline/config.toml` with `[canon]` and `[routing]` sections

**Non-interactive mode** (flags provided):
- Apply settings directly without prompts
- Fail with specific error if conflicting or invalid values

**Output** (JSON):
```json
{
  "workspace": "/path/to/workspace",
  "config_path": ".boundline/config.toml",
  "canon_preferences": {
    "mode_selection": "auto-confirm",
    "default_risk": "standard",
    "default_zone": "development",
    "default_owner": "operator"
  },
  "canon_surface": {
    "ready": true,
    "canon_path": "/usr/local/bin/canon",
    "version": "0.40.0",
    "modes_verified": true,
    "operations_verified": true
  },
  "routing": {
    "assistant_runtimes": ["Copilot", "Codex"],
    "planning": { "runtime": "Copilot", "model": "gpt-4o" }
  }
}
```

---

### `boundline run`

**Change**: Default to Canon governance when workspace is Canon-ready.  Add
`--mode`, `--no-canon`, and governance field flags.

```text
boundline run [--workspace <path>]
    [--goal "<goal>"]
    [--brief <path>...]
    [--mode <canon-mode>]
    [--governance <canon|local>]
    [--no-canon]
    [--risk <risk>]
    [--zone <zone>]
    [--owner <owner>]
```

**Resolution order**:
1. Resolve workspace (`.boundline/` parent → git root → CWD)
2. Load workspace config and Canon preferences
3. If `--no-canon` or `--governance local` → Local governance
4. If Canon preferences present and Canon surface verified → Canon governance
5. Otherwise → Local governance (backward compatible)

**Mode selection**:
- `--mode <canon-mode>` → use that mode regardless of preference
- No `--mode` + `manual` preference → error: "Canon mode-selection is manual;
  specify --mode <mode>"
- No `--mode` + `auto-confirm` → infer mode, prompt for confirmation
- No `--mode` + `auto` → infer mode, proceed if confident

**Output** (JSON, extends existing `run` output):
```json
{
  "routing": {
    "execution_path": "session-native",
    "governance_runtime": "canon",
    "explicit_opt_out": false,
    "mode_selection_preference": "auto-confirm",
    "selected_mode": "requirements",
    "canon_surface_ready": true
  },
  "governance": {
    "lifecycle_state": "governed_ready",
    "approval_state": "not_needed",
    "packet_ref": "pkt-abc123",
    "governed_document": ".canon/runs/run-xyz/requirements.md",
    "next_action": "Advance to next stage or inspect governed output"
  },
  "session": { "session_id": "...", "status": "Running" },
  "trace": { "trace_ref": "..." }
}
```

---

### `boundline doctor --install`

**Change**: Verify Canon governance surface (operations + modes), not just
version.

```text
boundline doctor [--workspace <path>] [--install]
```

**Enhanced Canon checks**:
1. `canon_binary`: Binary exists on PATH or bundled
2. `canon_version`: Version matches supported window
3. `canon_governance_surface`: `governance start` and `governance refresh`
   operations available
4. `canon_modes`: All 15 canonical modes present in capabilities response
5. `canon_path`: Report authoritative binary path

**Output** (JSON, extends existing diagnostics):
```json
{
  "subject": "Install",
  "checks": [
    { "name": "canon_binary", "status": "pass", "message": "Canon found at /usr/local/bin/canon" },
    { "name": "canon_version", "status": "pass", "message": "Version 0.40.0 matches supported window" },
    { "name": "canon_governance_surface", "status": "pass", "message": "governance start and governance refresh available" },
    { "name": "canon_modes", "status": "warn", "message": "14/15 modes verified; missing: supply-chain-analysis" }
  ],
  "ready": false,
  "canon_surface_verification": {
    "canon_path": "/usr/local/bin/canon",
    "version_compatible": true,
    "operations_verified": true,
    "modes_verified": false,
    "missing_modes": ["supply-chain-analysis"],
    "unsupported_modes": [],
    "repair_actions": ["Upgrade Canon to version 0.40.0 or later for supply-chain-analysis support"]
  }
}
```

---

### `boundline status` / `boundline next` / `boundline inspect`

**Change**: Project governance lifecycle state, selected Canon mode,
mode-selection preference, approval/blocked state, governed artifact references,
local opt-out state, and next safe action.

**Additional output fields** (JSON):
```json
{
  "governance": {
    "runtime": "canon",
    "explicit_opt_out": false,
    "mode_selection_preference": "auto-confirm",
    "current_mode": "requirements",
    "mode_sequence": ["requirements", "architecture", "implementation"],
    "current_stage_index": 0,
    "lifecycle_state": "awaiting_approval",
    "approval_state": "requested",
    "blocked_reason": null,
    "governed_artifacts": [
      { "stage": "delivery:requirements", "mode": "requirements", "packet_ref": "pkt-abc123", "readiness": "reusable" }
    ],
    "next_action": "Wait for approval or run 'boundline run' to refresh approval state"
  }
}
```

---

## New Commands

### `boundline config set-canon`

```text
boundline config set-canon [--workspace <path>]
    --mode-selection <manual|auto-confirm|auto>
```

**Behavior**: Update Canon mode-selection preference in workspace-local
`.boundline/config.toml`.  Subsequent runs use the new value.

**Output** (JSON):
```json
{
  "workspace": "/path/to/workspace",
  "canon_preferences": {
    "mode_selection": "auto",
    "previous_mode_selection": "manual"
  },
  "config_path": ".boundline/config.toml"
}
```

---

## Workspace Resolution Contract

All commands accepting `--workspace` follow this resolution order:

1. **Explicit**: Use `--workspace <path>` when supplied
2. **`.boundline/` parent**: Search upward from CWD for an existing
   `.boundline/` directory; use its parent
3. **Git root**: Search upward for nearest `.git/` directory; use its parent
4. **CWD**: Fall back to the current working directory

Before mutating files, the resolved workspace is surfaced in output.  If the
target is ambiguous (e.g., multiple `.boundline/` candidates) or outside the
active repository, the command stops with an error.

---

## Assistant Command Mapping

| Chat Command | Maps To | Notes |
|---|---|---|
| `/boundline-doctor` | `boundline doctor --install` | |
| `/boundline-init` | `boundline init` (guided or scripted) | Collect answers first |
| `/boundline-config-show` | `boundline config show` | |
| `/boundline-config-set-canon` | `boundline config set-canon` | |
| `/boundline-config-set` | `boundline config set` | |
| `/boundline-capture` | `boundline capture` | |
| `/boundline-plan` | `boundline plan` | |
| `/boundline-run` | `boundline run` | Default Canon path |
| `/boundline-run <mode>` | `boundline run --mode <mode>` | |
| `/boundline-requirements` | `boundline run --mode requirements` | Alias |
| `/boundline-discovery` | `boundline run --mode discovery` | Alias |
| `/boundline-system-shaping` | `boundline run --mode system-shaping` | Alias |
| `/boundline-architecture` | `boundline run --mode architecture` | Alias |
| `/boundline-backlog` | `boundline run --mode backlog` | Alias |
| `/boundline-change` | `boundline run --mode change` | Alias |
| `/boundline-implementation` | `boundline run --mode implementation` | Alias |
| `/boundline-refactor` | `boundline run --mode refactor` | Alias |
| `/boundline-review` | `boundline run --mode review` | Alias |
| `/boundline-verification` | `boundline run --mode verification` | Alias |
| `/boundline-incident` | `boundline run --mode incident` | Alias |
| `/boundline-security-assessment` | `boundline run --mode security-assessment` | Alias |
| `/boundline-system-assessment` | `boundline run --mode system-assessment` | Alias |
| `/boundline-migration` | `boundline run --mode migration` | Alias |
| `/boundline-supply-chain-analysis` | `boundline run --mode supply-chain-analysis` | Alias |
| `/boundline-status` | `boundline status` | |
| `/boundline-next` | `boundline next` | |
| `/boundline-inspect` | `boundline inspect` | |
