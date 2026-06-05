# Evals And Runtime Observability

## Integration Update

This roadmap item should absorb **Trace Compaction Policy**.

Trace compaction belongs here because this file owns event vocabulary, JSONL export, trace export, runtime metrics, eval fixtures, and observability.

Trace compaction does not belong to `16-session-memory-and-repository-knowledge-distillation.md`. Memory proposals are reviewed knowledge; compaction is trace hygiene.

## Relationship To Other Roadmap Files

| Related file | Relationship |
|---|---|
| `05-plan-analysis-contract.md` | Should be covered by planning-quality evals |
| `specs/070-large-codebase-context-substrate/spec.md` | Needs context-selection evals and context-pack metrics |
| `specs/071-capability-provider-protocol/spec.md` | Needs provider-call events and provider eval fixtures |
| `10-review-councils-and-role-gated-governance.md` | Needs council and guardian-finding evals |
| `14-ai-gateway-and-inference-economics.md` | Owns cost policy but depends on event and route telemetry |
| `16-session-memory-and-repository-knowledge-distillation.md` | Consumes trace refs for memory proposals; does not own trace compaction |

## Added Scope

Add trace retention and compaction classes:

- lossless
- structured
- summary
- index-only
- discardable

## Trace Compaction Classes

### Lossless

Must remain exact.

Examples:

- accepted decisions
- approvals
- final stage outputs
- rejection reasons
- operator answers
- contract validation results
- evidence references
- release validation results

Rules:

- never destructively compact
- never replace with summary only
- may be indexed, but exact record remains available

### Structured

Can be normalized into structured event records.

Examples:

- guardian findings
- provider findings
- test summaries
- lint summaries
- phase requests
- route decisions
- context selection records

Rules:

- store as structured event records
- preserve source references
- include reproducibility metadata where relevant

### Summary

Can be summarized with source references.

Examples:

- long assistant transcripts
- repeated troubleshooting attempts
- old implementation attempts
- verbose command logs after key evidence is extracted

Rules:

- source references must remain available
- summary must not become authority for completion
- lossy summaries must be marked as lossy

### Index-Only

Can be represented by searchable metadata.

Examples:

- old context packets
- stale trace fragments
- obsolete intermediate drafts

Rules:

- usable for retrieval/navigation only
- not sufficient for edit, approval, or completion decisions

### Discardable

Can be removed under retention policy.

Examples:

- duplicate generated output
- temporary debug dumps
- abandoned local diagnostics without decisions

Rules:

- never discard active stage evidence
- never discard rejection reasons
- record compaction action

## Required Event

Every compaction should emit an event:

```json
{
  "event_type": "trace.compacted",
  "policy": "trace-compaction-v1",
  "source_trace": "trace://abc",
  "actions": [
    {
      "item_ref": "assistant-transcript-1",
      "from": "raw",
      "to": "summary",
      "lossy": true
    }
  ],
  "preserved_refs": ["decision-12", "finding-22"]
}
```

## Minimal Evals Additions

Add evals for:

- context selection quality
- critical-context omission
- guardian finding quality
- council rejection behavior
- provider call failure handling
- trace compaction survival of accepted decisions
- trace compaction survival of rejection reasons

## Metrics Additions

Record:

- compaction count
- compaction class distribution
- trace size before/after compaction
- lossy compaction count
- preserved decision count
- preserved rejection count
- context size
- context item count
- provider latency
- stop reason
- finding count

## Acceptance Criteria Additions

- Evals can run locally.
- Evals can run in CI.
- Runtime emits structured events.
- Trace export can feed dashboards and analysis.
- Trace compaction policy exists with lossless, structured, summary, index-only, and discardable classes.
- Accepted decisions and rejection reasons survive compaction.
- Lossy summaries are marked as lossy.
- Source references remain available.
- Compaction emits trace-visible events.
- Active stage evidence is never destructively compacted.

## Risks

- LLM-as-judge becomes untrusted theater.
- Metrics become vanity dashboards.
- Eval corpus is too small or too clean.
- Observability leaks sensitive data.
- Compaction hides forensic detail.

## Hard Rules

- If a feature changes AI behavior, it must have an eval path.
- Compaction must never destroy accepted decisions, rejection reasons, or required evidence.
