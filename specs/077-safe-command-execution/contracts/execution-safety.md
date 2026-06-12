# Execution Safety Contracts

## CLI Contract: `boundline exec`

### Synopsis

```
boundline exec [OPTIONS] <COMMAND>
```

### Options

| Flag | Description |
|------|-------------|
| `COMMAND` | The shell command to execute (required) |
| `--dry-run` | Execute via deterministic dry-run tier; do not apply mutations |
| `--no-mutation` | Execute but block filesystem writes |
| `--classify-only` | Classify the command and print intent + policy decision without executing |
| `--zone <ZONE>` | Override execution zone (green, yellow, red) |
| `--json` | Output evidence as JSON to stdout instead of persisting to traces/ |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Command executed successfully |
| 1 | Command failed (non-zero exit) |
| 2 | Policy denied execution |
| 3 | Approval required (blocked pending governance) |
| 4 | Invalid configuration |

### Output Contract

When `--json` is passed, stdout receives a complete EvidencePacket JSON. Otherwise, a human-readable summary is printed to stdout and the full packet is persisted to `.boundline/traces/`.

## Evidence Packet Schema

```json
{
  "$schema": "https://boundline.dev/schemas/evidence-packet-v1.json",
  "trace_id": "string",
  "command": "string",
  "intent": "read|test|mutate|install|deploy|unknown",
  "execution_mode": "allow|dry-run|no-mutation|require-approval|deny",
  "dry_run_status": "native_dry_run_executed|read_only_executed|plan_only|unsupported_for_safe_dry_run|null",
  "timing": {
    "started_at": "ISO8601",
    "finished_at": "ISO8601",
    "wall_clock_ms": "u64"
  },
  "exit_code": "i32|null",
  "stdout": "string",
  "stdout_truncated": "bool",
  "stderr": "string",
  "stderr_truncated": "bool",
  "artifact_manifest": {
    "files": [{"path": "string", "size_bytes": "u64", "modified_at": "ISO8601", "operation": "created|modified|deleted"}],
    "total_files_produced": "u32"
  },
  "mutation_boundary": {
    "created": ["string"],
    "modified": [{"path": "string", "pre_hash": "string", "post_hash": "string"}],
    "deleted": ["string"],
    "truncated": "bool",
    "complete": "bool"
  },
  "policy_decision": {
    "inferred_intent": "string",
    "zone": "green|yellow|red",
    "matched_policy_entry": "string",
    "matched_override": "string|null",
    "safety_escalations": ["string"],
    "final_mode": "string",
    "rationale": "string"
  },
  "redaction_audit": [
    {"pattern_id": "string", "match_count": "u32"}
  ],
  "timestamp": "ISO8601"
}
```

## Policy File Schema

### `.boundline/execution-policy.toml`

```toml
[defaults]
unknown_intent_mode = "require-approval"  # allow|dry-run|no-mutation|require-approval|deny
missing_policy_mode = "deny"

[policy.<intent>.<zone>]
mode = "allow"  # allow|dry-run|no-mutation|require-approval|deny

[[overrides]]
command = "string"          # Binary name
args_contains = ["string"]  # Optional: match if all args present
intent = "string"           # Optional: override intent
mode = "string"             # Optional: override mode
```

### `.boundline/redaction.toml`

```toml
[defaults]
enabled = true
replacement = "[REDACTED:{kind}]"

[[patterns]]
id = "string"
kind = "string"
regex = "string"            # Valid regex
severity = "high|medium|low"
replacement = "string"

[[allowlist]]
id = "string"
path_glob = "string"
regex = "string"
reason = "string"
```
