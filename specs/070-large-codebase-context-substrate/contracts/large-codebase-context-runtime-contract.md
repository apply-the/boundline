# Contract: Large Codebase Context Runtime Projection

## Purpose

Define the additive Boundline-owned runtime contract for large-codebase context
selection, omission visibility, repository-map-assisted discovery, digest-backed
compaction, and derived snapshot-cache freshness.

## Evaluation Order

```text
goal accepted
  -> bounded context discovery
  -> candidate tier classification
  -> search-before-read narrowing
  -> hybrid local ranking
  -> inclusion / compaction / omission decision
  -> critical-context admission check
  -> planning or execution continuation
```

The substrate runs before Boundline treats the context pack as credible for the
current planning or execution decision.

## Evaluation Inputs

The initial slice may use only these local or governed inputs:

- workspace-relative files and repository metadata
- existing project-index and project-memory hints
- existing local context-intelligence and retrieval-index state
- current session, trace, validation, and changed-file signals already owned by
  Boundline
- optional Canon-enriched artifacts already present in the workspace and already
  supported by Boundline

The slice is read-only with respect to source artifacts. It may persist derived
local metadata, but it must not mutate Canon packet contracts, reviewed memory,
or repository files during context selection.

## Additive Session Projection

When the substrate has run, session, status, inspect, and orchestration
snapshots may include fields like:

```json
{
  "context_pack_projection": {
    "state": "blocked",
    "entries": [
      {
        "source_ref": "src/orchestrator/goal_planner.rs",
        "tier": "critical",
        "mode": "excerpt",
        "reason": "active planner surface narrowed by symbol search",
        "authority": "workspace"
      },
      {
        "source_ref": "logs/failed-run.log",
        "tier": "supporting",
        "mode": "digest",
        "reason": "large artifact compacted to bounded digest-backed reference",
        "authority": "runtime_artifact"
      }
    ],
    "omission_findings": [
      {
        "severity": "blocking",
        "reason_code": "critical_context_unavailable",
        "message": "active failing test file could not be loaded at required fidelity",
        "candidate_ref": "tests/integration/host_session_runtime_flow.rs"
      }
    ],
    "repository_map_state": "ready",
    "snapshot_cache_state": "stale"
  }
}
```

These fields are additive. Older snapshots may omit them entirely.

## Required Vocabularies

### Fidelity Tiers

- `critical`
- `supporting`
- `ambient`
- `archived`

### Inclusion Modes

- `full`
- `excerpt`
- `summary`
- `signature`
- `digest`
- `omitted`

### Repository Map States

- `ready`
- `stale`
- `missing`
- `degraded`
- `corrupt`

### Snapshot Cache States

- `ready`
- `stale`
- `missing`
- `degraded`
- `tracked`
- `corrupt`

## Blocking Rules

The substrate must block planning or execution admission when any of these
conditions is true:

- a `critical` context candidate is omitted without an approved high-fidelity
  retrieval path
- a `critical` candidate is represented only by a lossy mode that does not
  satisfy the current decision
- an oversized artifact requires a full read for safety, but only an unsafe
  unrestricted path is available
- required repository-map or freshness information is missing and Boundline
  cannot establish a safe context pack for the current step

The runtime must not silently downgrade these conditions into warning-only
states.

## Search-Before-Read Rules

Before a large file or similar artifact is read in full, the substrate must use
one or more bounded discovery signals such as:

- file path matches
- symbol matches
- import or export relations
- test relations
- previous trace relations
- changed-file proximity
- Canon relation hints already present locally

If no narrowing signal exists, the runtime must refuse the unsafe full read and
record the omission reason.

## Compaction Rules

- Large logs, diffs, generated outputs, and similar artifacts may be compacted
  into digest-backed references with bounded summaries or excerpts.
- Every compacted artifact must preserve enough attribution to recover the
  original source on demand.
- `digest` or `summary` modes must not be used for `critical` context when the
  active decision requires higher fidelity.

## Patch-Safe Editing Rules

- Large-file edits must not default to full-file rewrites.
- The runtime must use anchored edit scopes, drift checks, and post-apply
  verification before treating the change as accepted.
- If anchor drift is detected, the runtime must surface a rejected or
  manual-review-required state instead of pretending the edit was safely
  applied.

## Snapshot Cache Boundary

The persistent local context snapshot cache is:

- derived
- local
- disposable
- rebuildable
- non-authoritative

It is not:

- memory
- reviewed knowledge
- Canon truth
- an alternative planning source when freshness is stale or unknown

Freshness events must invalidate or downgrade the cache before reuse. Tracked
cache files and stale cache state must surface through diagnostics and repair
guidance.

## Assistant Asset Contract

Copilot, Claude, Codex, and Antigravity status, inspect, plan, and run assets
must preserve:

- the active context-pack state
- included and omitted entries when projected
- fidelity tiers
- inclusion modes
- omission reasons
- repository-map state
- snapshot-cache state
- continuation or repair guidance

Assistant assets must not synthesize a safe-ready state when the substrate has
reported a blocked or stale critical-context outcome.

## Explicit Non-Goals

- No remote retrieval service or hosted vector store
- No reviewed memory promotion
- No Canon contract mutation
- No implicit background repository crawl
- No generalized semantic database for every language ecosystem
- No silent full-file fallback when narrowing signals are weak
