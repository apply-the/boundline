# Ontology (Ubiquitous Language)

This page defines the shared vocabulary for Boundline. Consistent use of these terms ensures clarity between operators, documentation, and the runtime.

## Core Entities

### Bounded Session
The active delivery container for a declared goal. It is "bounded" because it has an explicit scope, context, and end-state. It prevents the AI from drifting into unrelated tasks.

### Context Pack
The complete set of information Boundline uses to plan or execute. Unlike a generic "prompt," a context pack is a structured bundle of files, briefs, validation results, and historical traces.

### Runtime State
The local workspace data (stored in `.boundline/`) that tracks the progress of a session. It is the single source of truth for the "current" state of work, independent of chat history.

## Governance & Logic

### Guidance
Proactive rules and practices provided to the worker **before** or **during** action.
*Example: "Always use TypeScript strict mode."*

### Guardian
Reactive validation logic that checks work **after** action or at quality gates. Guardians emit structured findings.
*Example: "Ensure no secrets were committed in the last run."*

### Finding
A structured report from a Guardian. Unlike a text comment, a Finding has a severity (Blocker, Risk, Warning, Observation) and a specific scope.

### Trace
A durable, inspectable record of a specific decision or action taken by the runtime. Traces allow an operator to audit *why* a certain path was chosen.

### Inspect Closure
A human-facing projection built from authoritative trace state, such as `inspect_context`, `inspect_council`, or `inspect_timeline`.

## Integration & Extension

### Expert Pack
A reusable package of expertise (prompts, guidance, guardians) tailored for a specific stack, domain, or role (e.g., "React Expert," "Security Reviewer").

### Canon
The external system of record for governed knowledge, approvals, and lineage. Boundline consumes Canon knowledge but does not own the governance authority.

### Project Memory
Durable knowledge about a project (architecture decisions, business logic, governed evidence) that survives across multiple sessions, often sourced from Canon-promoted `docs/project/` or `docs/evidence/` material.

### Review Council
A bounded governance review assembly selected from the active authority zone and runtime posture. Councils add structured independent review without turning the product into an unbounded orchestration graph.

### Reasoning Profile
A bounded runtime challenge pattern that activates when governance posture or stage policy requires stronger challenge. Current shipped examples include `bounded_self_consistency`, `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion`.

### Delight Feedback Signal
A lightweight session-scoped usefulness metric such as time to first useful answer, explanation attribution rate, next-action acceptance rate, or latest next-action outcome.

### Support Mode
The declared host parity posture for an assistant surface. Current values include `repo-local-full`, `copy-ready-assets`, and `manual-fallback`.

### Compatibility Window
The explicit release pair that declares which Boundline and Canon versions can consume the same governed contract surface without manual interpretation.
