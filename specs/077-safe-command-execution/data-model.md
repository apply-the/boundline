# Data Model: Safe Command Execution and Evidence Capture

## Core Entities

### CommandIntent

Classification of a command's purpose. Determined by the classifier before execution.

| Variant | Description | Example Commands |
|---------|-------------|-----------------|
| `read` | Inspects state without mutation | `ls`, `cat`, `grep`, `git status`, `git diff` |
| `test` | Runs tests or checks | `cargo test`, `go test`, `npm test`, `pytest` |
| `mutate` | Changes local state | `rm`, `mv`, `sed -i`, `git commit`, `git push` |
| `install` | Installs dependencies or tools | `apt-get install`, `brew install`, `cargo install` |
| `deploy` | Deploys to external environments | `kubectl apply`, `terraform apply`, deploy scripts |
| `unknown` | Command not in whitelist | Default → treated as `require-approval` |

### ExecutionMode

Determines how a command is executed based on policy resolution.

| Mode | Behavior |
|------|----------|
| `allow` | Execute normally, capture evidence |
| `dry-run` | Execute via dry-run tier, do not apply mutations |
| `no-mutation` | Execute but block filesystem writes |
| `require-approval` | Block until operator or governance hook approves |
| `deny` | Block execution entirely |

### DryRunStatus

Result of a dry-run execution.

| Status | Meaning |
|--------|---------|
| `native_dry_run_executed` | Known command ran in its native safe mode |
| `read_only_executed` | Read-only command executed directly |
| `plan_only` | Plan emitted, command NOT executed |
| `unsupported_for_safe_dry_run` | Unknown command, NOT executed |

### EvidencePacket

```rust
struct EvidencePacket {
    trace_id: String,           // "{ISO8601_timestamp}-{sha256_hex12}"
    command: String,            // Full command string
    intent: CommandIntent,
    execution_mode: ExecutionMode,
    dry_run_status: Option<DryRunStatus>,
    timing: ExecutionTiming,
    exit_code: Option<i32>,
    stdout: String,             // Redacted, max 1MB
    stdout_truncated: bool,
    stderr: String,             // Redacted, max 1MB
    stderr_truncated: bool,
    artifact_manifest: ArtifactManifest,
    mutation_boundary: MutationBoundary,
    policy_decision: PolicyDecision,
    redaction_audit: Vec<RedactionRecord>,
    timestamp: OffsetDateTime,
}
```

### ExecutionTiming

```rust
struct ExecutionTiming {
    started_at: OffsetDateTime,
    finished_at: OffsetDateTime,
    wall_clock_ms: u64,
    cpu_time_ms: Option<u64>,
}
```

### ArtifactManifest

```rust
struct ArtifactManifest {
    files: Vec<ArtifactEntry>,
    total_files_produced: u32,
}

struct ArtifactEntry {
    path: String,               // Relative to workspace root
    size_bytes: u64,
    modified_at: OffsetDateTime,
    operation: FileOperation,   // Created, Modified, Deleted
}
```

### MutationBoundary

```rust
struct MutationBoundary {
    created: Vec<String>,       // File paths
    modified: Vec<ModifiedFile>,
    deleted: Vec<String>,
    truncated: bool,            // True if >10K entries
    total_observed: Option<u32>, // Actual count when truncated
    complete: bool,             // False on filesystem error
    error: Option<String>,
}

struct ModifiedFile {
    path: String,
    pre_hash: String,           // SHA-256 hex
    post_hash: String,
}
```

### PolicyDecision

Records the resolution path for a single command execution.

```rust
struct PolicyDecision {
    inferred_intent: CommandIntent,
    zone: RiskZone,             // green, yellow, red
    matched_policy_entry: String, // e.g., "policy.mutate.green"
    matched_override: Option<String>, // override ID if any
    safety_escalations: Vec<String>, // applied flags: --force, --delete, etc.
    final_mode: ExecutionMode,
    rationale: String,
}
```

### SecretPattern (from `.boundline/redaction.toml`)

```rust
struct SecretPattern {
    id: String,
    kind: String,               // e.g., "github_token", "aws_access_key"
    regex: String,              // Compiled regex
    severity: Severity,         // high, medium, low
    replacement: String,        // e.g., "[REDACTED:github_token]"
}

struct AllowlistRule {
    id: String,
    path_glob: String,          // e.g., "docs/**"
    regex: String,              // Pattern to allow
    reason: String,
}
```

### ExecutionPolicy (from `.boundline/execution-policy.toml`)

```rust
struct ExecutionPolicy {
    defaults: PolicyDefaults,
    policy: HashMap<CommandIntent, HashMap<RiskZone, PolicyEntry>>,
    overrides: Vec<CommandOverride>,
}

struct PolicyDefaults {
    unknown_intent_mode: ExecutionMode, // default: require-approval
    missing_policy_mode: ExecutionMode, // default: deny
}

struct PolicyEntry {
    mode: ExecutionMode,
}

struct CommandOverride {
    command: String,            // Binary name
    args_contains: Option<Vec<String>>,
    intent: Option<CommandIntent>,
    mode: Option<ExecutionMode>,
}
```

## File Formats

### `.boundline/execution-policy.toml`

```toml
[defaults]
unknown_intent_mode = "require-approval"
missing_policy_mode = "deny"

[policy.read.green]
mode = "allow"
# ... 18 entries total

[[overrides]]
command = "rm"
intent = "mutate"
mode = "require-approval"
```

### `.boundline/redaction.toml`

```toml
[defaults]
enabled = true
replacement = "[REDACTED:{kind}]"
max_preview_chars = 4

[[patterns]]
id = "github-token"
kind = "github_token"
regex = '''gh[pousr]_[A-Za-z0-9_]{36,255}'''
severity = "high"
replacement = "[REDACTED:github_token]"
```

### `.boundline/traces/<trace-id>.json`

Single JSON object per command execution, conforming to EvidencePacket schema.

## State Machine

```
Command Received
     │
     ▼
┌─────────────┐
│  Classify   │───→ CommandIntent
│   Intent    │
└─────────────┘
     │
     ▼
┌─────────────┐
│  Resolve    │───→ ExecutionMode
│   Policy    │
└─────────────┘
     │
     ├── deny ──→ Block + Log
     │
     ├── require-approval ──→ Pending Approval ──→ Execute or Deny
     │
     ├── dry-run ──→ DryRunTier ──→ Evidence (no mutations)
     │
     └── allow / no-mutation ──→ Execute ──→ Capture ──→ Redact ──→ Persist
```
