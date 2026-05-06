# Canon Request/Response Contract: Native Canon CLI Surface

**Feature**: 042-native-canon-cli
**Date**: 2026-05-05

## Overview

This contract documents the Canon governance CLI interface that Boundline
invokes.  Boundline is the client; Canon is the external governed runtime.
All interactions use JSON over stdin/stdout through the Canon CLI binary.

---

## Operations

### `canon governance capabilities --json`

**Purpose**: Discover the Canon binary's supported governance surface.

**Invocation**:
```text
canon governance capabilities --json
```

**Response schema**:
```json
{
  "canon_version": "0.40.0",
  "supported_modes": [
    "requirements", "discovery", "system-shaping", "architecture",
    "backlog", "change", "implementation", "refactor", "review",
    "verification", "incident", "security-assessment",
    "system-assessment", "migration", "supply-chain-analysis"
  ],
  "operations": ["start", "refresh", "capabilities"],
  "status_values": [
    "pending_selection", "running", "governed_ready",
    "awaiting_approval", "blocked", "completed", "failed"
  ],
  "approval_state_values": [
    "not_needed", "requested", "granted", "rejected", "expired"
  ],
  "packet_readiness_values": [
    "pending", "incomplete", "reusable", "rejected"
  ],
  "template_hints": {
    "requirements": {
      "expected_sections": ["Problem Statement", "Stakeholders", "Functional Requirements", "Non-Functional Requirements"],
      "input_guidance": "Provide a product brief or PRD covering the problem domain and target users"
    }
  }
}
```

**Boundline verification**: Boundline checks that:
- `operations` contains `"start"` and `"refresh"`
- `supported_modes` contains all 15 canonical mode identifiers
- Any missing mode or operation is reported through `CanonSurfaceVerification`

---

### `canon governance start --json`

**Purpose**: Start a governed stage for a specific Canon mode.

**Invocation**:
```text
echo '<request_json>' | canon governance start --json
```

**Request schema** (Boundline → Canon):
```json
{
  "governance_attempt_id": "uuid-v4",
  "stage_key": "delivery:requirements",
  "goal": "Build a task management API with authentication",
  "workspace_ref": "/path/to/workspace",
  "autopilot": false,
  "mode": "requirements",
  "system_context": "new",
  "risk": "standard",
  "zone": "development",
  "owner": "operator-name",
  "input_documents": [
    {
      "kind": "stage-brief",
      "name": "product-brief.md",
      "content": "# Product Brief\n\n..."
    },
    {
      "kind": "authored-brief",
      "name": "architecture-notes.md",
      "content": "# Architecture Notes\n\n..."
    },
    {
      "kind": "clarification-answer",
      "name": "clarification-001",
      "content": "The API should support OAuth 2.0 and API key authentication."
    }
  ],
  "bounded_context": {
    "read_targets": ["/path/to/workspace/src/", "/path/to/workspace/Cargo.toml"],
    "stage_brief_ref": "product-brief.md",
    "reused_packets": [
      {
        "packet_ref": "pkt-prior-stage",
        "canon_mode": "requirements",
        "stage_key": "delivery:requirements",
        "content_hash": "sha256:abc123"
      }
    ]
  }
}
```

**Response schema** (Canon → Boundline):
```json
{
  "status": "governed_ready",
  "approval_state": "not_needed",
  "packet": {
    "packet_ref": "pkt-uuid",
    "runtime": "canon",
    "canon_mode": "requirements",
    "readiness": "reusable",
    "missing_sections": [],
    "headline": "Requirements document produced for task management API"
  },
  "message": "Governed requirements document produced successfully",
  "run_ref": "run-uuid",
  "produced_document": ".canon/runs/run-uuid/requirements.md"
}
```

**Status values and Boundline behavior**:

| Status | Boundline Action |
|--------|-----------------|
| `governed_ready` | Record governed document, advance to next stage |
| `awaiting_approval` | Persist approval state, surface via `status`/`next`, wait for operator `refresh` |
| `blocked` | Record block reason, surface via `status`/`next`, do not advance |
| `pending_selection` | Surface missing mode choice or unresolved input as clarification prompt |
| `incomplete` | Surface `missing_sections` as targeted clarification, do not advance |
| `running` | Wait and re-poll (bounded by max-step limits) |
| `failed` | Record failure, surface error, terminate governed stage |

---

### `canon governance refresh --json`

**Purpose**: Check approval state or updated readiness for an existing governed
stage without re-running the full `start` operation.

**Invocation**:
```text
echo '<request_json>' | canon governance refresh --json
```

**Request schema** (subset of start request):
```json
{
  "governance_attempt_id": "uuid-v4",
  "run_ref": "run-uuid",
  "stage_key": "delivery:requirements",
  "workspace_ref": "/path/to/workspace"
}
```

**Response schema**: Same as `start` response.

---

## Input Document Assembly Contract

Boundline assembles operator-provided inputs into `input_documents` using these
mappings:

| Operator Input | `kind` Value | Notes |
|----------------|-------------|-------|
| First `--brief` file or primary goal text | `"stage-brief"` | Always first; sets `stage_brief_ref` in bounded_context |
| Additional `--brief` files | `"authored-brief"` | Ordered by CLI argument order |
| Clarification answers | `"clarification-answer"` | Named `clarification-NNN` |
| Repository evidence (auto-detected) | `"repository-evidence"` | E.g., Cargo.toml, package.json |

Bounded context `reused_packets` are populated automatically from prior governed
stages in the same session, using `GovernedDocumentRef` entries from
`GovernedSessionLifecycle.accumulated_context`.

---

## Error Contract

When Canon returns a non-zero exit code or unparseable JSON:

```json
{
  "error": true,
  "canon_exit_code": 1,
  "canon_stderr": "Error: mode 'supply-chain-analysis' is not supported in this version",
  "boundline_action": "blocked",
  "message": "Canon governance start failed: unsupported mode",
  "suggested_actions": ["Upgrade Canon to version 0.40.0 or later"]
}
```

Boundline treats Canon errors as governance stage failures and surfaces them
through the standard failure path (session status → Failed, trace recorded).
