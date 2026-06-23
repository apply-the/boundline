# Browser Provider Protocol Contract

**Protocol Line**: `browser-provider-v1`
**Transport**: JSON over stdio (subprocess, stdin/stdout)
**Schema Version**: 1

## Overview

The browser provider contract defines the JSON request and response schemas exchanged between Boundline (the orchestrator) and a registered browser capability provider (an external binary). Boundline spawns the provider process, writes one JSON request per line to stdin, and reads one JSON response per line from stdout. Stderr is captured for diagnostics and surfaced on provider errors.

## Request Schema

```json
{
  "schema_version": 1,
  "validation_run_id": "browser-run-abc123",
  "url": "http://localhost:3000/dashboard",
  "readiness": {
    "locator": {
      "type": "test_id",
      "value": "dashboard-ready"
    },
    "state": "visible",
    "timeout_seconds": 20,
    "stabilization_delay_ms": 250
  },
  "interaction_script": {
    "steps": [
      { "action": "navigate", "url": "/login", "timeout_seconds": 10 },
      { "action": "type", "selector": "#username", "text": "admin", "timeout_seconds": 5 },
      { "action": "click", "selector": "#submit", "timeout_seconds": 5 },
      { "action": "screenshot", "label": "after-login" }
    ]
  },
  "accessibility": true,
  "dom_inspection": { "root_selector": "#main", "max_depth": 5 },
  "baseline_ref": "dashboard-home-2026-06-01",
  "timeouts": {
    "page_load_seconds": 30,
    "readiness_seconds": 20,
    "script_step_seconds": 10,
    "execution_seconds": 120
  },
  "network_allowlist": ["localhost", "api.example.com", "cdn.example.com"],
  "artifact_dir": ".boundline/sessions/sess-xyz/browser/browser-run-abc123"
}
```

### Request Fields

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `schema_version` | Yes | `u32` | Must be `1` |
| `validation_run_id` | Yes | `String` | Unique run identifier |
| `url` | Yes | `String` | Target page URL |
| `readiness` | No | `ReadinessLocator` | Page-ready condition |
| `interaction_script` | No | `InteractionScript` | Scripted steps |
| `accessibility` | No | `bool` | Run accessibility audit |
| `dom_inspection` | No | `DomInspectionConfig` | DOM capture config |
| `baseline_ref` | No | `String` | Visual baseline identifier |
| `timeouts` | No | `ValidationTimeouts` | Timeout overrides |
| `network_allowlist` | No | `Vec<String>` | Permitted domains |
| `artifact_dir` | Yes | `String` | Output directory |

## Response Schema

```json
{
  "schema_version": 1,
  "validation_run_id": "browser-run-abc123",
  "status": "completed",
  "evidence_packet": {
    "provider_id": "browser-playwright",
    "page_title": "Dashboard — My App",
    "http_status": 200,
    "artifacts": [
      {
        "kind": "screenshot",
        "relative_path": "screenshots/final.png",
        "content_hash": "sha256:abc123def456",
        "media_type": "image/png",
        "byte_size": 245760,
        "created_at": "2026-06-20T10:30:00Z",
        "retention_class": "required_evidence",
        "validation_run_id": "browser-run-abc123"
      },
      {
        "kind": "console_log",
        "relative_path": "console.json",
        "content_hash": "sha256:789012ghi345",
        "media_type": "application/json",
        "byte_size": 4096,
        "created_at": "2026-06-20T10:30:01Z",
        "retention_class": "required_evidence",
        "validation_run_id": "browser-run-abc123"
      }
    ],
    "findings": [
      {
        "kind": "console_error",
        "severity": "warning",
        "message": "Failed to load resource: /api/stale-endpoint",
        "evidence_refs": ["console.json"],
        "retryability": null,
        "confirmed_intermittent": false
      }
    ],
    "timing": {
      "queue_wait_ms": 120,
      "navigation_ms": 1840,
      "readiness_wait_ms": 350,
      "script_execution_ms": null,
      "accessibility_ms": null,
      "total_ms": 2310
    },
    "capabilities_active": ["screenshot", "console", "readiness"],
    "started_at": "2026-06-20T10:29:58Z",
    "completed_at": "2026-06-20T10:30:01Z"
  }
}
```

### Response Fields

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `schema_version` | Yes | `u32` | Must be `1` |
| `validation_run_id` | Yes | `String` | Echoed from request |
| `status` | Yes | `StepStatus` | completed, failed, timed_out, provider_error |
| `evidence_packet` | On success | `BrowserEvidencePacket` | Full evidence payload |
| `error` | On failure | `ErrorDetail` | Error code and message |

### Status Values

| Value | Meaning |
|-------|---------|
| `completed` | Step finished (findings may still be present) |
| `failed` | Step encountered a blocking failure |
| `timed_out` | Execution timeout reached |
| `provider_error` | Provider internal error (stderr captured) |
| `cancelled` | Cancelled by orchestrator before completion |
| `queue_timeout` | Queued request timed out before execution |
| `queue_full` | Queue at capacity; request rejected immediately |

### Error Detail

```json
{
  "code": "BROWSER_LAUNCH_FAILED",
  "message": "Could not find Chromium binary at configured path.",
  "stderr_summary": "Error: spawn ENOTFOUND",
  "partial_evidence": null
}
```

## Provider Startup Handshake

Before accepting validation requests, the provider writes a startup line to stdout:

```json
{
  "protocol": "browser-provider-v1",
  "schema_version": 1,
  "provider_id": "browser-playwright",
  "capabilities": {
    "screenshot": true,
    "console": true,
    "readiness_locators": true,
    "dom_inspection": true,
    "accessibility": true,
    "network_failure_capture": true,
    "interaction_scripts": true,
    "visual_diff": false
  },
  "concurrency": {
    "active": 0,
    "max": 2,
    "queue_depth": 0,
    "max_queue": 10
  }
}
```

Boundline waits for this handshake line (with configurable startup timeout) before dispatching validation requests. If the handshake does not arrive or is malformed, the provider is marked as `Blocked` with a health failure.
