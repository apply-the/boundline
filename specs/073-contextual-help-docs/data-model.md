# Data Model: Contextual Help And Documentation Architecture (Boundline)

**Feature**: 073-contextual-help-docs
**Date**: 2026-06-06

## Entities

### HelpNextState

An enumeration of detectable workspace/runtime states.

| Variant | Description |
|---------|-------------|
| `Uninitialized` | No `.boundline/` directory exists |
| `Initialized` | `.boundline/` exists but no active session |
| `Active` | Active session with current lifecycle phase, no blockers |
| `Blocked` | Active session with a blocking planning-analysis or execution gate |
| `Failed` | Session in a terminal failure state |
| `Ready` | Healthy active session — no blockers, next command available |

### HelpNextDiagnostic

A single actionable finding.

| Field | Type | Description |
|-------|------|-------------|
| `key` | `String` | Stable diagnostic key (e.g., `"workspace_not_initialized"`, `"config_missing_key"`) |
| `severity` | `DiagnosticSeverity` (enum) | `Blocking`, `Warning`, `Info` |
| `message` | `String` | Human-readable description |
| `source` | `Option<String>` | Source file or config path (e.g., `".boundline/config.toml"`) |
| `command` | `Option<String>` | Recommended CLI command to resolve the issue |
| `docs_key` | `String` | Key for the link map (e.g., `"workspace_not_initialized"`) |

**DiagnosticSeverity** variants:
- `Blocking` — prevents the next action from proceeding
- `Warning` — does not block but indicates a gap
- `Info` — informational, no action needed

### HelpNextRecommendation

The resolved next action returned to the operator.

| Field | Type | Description |
|-------|------|-------------|
| `state` | `HelpNextState` | Current workspace/runtime state |
| `blockers_found` | `bool` | Whether any blocking diagnostics were found |
| `primary_issue` | `Option<HelpNextDiagnostic>` | Highest-priority blocking issue (or None if healthy) |
| `additional_issues` | `Vec<HelpNextDiagnostic>` | Remaining issues ordered by priority |
| `additional_count` | `u64` | Count of issues beyond the primary |
| `recommended_action` | `String` | Human-readable action description |
| `recommended_command` | `Option<String>` | Exact CLI command to run |
| `reason` | `String` | Why this action is recommended |
| `docs_link` | `Option<String>` | Resolved documentation URL from link map |

### HelpNextEvent

The structured runtime event emitted on each invocation (payload for `EventType::HelpNextRequested`).

| Field | Type | Description |
|-------|------|-------------|
| `state` | `String` | Serialized `HelpNextState` variant name |
| `lifecycle_phase` | `Option<String>` | Current lifecycle phase when in an active session |
| `blocked_category` | `Option<String>` | `"planning"`, `"execution"`, or `"configuration"` when blocked |
| `diagnostics_count` | `u64` | Total diagnostics found |
| `recommended_action_id` | `String` | Stable action identifier |
| `recommended_command` | `Option<String>` | CLI command string |
| `docs_link` | `Option<String>` | Resolved docs link |
| `output_format` | `String` | `"human"` or `"json"` |

### HelpLinkMap

The versioned TOML file mapping diagnostic keys to documentation URLs.

```toml
[metadata]
schema_version = "1.0"

[links]
workspace_not_initialized = "wiki/Getting-Started"
workspace_initialized_no_session = "wiki/Daily-Operating-Guide"
session_blocked_planning = "wiki/Troubleshooting#planning-blocked"
session_blocked_execution = "wiki/Troubleshooting#execution-blocked"
session_failed = "wiki/Troubleshooting#session-failed"
config_missing_key = "wiki/Configuration#required-keys"
provider_not_activated = "wiki/Configuration#provider-routes"
context_pack_missing = "wiki/Core-Concepts#context-packs"
guardian_finding_active = "wiki/Guidance-And-Guardians#guardian-findings"
stop_rule_active = "wiki/Guidance-And-Guardians#stop-rules"
session_healthy = "wiki/Daily-Operating-Guide"
fallback = "wiki/Troubleshooting"
```

**Validation rules**:
- `schema_version` must be present and parse as `MAJOR.MINOR`.
- Unknown keys in `[links]` are silently ignored (forward-compatible).
- Missing keys produce a non-blocking warning with `docs_link` set to the `"fallback"` entry.
- The file is loaded once per `help-next` invocation; not cached across invocations.

## Entity Relationships

```
HelpNextState 1 ──── 1 HelpNextRecommendation
HelpNextRecommendation 1 ──── 0..1 HelpNextDiagnostic (primary_issue)
HelpNextRecommendation 1 ──── * HelpNextDiagnostic (additional_issues)
HelpNextRecommendation 1 ──── 0..1 HelpLinkMap (for docs_link resolution)
HelpNextEvent (standalone — emitted per invocation)
```
