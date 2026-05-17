# Canon Semantic Retrieval Consumer Contract

## Purpose

Define the minimum Canon-owned artifact indexing surface that Boundline may
consume for Advanced Context Intelligence without shifting authority for
runtime context, retrieval policy, or delivery decisions away from Boundline.

This contract is consumer-side only. Canon remains the producer and semantic
owner of published artifact metadata through its existing artifact-indexing
surfaces; Boundline remains the retrieval orchestrator that decides how local
retrieval, ranking, inference, and explainability work.

## Required Canon Baseline

For the first S5 slice, Boundline relies on Canon's existing artifact-indexing
baseline rather than requiring a new Canon feature.

The authoritative producer-side references are:

- Canon `051-artifact-indexing-contract`
- Canon `docs/integration/project-memory-promotion-contract.md`

Boundline may treat Canon artifacts as semantically indexable only when Canon
declares them part of that stable indexing surface.

## Minimum Canon Metadata Boundline May Consume

For a Canon artifact to participate in advanced retrieval, Boundline must be
able to recover these minimum facts from Canon's documented metadata carrier:

- `artifact_class`
- `contract_line`
- producer attribution or owner identity
- `source_reference`
- the normative metadata carrier location that makes the artifact discoverable

Boundline may consume additional additive Canon metadata when Canon publishes
it on a compatible contract line, but that metadata is optional for this slice.

## Consumer Rules

- Canon artifacts are optional enrichment, not authoritative runtime state.
- Boundline may tokenize, index, cluster, or locally derive relationships from
  compatible Canon artifacts, but it must preserve provenance back to the
  originating Canon artifact and contract line.
- Boundline must ignore Canon artifacts that are outside the documented stable
  indexing surface instead of inferring support from surrounding prose.
- Boundline must treat unsupported contract lines, missing required metadata,
  or incompatible metadata carriers as retrieval unavailability for that
  artifact rather than partially indexing it.
- Canon metadata may influence discoverability and provenance, but it does not
  choose ranking policy, retrieval depth, reviewer inference policy, impact
  thresholds, stop semantics, or execution flow inside Boundline.

## Remote Transmission Rules

- Canon-backed content may only participate in remote semantic retrieval when
  the active workspace explicitly enables remote mode.
- When remote mode is disabled or policy-blocked, Boundline must keep Canon
  artifact processing local or exclude the Canon artifact from semantic
  expansion.
- Boundline must surface when Canon-backed content was excluded from remote
  enrichment because transmission was not permitted.

## Projection Rules

When Boundline uses Canon artifacts during advanced retrieval, the runtime
surfaces must preserve:

- the Canon artifact class
- the Canon contract line
- the source reference or originating artifact identifier
- the reason the artifact was retrieved, matched, or rejected
- whether the artifact was used in local-only or explicit remote mode

When a Canon artifact is skipped, the runtime should surface whether the cause
was unsupported contract line, missing metadata, policy restriction,
credibility failure, or explicit operator configuration.

## Explicit Exclusions

Canon does not choose, and this consumer contract does not accept, Canon-owned
control over any of the following runtime behaviors:

- retrieval ranking policy
- graph traversal policy
- reviewer inference policy
- impact-analysis thresholds
- evidence-depth limits
- risk-escalation policy inside Boundline
- local vector or graph index ownership
- stop-transition policy
- final delivery adjudication