# S11 - Evals And Runtime Observability

## Owner

Boundline and Canon

## Status

Required early, before advanced autonomy

## Speckit Seed Notes

- Seed role: measurement and trace substrate for later AI behavior changes.
- First slice: add a stable local event schema, JSONL export, and a tiny golden
  eval corpus for planning gates and context selection.
- Depends on: existing trace surfaces; can run before provider protocol if it
  starts with runtime-owned events only.
- De-duplication: this seed owns event vocabulary and eval fixtures; gateway
  cost policy belongs to seed 14, and provider-specific artifacts belong to the
  concrete provider seeds.

## Strategic Role

This feature makes quality measurable.

Without evals and observability, every model change, prompt change, council change, and guardian change is a blind release.

## Problem

Canon and Boundline aspire to excellence, but excellence needs measurement.

Current risks:

- no regression suite for plan quality
- no evals for context selection
- no evals for guardian findings
- no evals for Canon packet quality
- no runtime metrics for costs, stops, findings, latency
- no structured trace export for dashboards or analysis

## Scope Split

### Boundline Owns

- runtime trace metrics
- plan-quality evals
- context-selection evals
- guardian-finding evals
- stop-semantics evals
- council-decision evals
- provider protocol evals
- dashboard metrics

### Canon Owns

- packet quality evals
- mode document completeness evals
- evidence quality evals
- approval/readiness consistency evals
- lineage validation evals
- project memory promotion evals

## Minimal Evals

### Boundline

- Can the system classify task risk?
- Can it build a Context Pack without overloading the model?
- Can it reject missing context when needed?
- Can guardians catch known bad patterns?
- Can review councils reject unsafe plans?
- Can stop semantics trigger correctly?

### Canon

- Can each mode produce required ordered documents?
- Does the packet separate claims from evidence?
- Does readiness match evidence?
- Is lineage complete?
- Are approval states consistent?
- Is promoted project memory traceable?

## Observability Events

Required event families:

- session lifecycle
- context retrieval
- plan generation
- confirmation
- run execution
- guardian execution
- finding emission
- stop rule trigger
- provider call
- council review
- Canon packet generation
- Canon promotion
- error and recovery

## Metrics

### Runtime Metrics

- run duration
- step duration
- model/provider route
- context size
- context item count
- tokens/cost when available
- stop reason
- findings count
- guardian outcomes
- review outcomes
- provider latency
- recovery count

### Governance Metrics

- packet completeness
- evidence count
- unresolved uncertainty count
- approval state
- lineage completeness
- promotion state
- project memory update count

## Suggested Technology

Start simple:

- JSONL trace export
- stable event schema
- local eval fixtures
- snapshot tests
- golden output comparison
- deterministic scoring rules where possible

Then add:

- OpenTelemetry-compatible export
- Langfuse or similar optional sink
- promptfoo or custom eval runner for LLM-as-judge
- dashboard aggregation
- CI regression jobs

## Eval Corpus

Create a small but high-value golden corpus:

- tiny implementation
- unsafe auth change
- migration without rollback
- API breaking change
- large file edit
- missing tests
- domain-language drift
- Canon architecture packet
- Canon verification packet
- review council rejection case

## Acceptance Criteria

- Evals can run locally.
- Evals can run in CI.
- Each eval has expected outcome and failure explanation.
- Runtime emits structured events.
- Trace export can feed S8.
- Model/provider changes can be compared.
- Canon packet quality can regress visibly.

## Risks

- LLM-as-judge becomes untrusted theater.
- Metrics become vanity dashboards.
- Eval corpus is too small or too clean.
- Observability leaks sensitive data.

## Hard Rule

If a feature changes AI behavior, it must have an eval path.
