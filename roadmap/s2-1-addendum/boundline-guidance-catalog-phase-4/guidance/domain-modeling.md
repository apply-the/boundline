# Domain Modeling Guidance

## Purpose

This guidance defines how Boundline should protect domain concepts, relationships, invariants, and feature-impact rules during AI-assisted delivery.

It applies to system-shaping, domain-model, architecture, backlog, implementation, review, refactor, migration, and verification work.

## Authority Classification

Default strength: recommended  
Canon-governed domain-model packets may make specific concepts, relationships, and invariants mandatory for a project area.

## Core Thesis

Domain modeling is not database modeling.

A domain model describes:

- concepts
- relationships
- invariants
- ownership
- lifecycle
- capabilities
- events
- allowed transitions
- feature impact rules

AI-generated code often jumps from vague requirements to data structures and services without stabilizing domain meaning.

## Concepts

A concept should define:

- name
- definition
- identity
- lifecycle
- owner
- related concepts
- allowed states
- invalid states
- examples
- non-examples

## Entity vs Value Object

Entities have continuity and identity.

Value objects are defined by their attributes.

Use value objects for:

- money
- quantities
- date ranges
- identifiers
- addresses where identity is irrelevant
- permissions
- thresholds

Avoid primitive obsession.

## Aggregates

Aggregates protect invariants.

An aggregate boundary should define:

- root
- internal entities/value objects
- invariants
- allowed commands
- emitted events
- persistence boundary
- concurrency assumptions

Do not split invariants across multiple aggregates without explicit coordination.

## Bounded Contexts

A bounded context defines where a model has a specific meaning.

Check:

- does the term mean the same thing here?
- who owns the concept?
- what translations are needed?
- where does integration occur?
- which team/system owns data?
- which contracts cross the boundary?

## Domain Invariants

Invariants are rules that must remain true.

Examples:

- a refresh token can be used only once after rotation
- an invoice cannot be paid twice
- a tenant cannot access another tenant's resources
- a shipment cannot be dispatched before payment authorization
- a user cannot approve their own high-value transfer

Invariants require evidence:

- type model
- validation
- transaction boundary
- tests
- authorization check
- database constraint
- event sequencing

## Domain Events

Domain events describe meaningful facts that occurred.

Good domain events:

- use domain language
- represent completed facts
- are stable enough for consumers
- include necessary identity and context
- do not expose internal implementation state

Avoid events that are just CRUD notifications unless that is the actual contract.

## Anti-Corruption Layer

Use anti-corruption mapping when integrating with external systems or contexts with different language/model.

Do not let external provider shapes rewrite the internal domain model.

## Feature Impact Rules

Domain model should help answer:

- which concepts are affected?
- which invariants are at risk?
- which tests should exist?
- which APIs/events may change?
- which reviewers are required?
- which migration risks exist?

This is where domain modeling becomes operational for Boundline.

## AI-Assisted Delivery Risks

AI often:

- creates database tables before concepts
- invents services without ownership
- uses primitives for domain concepts
- duplicates existing concepts under new names
- ignores invariants
- puts all behavior in application services
- treats external provider schema as internal model
- creates events that leak implementation

Guardians should challenge these patterns.

## Anti-Patterns

- primitive obsession
- concept without owner
- aggregate without invariant
- invariant enforced only in UI
- bounded context leakage
- external provider model used as internal model
- event name not in domain language
- table-first modeling
- service boundary without capability ownership
- duplicated domain concepts
- domain model not reflected in tests

## Guardian Hooks

Recommended guardians:

- domain-invariant-guardian
- aggregate-boundary-guardian
- bounded-context-guardian
- primitive-domain-modeling-guardian
- domain-event-guardian
- anti-corruption-layer-guardian
- feature-impact-guardian
- domain-test-coverage-guardian

## Structured Finding Example

```json
{
  "guardian": "domain-invariant",
  "rule": "invariant-enforced-only-in-ui",
  "disposition": "blocker",
  "summary": "The high-value transfer approval rule is enforced only in the frontend and is absent from the server-side command handler.",
  "evidence_refs": ["src/ui/TransferApproval.tsx", "src/transfers/ApproveTransferHandler.cs"],
  "recommended_action": "Move the invariant to the authoritative domain/application boundary and add negative authorization tests."
}
```

## Lifecycle Usage

System-shaping:
- identify capabilities and ownership

Domain-model:
- define concepts, relationships, invariants, and impact rules

Architecture:
- map domain model to boundaries and contracts

Backlog:
- decompose delivery slices by invariant and capability

Implementation:
- preserve invariants in code, tests, and contracts

Review:
- check model drift, primitive obsession, and invariant gaps

Refactor:
- improve model clarity without changing behavior

Verification:
- compare claims to invariant evidence
