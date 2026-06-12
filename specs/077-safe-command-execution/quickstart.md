# Quickstart: Safe Command Execution and Evidence Capture

## Overview

`boundline exec` wraps any shell command with intent classification, policy enforcement, evidence capture, and secret redaction.

## Prerequisites

- Boundline workspace initialized (`boundline init`)
- No Docker required

## Basic Usage

```bash
# Execute a safe command — captured with evidence
boundline exec "echo hello world"

# Dry-run a potentially destructive command
boundline exec --dry-run "rm -rf ./build"

# Execute a command in no-mutation mode
boundline exec --no-mutation "npm install"

# Check what classification a command would get
boundline exec --classify-only "rm -rf ./data"
# Output: intent=mutate, mode=require-approval (default green zone policy)
```

## Configuration Files

### `.boundline/execution-policy.toml`

Defines the Intent × Zone matrix. Defaults deny unknown intents and require approval for mutate/install/deploy in green zones.

```toml
[defaults]
unknown_intent_mode = "require-approval"
missing_policy_mode = "deny"

[policy.read.green]
mode = "allow"

[policy.mutate.green]
mode = "require-approval"

[policy.deploy.red]
mode = "deny"
```

### `.boundline/redaction.toml`

Configures secret patterns for output redaction. Built-in defaults cover GitHub tokens, AWS keys, and JWT.

```toml
[[patterns]]
id = "custom-api-key"
kind = "api_key"
regex = '''sk-[a-zA-Z0-9]{32,}'''
severity = "high"
replacement = "[REDACTED:api_key]"
```

### `.boundline/evidence-limits.toml`

Overrides default size/time limits.

```toml
[limits]
stdout_max_bytes = 2_000_000    # Default: 1MB
mutation_max_entries = 20_000   # Default: 10K
```

## Evidence Output

After each command, evidence is persisted to `.boundline/traces/<trace-id>.json`:

```json
{
  "trace_id": "20260611T103000Z-a1b2c3d4e5f6",
  "command": "echo hello",
  "intent": "read",
  "execution_mode": "allow",
  "exit_code": 0,
  "stdout": "hello\n",
  "stdout_truncated": false,
  "artifact_manifest": { "files": [], "total_files_produced": 0 },
  "mutation_boundary": { "created": [], "modified": [], "deleted": [], "complete": true }
}
```

## Governance Hooks

Configure hooks in `.boundline/config.toml`:

```toml
[governance.hooks.deploy-gate]
trigger_intents = ["deploy"]
trigger_zones = ["red"]
action = "require-approval"
```
