# Data Model: Project Memory Delivery Integration

## Entities

### PromotionStateView

Consumer-side read-only projection of Canon's promotion state vocabulary.

| Variant | Meaning (consumer perspective) |
|---------|-------------------------------|
| `Stable` | Canon output is accepted; Boundline may use as credible context |
| `PendingOrIndex` | Canon output is pending or index-only; visible but non-authoritative |
| `EvidenceOnly` | Canon output is evidence-only; usable for assurance, not planning |
| `Manual` | Canon output requires manual action; treat as absent for planning |
| `Unknown` | Canon output has an unrecognized promotion state; treat as non-authoritative |

**Mapping from Canon vocabulary**: `auto` maps to `Stable`. `auto-if-approved`
maps to `Stable` only when emitted metadata reports
`approval_state = Completed` and `readiness = complete`; otherwise Boundline
keeps it non-authoritative (`PendingOrIndex` when approval metadata shows a
non-completed state, `Unknown` when approval metadata is missing). `pending-index`
and `index-only` map to `PendingOrIndex`. `evidence-only` maps to
`EvidenceOnly`. `manual` maps to `Manual`. Unrecognized values map to
`Unknown`.

### LineageRef

Consumer-side lineage metadata preserved from Canon packet-metadata sidecars.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| contract_version | String | yes | Shared contract version |
| source_run | String | yes | Originating Canon run identifier |
| mode | String | yes | Canon mode that produced the output |
| profile | String? | no | Producer profile label when the emitting framework exposes one |
| promotion_state | String | yes | Raw Canon promotion state |
| approval_state | String? | conditional | Required when promotion-state semantics depend on approval metadata, including `auto-if-approved` |
| readiness | String | conditional | Required when promotion-state semantics depend on completion metadata, including `auto-if-approved` |
| published_at | String? | no | RFC3339 publish timestamp |
| update_strategy | String? | no | Canon update strategy vocabulary |
| source_artifacts | Vec\<String\> | no | Producer-provided source artifact refs when the framework exposes them |

**Validation rules**: `contract_version` must be parseable as semver.

### CompatibilityOutcome

Result of checking a Canon `contract_version` against the Boundline-supported
version window.

| Variant | Meaning |
|---------|---------|
| `Compatible` | `contract_version` is on the supported `0.1.x` line |
| `Unsupported` | `contract_version` is malformed or outside the supported line |

### ProjectMemoryContext

Aggregated consumer-side snapshot of Canon project-memory state for a given
workspace.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| status | enum | yes | `Available`, `Absent`, `Incompatible` |
| compatibility | CompatibilityOutcome | when Available | Contract compatibility result |
| surfaces | Vec\<ProjectMemorySurface\> | when Available | Discovered Canon surfaces |
| evidence_refs | Vec\<LineageRef\> | no | Evidence surface lineage refs |
| effective_promotion_state | PromotionStateView | when Available | Highest-credibility state across surfaces |

### ProjectMemorySurface

A single Canon-promoted document discovered by Boundline.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| path | PathBuf | yes | Workspace-relative path to the promoted document |
| lineage | Option\<LineageRef\> | no | Parsed lineage sidecar if present |
| promotion_view | PromotionStateView | yes | Consumer-side promotion state |
| category | String | yes | Surface category (e.g. `architecture-map`, `domain-language`) |

## Entity Relationship Summary

```text
Workspace filesystem
  └── Canon-promoted surfaces (docs/project/, docs/evidence/)
        └── read_project_memory()
              └── ProjectMemoryContext
                    ├── CompatibilityOutcome (contract version check)
                    ├── Vec<ProjectMemorySurface>
                    │     └── LineageRef (from <surface>.packet-metadata.json)
                    │           └── PromotionStateView
                    └── feeds into
                          ├── stage_planner (credible vs. non-authoritative)
                          ├── assurance evaluation (evidence refs)
                          └── session-native views (status, inspect)
```
