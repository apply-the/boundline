# Architecture Guidance

## Purpose

This guidance defines architecture-level rules for bounded AI-assisted delivery.

It should shape system-shaping, architecture, backlog, implementation, refactor, review, migration, and verification work.

## Authority Classification

Default strength: recommended  
May become mandatory when promoted by Canon architecture packets, ADRs, or workspace policy.

## Core Architecture Principles

### Boundaries Before Code

Before implementation, identify:

- capability boundary
- ownership boundary
- data boundary
- integration boundary
- deployment boundary
- operational boundary

AI-generated implementation often jumps directly into code. Boundline should force boundary visibility before high-impact changes proceed.

### Capability Ownership

A capability should have an accountable owner and a clear reason to exist.

Weak signals:
- a service exists only because a model generated it
- multiple services own the same concept
- data ownership is split without explicit coordination
- public API exposes internal domain shape

### Dependency Direction

Dependencies should point inward toward stable domain concepts and outward toward replaceable adapters.

For hexagonal or clean architecture styles:
- domain must not depend on adapters
- application services orchestrate use cases
- infrastructure implements ports
- presentation maps external input/output

### Contract Stability

Public contracts are Type 1 decisions.

Treat as high-impact:
- public APIs
- event schemas
- database schemas
- authentication/authorization boundaries
- integration payloads
- domain ownership contracts

These require stronger review than local implementation changes.

### Architecture Decision Records

Architecture decisions should capture:

- context
- decision
- alternatives
- tradeoffs
- consequences
- reversibility
- validation plan

AI-generated architecture that lacks rationale should not be treated as accepted architecture.

### Reversibility Classification

Classify decisions before implementation:

Type 2:
- local helpers
- private implementation details
- isolated UI behavior
- replaceable internal code

Type 1:
- schema design
- API contracts
- service boundaries
- data ownership
- authentication/authorization models
- cross-team integrations
- migration paths

Type 1 changes require stronger governance and evidence.

### System Coherence

Local optimization must not break system coherence.

Common AI-induced failures:
- duplicate abstractions
- new service boundary without ownership rationale
- bypassing existing domain language
- introducing a second pattern for the same problem
- hidden coupling through shared database access
- ad hoc integration instead of owned contracts

### Operational Fitness

Architecture must account for:

- deployment
- observability
- rollback
- compatibility
- failure modes
- scaling assumptions
- data migration
- incident containment

Architecture that only describes static structure is incomplete.

## Guardian Hooks

Recommended guardians:
- architecture-boundary-guardian
- dependency-direction-guardian
- contract-stability-guardian
- type-one-decision-guardian
- operational-fitness-guardian

## Structured Finding Examples

```json
{
  "guardian": "architecture-boundary",
  "rule": "data-ownership",
  "disposition": "blocker",
  "summary": "The change introduces writes to a database table owned by another capability.",
  "evidence_refs": ["src/billing/repository.rs", "docs/project/domain-model.md"],
  "recommended_action": "Introduce an integration contract or move ownership explicitly before implementation."
}
```

## Lifecycle Usage

System-shaping:
- identify capability boundaries before architecture

Architecture:
- validate decisions, alternatives, and reversibility

Backlog:
- decompose architecture into governed delivery slices

Implementation:
- ensure code follows accepted boundaries

Review:
- check boundary leakage and contract drift

Migration:
- validate sequencing, fallback, and compatibility
