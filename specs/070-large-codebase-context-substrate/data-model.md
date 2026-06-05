# Data Model: Large Codebase Context Substrate

## Entity: Context Candidate

Represents one repository, runtime, trace, or governed artifact considered for
inclusion in the active context pack.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `source_ref` | Relative artifact identifier | Yes | Must identify the file, packet document, trace artifact, or logical source without using absolute paths. |
| `source_kind` | Stable label such as `workspace_file`, `symbol`, `test`, `trace`, `canon_artifact`, `docs_project`, or `runtime_state` | Yes | Stable labels keep ranking and rendering machine-readable. |
| `authority` | Stable owner label | Yes | Distinguishes local workspace evidence, Boundline runtime state, Canon-enriched evidence, or authored project memory. |
| `fidelity_tier` | `critical`, `supporting`, `ambient`, or `archived` | Yes | Tier is assigned before inclusion, compaction, or omission decisions. |
| `relevance_signals` | Ordered list of local ranking signals | No | Signals explain why the candidate was considered, such as symbol match, dependency relation, or changed-file proximity. |
| `estimated_budget_cost` | Integer | No | Used only as a bounded local estimate, not as an authoritative token count. |

## Entity: Context Fidelity Tier

Represents the protection level attached to a candidate before it becomes a
pack entry.

| Tier | Meaning | Rules |
|---|---|---|
| `critical` | Required for a safe planning or execution decision | Cannot be silently omitted or represented only by a lossy summary; absence at required fidelity blocks admission. |
| `supporting` | Relevant and often useful | May be excerpted, summarized, or retrieved on demand with attribution. |
| `ambient` | Background or low-priority context | Defaults to summary, signature, or index-only modes and must not dominate the active pack. |
| `archived` | Superseded or discardable context | Excluded from normal planning and execution unless an explicit archive-oriented lookup asks for it. |

## Entity: Context Inclusion Mode

Represents how a candidate was actually handled in the final pack.

Allowed values:

- `full`
- `excerpt`
- `summary`
- `signature`
- `digest`
- `omitted`

Mode is mandatory for every final pack entry. A mode describes the actual
representation, not the candidate's importance.

## Entity: Context Pack Entry

Represents the persisted handling decision for one candidate in the active
context pack.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `candidate_ref` | Stable reference to `ContextCandidate` | Yes | Must resolve back to the candidate that was evaluated. |
| `mode` | `Context Inclusion Mode` | Yes | `omitted` is a first-class result, not absence. |
| `reason` | Stable operator-readable reason | Yes | Explains inclusion, compaction, downgrade, or omission. |
| `required_for_admission` | Boolean | Yes | `true` for entries whose absence or downgrade can block planning or execution. |
| `resolved_excerpt_anchor` | Optional anchor | No | Present when `excerpt` or bounded retrieval points to a stable file or artifact range. |
| `digest_ref` | Optional `DigestBackedArtifactRef` | No | Present when large artifacts are compacted by digest. |
| `lifecycle_relevance` | Stable bounded label | No | Indicates why the entry matters to the active phase or stage. |
| `risk_relevance` | Stable bounded label | No | Indicates why the entry matters to active risk or validation work. |

## Entity: Context Omission Finding

Represents an inspectable explanation for context that was excluded, downgraded,
compacted, or blocked.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `severity` | `info`, `warning`, or `blocking` | Yes | `blocking` prevents planning or execution admission. |
| `reason_code` | Stable code | Yes | Codes remain stable across session, inspect, trace, and assistant surfaces. |
| `candidate_ref` | Stable reference | Yes | Each finding must point at the affected candidate. |
| `message` | Operator-readable summary | Yes | Explains the omission or downgrade and the expected repair action. |
| `required_fidelity` | Optional tier | No | Present when the issue is specifically a fidelity mismatch. |
| `observed_mode` | Optional mode | No | Present when a downgrade or compaction actually occurred. |

Initial-slice reason codes:

- `unsafe_full_read_refused`
- `critical_context_unavailable`
- `critical_context_downgraded`
- `search_required_before_read`
- `artifact_compacted_to_digest`
- `repository_map_unavailable`
- `snapshot_cache_stale`
- `tracked_cache_detected`

## Entity: Repository Map Snapshot

Represents the derived local navigation model used before large reads.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `snapshot_id` | Stable identifier | Yes | Identifies one derived repository-map build. |
| `workspace_fingerprint` | Stable fingerprint | Yes | Ties the snapshot to one workspace shape. |
| `freshness_state` | `ready`, `stale`, `missing`, `degraded`, or `corrupt` | Yes | The map must not be treated as current when stale. |
| `files` | Ordered `RepositoryMapNode` list | No | Bounded projection of files and lightweight metadata. |
| `relationships` | Ordered relation list | No | Bounded graph edges such as imports, callers, tests, and Canon references. |

## Entity: Repository Map Node

Represents one navigable file or semantic unit in the repository map.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `path` | Relative path | Yes | Must stay repository-relative. |
| `symbol_refs` | Ordered list | No | May include functions, types, modules, routes, schemas, tests, or config blocks. |
| `owner_hint` | Optional bounded label | No | Carries team, system, or domain ownership hints when available locally. |
| `criticality_hint` | Optional bounded label | No | May be used as one local signal during context ranking. |

## Entity: DigestBackedArtifactRef

Represents a recoverable reference for a compacted large artifact.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `digest` | Stable content digest | Yes | Must identify the original artifact deterministically. |
| `artifact_kind` | Stable label | Yes | Examples: `log`, `diff`, `generated_output`, `packet_document`. |
| `summary` | Bounded operator-readable summary | Yes | Summary explains why the artifact mattered without replaying all content. |
| `excerpt_anchor` | Optional anchor | No | Present when a relevant excerpt accompanies the digest. |
| `resolve_path` | Relative reference | Yes | Must let operators recover the authoritative source on demand. |

## Entity: Snapshot Cache Entry

Represents the reusable persistent local snapshot cache for the context
substrate.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `cache_key` | Stable identifier | Yes | Uniquely identifies one derived cache entry. |
| `workspace_fingerprint` | Stable fingerprint | Yes | Must match the workspace shape the cache was derived from. |
| `freshness_state` | `ready`, `stale`, `missing`, `degraded`, `tracked`, or `corrupt` | Yes | Any non-`ready` state prevents trusted reuse. |
| `derived_artifacts` | Ordered list of derived artifact refs | No | May include repository-map snapshots, retrieval metadata, or last-known-good pack projections. |
| `freshness_events` | Ordered list of `FreshnessEvent` | No | Explains why a cache entry is stale or downgraded. |
| `authority_boundary` | Stable label | Yes | Always describes the cache as derived and non-authoritative. |

## Entity: Freshness Event

Represents one condition that invalidates or downgrades cache reuse.

Allowed values for the initial slice:

- `branch_switch`
- `merge`
- `rebase`
- `config_change`
- `schema_change`
- `adapter_change`
- `canon_packet_change`
- `workspace_shape_change`

Each event may carry:

- `detected_at`
- `evidence_ref`
- `message`

## Entity: Patch Safe Edit Attempt

Represents one large-file edit attempt that must stay bounded and verified.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `target_ref` | Relative file ref | Yes | Identifies the file being edited. |
| `anchor_refs` | Ordered list | Yes | Must define the bounded hunk anchors used for the patch. |
| `pre_apply_digest` | Stable digest | Yes | Protects against drift before apply. |
| `post_apply_verification` | Ordered list of checks | Yes | Includes formatter, parser, or targeted validation evidence. |
| `result_state` | `applied`, `drifted`, `rejected`, or `manual_review_required` | Yes | Large-file edits must not silently succeed when anchors drift. |

## State Transitions

```text
candidate discovered
  -> tier classified
  -> ranking signals attached

critical candidate unavailable at required fidelity
  -> omission finding severity=blocking
  -> context admission blocked

oversized artifact without explicit safe full-read path
  -> mode=omitted or digest/excerpt
  -> omission finding code=unsafe_full_read_refused or search_required_before_read

snapshot cache ready + no freshness event
  -> reusable derived aid

snapshot cache ready + freshness event
  -> stale or degraded before reuse

tracked cache files detected
  -> freshness_state=tracked
  -> diagnostics surface repair action

large-file patch anchors drift
  -> result_state=manual_review_required or rejected
  -> edit not treated as accepted
```

## Compatibility Rules

- Older sessions without the new substrate fields must deserialize and render
  successfully.
- `status`, `inspect`, traces, and assistant surfaces may omit additive
  substrate fields when the feature has never run for a session.
- The snapshot cache must remain derived state under `.boundline/`; it must not
  be promoted to docs, Canon packets, or reviewed memory surfaces.
- Repository-map and cache artifacts must remain repository-relative and local;
  no absolute workspace paths may be persisted in stable files.
