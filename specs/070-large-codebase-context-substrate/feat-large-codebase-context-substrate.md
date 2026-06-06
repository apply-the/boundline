# Large Codebase Context Substrate

## Integration Update

This roadmap item should absorb **Context Fidelity Tiers** as a core part of the local context substrate.

It should also define the boundary for a follow-up **Persistent Context Snapshot Cache**.

The cache remains:

- derived
- local
- disposable
- rebuildable
- non-authoritative

It must not become memory or semantic truth.

## Relationship To Other Roadmap Files

| Related file | Relationship |
|---|---|
| `05-plan-analysis-contract.md` | Consumes context packs and consistency evidence, but does not own indexing or context selection |
| `07-external-capability-provider-protocol.md` | Provider-supplied context belongs to provider protocol, not local context substrate |
| `../072-evals-runtime-observability/feat-evals-and-runtime-observability.md` | Owns evals, events, trace export, and trace compaction |
| `16-session-memory-and-repository-knowledge-distillation.md` | Owns reviewed memory proposals; cache must not become memory |
| `19-plan-execution-orchestration.md` | Consumes validated context and task surfaces during execution |

## Added Scope

Add these concepts:

- context fidelity classification
- context inclusion mode
- explicit omitted-context reasons
- critical-context blocking behavior
- cache freshness boundaries
- cache is not memory rule

## Context Fidelity Tiers

Boundline should not treat every context item equally.

Each context candidate should be classified before inclusion, summarization, digesting, or omission.

### Tier 0: Critical Context

Critical context must remain high fidelity.

Examples:

- active goal
- active feature specification
- active plan
- tasks or backlog accepted for execution
- contracts
- failing tests
- current phase request
- accepted Canon packets
- stage ownership contract
- execution admission gates

Rules:

- Must not be silently omitted.
- Must not be represented only by a lossy summary.
- Must be included directly or referenced through mandatory retrieval.
- Missing critical context is a blocking finding.

### Tier 1: Supporting Context

Supporting context is useful but can often be excerpted or summarized.

Examples:

- nearby source modules
- relevant documentation
- related tests
- previous traces for the same feature
- architecture notes
- known examples

Rules:

- May be excerpted.
- May be summarized with source references.
- May be retrieved on demand.
- Must keep source attribution.

### Tier 2: Ambient Context

Ambient context is background information.

Examples:

- old completed traces
- broad documentation
- unrelated roadmap notes
- long historical chat logs
- stale examples

Rules:

- Summary or index-only by default.
- Included only when relevance rules activate it.
- Must not dominate planning context.

### Tier 3: Archived Or Discardable Context

Archived or discardable context should not influence normal planning.

Examples:

- superseded drafts
- duplicate assistant output
- abandoned intermediate attempts
- obsolete generated artifacts

Rules:

- Not included by default.
- Available only through explicit inspect or archive lookup.
- Must not affect execution admission.

## Context Pack Budgeting

Every included context item should record:

- source reference
- fidelity tier
- inclusion mode
- reason
- authority
- estimated cost
- lifecycle relevance
- risk relevance

Supported inclusion modes:

- full
- excerpt
- summary
- signature
- digest
- omitted

Example:

```json
{
  "source_ref": "specs/070/spec.md",
  "tier": "critical",
  "mode": "full",
  "reason": "active feature specification",
  "authority": "feature_packet",
  "budget_cost": 18400
}
```

## Search-Before-Read

Before reading a large file, Boundline should require one or more of:

- ripgrep query
- file path search
- symbol search
- import/export search
- test relation lookup
- previous trace relation lookup

## Symbol-Aware Indexing

Prefer semantic units over arbitrary token chunks:

- functions
- methods
- classes
- structs
- traits or interfaces
- modules
- routes
- schemas
- tests
- migrations
- config blocks

## Repository Map

Store a compact navigation model:

```text
file -> symbols
symbol -> location
file -> imports/exports
symbol -> callers/callees where available
symbol -> tests where available
file -> Canon refs where available
file -> owner hints where available
```

## Hybrid Ranking

Rank context candidates with multiple signals:

- lexical score
- symbol match
- graph distance
- dependency relation
- test relation
- recent trace relation
- changed-file proximity
- Canon authority
- guidance/guardian relevance
- fidelity tier
- optional vector similarity

## Lazy Hash References

For huge logs and diffs:

```text
store full artifact by digest
include digest + summary + relevant excerpt
resolve full content only on demand
```

## Patch-Safe Editing

For huge files:

- never rewrite full file by default
- use anchored hunks
- verify anchors before apply
- verify file digest expectations
- re-run formatter or parser
- emit trace for applied hunks
- fall back to manual review if anchors drift

## Persistent Context Snapshot Cache Boundary

A follow-up sub-slice may add a persistent local cache for:

- workspace fingerprint
- active spec context
- adapter capabilities
- repository map
- retrieval index metadata
- last good planning context

Rules:

- cache is derived and rebuildable
- cache files are ignored by Git
- branch switch, merge, rebase, config changes, schema changes, adapter changes, and Canon packet changes are freshness events
- full rebuild must not run automatically inside Git hooks
- `doctor` must report accidentally tracked cache files

Cache is not memory. `16-session-memory-and-repository-knowledge-distillation.md` owns reviewed memory proposals and Canon promotion paths.

## Acceptance Criteria Additions

- Boundline refuses huge full-file reads unless explicitly allowed.
- Boundline can build a repo map for common language stacks.
- Context Packs show why items were selected.
- Context Packs show fidelity tier and inclusion mode for each item.
- Critical context is not silently omitted or represented only by lossy summaries.
- Large logs and diffs are compacted with hash references.
- Patch application uses anchors and post-apply verification.
- Inspect can show omitted context and why it was omitted.
- Tests cover huge file, huge diff, missing index, critical-context omission, and stale cache cases.

## Risks

- Overbuilding the index before proving retrieval quality.
- Treating vector search as source of truth.
- Hiding important code behind summaries.
- Slowing down every session.
- Misclassifying critical context as compressible.

## Hard Rules

- Summaries are for navigation. Source code is required for edits.
- Critical context must not be silently lossy.
- Cache is not memory.
