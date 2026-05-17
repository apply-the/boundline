# Canon Semantic Acceleration Consumer Contract

- **Owner**: Boundline
- **Status**: Active consumer contract
- **Upstream Producer**: Canon
- **Stable Producer Surface**: Canon `.packet-metadata.json` sidecar
- **Current Consumer Line**: V1

## Purpose

Define the minimum Canon semantic metadata that Boundline may consume for S5.v2
semantic acceleration without shifting authority for retrieval policy, ranking,
or delivery control away from Boundline.

This brief is consumer-side only. Canon remains the producer and semantic owner
of artifact eligibility and provenance metadata. Boundline remains the runtime
that decides local fragment derivation, ranking, fallback, and inspectability.

## Required Canon Semantic Baseline

For a Canon artifact to participate in Boundline semantic acceleration, all of
these conditions must hold:

- the artifact is already inside Canon's documented stable indexing surface
- the artifact exposes a compatible semantic descriptor
- the artifact's semantic contract line is supported by Boundline
- the artifact provides a semantic provenance reference that Boundline can
  preserve on status, inspect, and trace surfaces

Until Canon republishes separate prose integration docs, Boundline treats the
typed packet metadata sidecar itself as the promoted stable producer contract
surface. The consumer relies on these producer-owned fields and contract lines:

- `lineage.contract_version = "v1"`
- `publication_target_class = "stable"`
- `semantic_descriptor.semantic_contract_line = "v1"`
- `semantic_descriptor.semantic_eligibility`
- `semantic_descriptor.semantic_provenance_boundary`
- `semantic_descriptor.semantic_provenance_ref`

Boundline verifies the consumer projection against this stable surface in
`tests/contract/context_intelligence_canon_semantic_contract.rs`.

## Minimum Canon Fields Boundline May Consume

Boundline may treat a Canon artifact as semantically eligible only when it can
recover these minimum facts from Canon's documented metadata carrier:

- `semantic_contract_line`
- `semantic_eligibility`
- `semantic_provenance_boundary`
- `semantic_provenance_ref`

Boundline may also consume these additive optional facts when Canon publishes
them on a compatible contract line:

- `semantic_labels`
- `semantic_exclusion_reason`

## Consumer Rules

- Canon semantic metadata is optional enrichment, not authoritative runtime
  state.
- Boundline may derive local fragments or local embeddings from Canon content,
  but every derived fragment must remain traceable to Canon's semantic
  provenance boundary and provenance reference.
- Boundline must ignore Canon artifacts that are excluded from the semantic
  contract even if the same artifacts remain indexable under the older V1
  indexing contract.
- Boundline must treat unsupported contract lines, missing required semantic
  metadata, or incompatible provenance boundaries as skip conditions rather
  than partially accepting the artifact.
- Boundline must preserve whether a Canon candidate expanded the V1 set,
  reranked it, or was rejected after compatibility checks.

## Projection Requirements

When Boundline surfaces a Canon-backed semantic candidate, the projection must
preserve:

- Canon artifact class
- semantic contract line
- semantic provenance boundary
- semantic provenance reference
- whether the candidate expanded or reranked the V1 set
- why the candidate was selected, downgraded, rejected, or skipped

When a Canon artifact is skipped, the runtime should distinguish at least these
causes when they apply:

- artifact excluded by Canon semantic policy
- unsupported semantic contract line
- missing required semantic metadata
- local policy disabled semantic acceleration
- local accelerator unavailable or degraded

## Explicit Exclusions

Canon does not choose, and this consumer contract does not accept, Canon-owned
control over any of the following Boundline runtime behaviors:

- fragment size or chunking strategy
- embedding generation policy
- vector storage ownership
- hybrid ranking or reranking policy
- fallback thresholds
- stop semantics
- remote transmission policy for this slice
