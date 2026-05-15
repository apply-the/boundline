# Guardian Finding Contract

## Purpose

Define the structured output that every guardian execution attempt must emit so
Boundline can inspect, project, and govern the result without guesswork.

## Required Finding Fields

Every guardian execution attempt must produce either one or more structured
findings or one explicit degraded or failure record containing:
- `guardian_id`
- `rule_id`
- `disposition`
- `summary`
- `evidence_refs`
- `confidence`
- `recommended_action`
- `authority_source`
- `source_ref`
- `phase`

## Execution Outcome Rules

- `deterministic` guardians that exit non-successfully without structured output must still emit an explicit guardian-failure finding using raw exit evidence.
- `llm` or `hybrid` guardians must degrade explicitly when no suitable runtime route is available.
- A guardian may be marked `skipped` only when bounded ordering or prior blocking findings made the execution unnecessary.
- A guardian must never disappear silently from the runtime story.
- The emitted finding or failure record must be stable enough for `status`, `next`, `inspect`, and downstream governance to reuse without recomputing it.

## Disposition Rules

- `advise`, `warn`, and `concern` remain non-blocking unless downstream governance later escalates them.
- `error` and `block` remain explicit high-severity dispositions and must surface clearly in session and trace projections.
- Guardian-failure and guardian-degraded outcomes must remain distinct from a rule violation so operators can tell whether the work failed verification or the verifier failed to run.
