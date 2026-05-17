# Advanced Context Intelligence Projection Contract

## Purpose

Define the minimum operator-facing projection surfaces for Advanced Context
Intelligence so retrieved evidence, relationship projections, impact findings,
and degradation reasons remain visible on the normal Boundline runtime path.

## Session And Compact Surface Requirements

When advanced context intelligence is evaluated for an active decision point,
Boundline must keep the following fields or equivalent structured projection
available to compact runtime surfaces:

- `retrieval_mode: disabled|local|remote`
- `retrieval_state: selected|degraded|insufficient|exhausted|unavailable`
- `retrieval_authority_order: structured>canon>workspace_override>semantic`
- `retrieval_index_state: ready|stale|building|insufficient`
- `retrieval_terminal_reason: <reason>` when the state is not `selected`
- `selected_evidence_count: <count>` when evidence was selected
- `impact_finding_count: <count>` when impact analysis produced findings

These compact projections must be available on the primary session-native path.
Any compatibility-route use must remain explicitly labeled as secondary.

## Inspect Surface Requirements

`inspect` must make it possible to answer all of the following without reading
source code:

- why a retrieved item was selected or rejected
- whether the item came from structured runtime context, a local repository
  artifact, a trace, a review finding, verification evidence, or a Canon
  artifact
- whether a Canon artifact was skipped because of incompatible metadata or
  policy restriction
- which projected relationships support an impact finding
- which limits, stale-refresh events, or remote-policy boundaries influenced the result

Minimum detailed projection lines or equivalent structured output:

- `retrieval_mode`
- `retrieval_state`
- `retrieval_terminal_reason`
- `selected_evidence[*].source_kind`
- `selected_evidence[*].source_ref`
- `selected_evidence[*].authority_rank`
- `selected_evidence[*].selection_reason`
- `relationships[*].relationship_kind`
- `relationships[*].credibility_state`
- `relationships[*].explanation`
- `impact_findings[*].finding_kind`
- `impact_findings[*].recommended_follow_up`

## Trace Projection Requirements

Trace output must preserve a step-by-step story of retrieval behavior.
Minimum trace events or equivalent typed trace records:

- retrieval started
- retrieval refreshed because prior evidence was stale
- candidate selected
- candidate downgraded or rejected
- relationship projected
- impact finding recorded
- Canon artifact skipped because of incompatibility
- remote transmission blocked or not enabled
- retrieval degraded, insufficient, or exhausted

Each trace record must preserve enough provenance to connect the event back to
the active retrieval query and supporting evidence.

## Failure Projection Rules

When advanced context intelligence does not produce a selected result, the
runtime must make the failure mode explicit:

- `degraded`: retrieval continued in a lower-confidence or structured-only path
- `insufficient`: available evidence was not credible enough to support the
  requested projection
- `exhausted`: configured retrieval limits were reached
- `unavailable`: required local or remote capability was not available for the
  requested mode

The projection must never imply that semantic expansion succeeded when the
runtime actually fell back to a weaker path.

## Explicit Exclusions

This contract does not require:

- a new UI surface outside the existing CLI and trace projections
- hidden ranking logic without surfaced rationale
- Canon-owned control over retrieval ranking, impact thresholds, or stop policy
- remote-provider-specific output when remote mode is not enabled