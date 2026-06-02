# Quickstart: Native Canon CLI Surface

**Feature**: 042-native-canon-cli

## Prerequisites

- Boundline CLI installed (`boundline --version`)
- Canon CLI installed and on PATH (`canon --version` reports 0.40.0+)
- A workspace directory (new or existing repository)

## 1. Verify Installation

```bash
boundline doctor --install
```

Confirm all checks pass, including `canon_governance_surface` and `canon_modes`.
If any check fails, follow the `suggested_actions` in the output.

## 2. Initialize Workspace

**Guided** (interactive):
```bash
cd /path/to/workspace
boundline init
```

Follow prompts to:
- Choose Canon mode-selection preference (`manual`, `auto-confirm`, or `auto`)
- Select assistant surfaces (Copilot, Codex, Claude, Gemini)
- Configure model routes for planning, implementation, verification, and review

**Scripted** (non-interactive):
```bash
boundline init \
  --template delivery \
  --canon-mode-selection auto-confirm \
  --risk medium \
  --zone engineering \
  --owner platform \
  --assistant copilot --assistant codex \
  --route planning=copilot:gpt-4o \
  --route implementation=codex:codex-1
```

## 3. Run Governed Work

**From a goal**:
```bash
boundline run --goal "Add OAuth 2.0 authentication to the API"
```

**From a goal plus authored inputs**:
```bash
boundline run \
  --goal "Add OAuth 2.0 authentication" \
  --brief docs/prd.md \
  --brief tech-docs/architecture.md
```

**With explicit Canon mode**:
```bash
boundline run --mode requirements --brief docs/prd.md
```

**With local governance opt-out**:
```bash
boundline run --no-canon --goal "Quick fix for test flake"
```

## 4. Check Status

```bash
boundline status
```

View the governed lifecycle, current mode, approval state, and next safe action.

## 5. Continue or Inspect

```bash
boundline next     # See the next available action
boundline inspect  # Read persisted traces and governed output
boundline run      # Continue execution (refreshes approval if pending)
```

## 6. Change Canon Settings Later

```bash
# View current config
boundline config show

# Switch mode-selection preference
boundline config set-canon --mode-selection auto

# Update model routing
boundline config set --slot planning --runtime copilot --model gpt-4o
```

## Using Assistant Commands

All operations are available through chat commands:

```text
/boundline-init              # Guided workspace setup
/boundline-run               # Default Canon-governed run
/boundline-requirements      # Run requirements mode
/boundline-architecture      # Run architecture mode
/boundline-implementation    # Run implementation mode
/boundline-status            # Check governed lifecycle
/boundline-next              # Next safe action
/boundline-inspect           # Read traces
/boundline-config-show       # View workspace settings
/boundline-config-set-canon  # Change mode-selection preference
```
