# Architecture

Architecture-level rules for bounded delivery: capability ownership, dependency direction, contract stability, and operational fitness.

## Core Principles

### Boundaries Before Code

Before implementation, identify: capability boundary, ownership boundary, data boundary, integration boundary, deployment boundary, operational boundary.

AI-generated implementation often jumps directly into code. Architecture visibility must precede high-impact changes.

### Capability Ownership

A capability should have an accountable owner and a clear reason to exist.

Weak signals:
- A service exists only because a model generated it.
- Multiple services own the same concept.
- Data ownership is split without explicit coordination.
- Public API exposes internal domain shape.

### Dependency Direction

Dependencies should point inward toward stable domain concepts and outward toward replaceable adapters.

For hexagonal or clean architecture styles:
- Domain must not depend on adapters.
- Application services orchestrate use cases.
- Infrastructure implements ports.
- Presentation maps external input/output.

### Contract Stability

Public contracts are Type 1 decisions. Treat as high-impact:
- Public APIs and event schemas
- Database schemas
- Authentication and authorization boundaries
- Integration payloads
- Domain ownership contracts

These require stronger review than local implementation changes.

### Reversibility Classification

Type 2 (reversible): local helpers, private implementation details, isolated UI behavior, replaceable internal code.

Type 1 (hard to reverse): schema design, API contracts, service boundaries, data ownership, authentication models, cross-team integrations, migration paths.

Type 1 changes require stronger governance and evidence.

### System Coherence

Local optimization must not break system coherence.

Common AI-induced failures:
- Duplicate abstractions for the same concept
- New service boundary without ownership rationale
- Bypassing existing domain language
- Introducing a second pattern for the same problem
- Hidden coupling through shared database access
- Ad hoc integration instead of owned contracts

### Operational Fitness

Architecture must account for deployment, observability, rollback, compatibility, failure modes, scaling assumptions, data migration, and incident containment.

Architecture that only describes static structure is incomplete.

### Architecture Decision Records

Architecture decisions should capture: context, decision, alternatives, tradeoffs, consequences, reversibility, and validation plan.

AI-generated architecture that lacks rationale should not be treated as accepted architecture.

## Anti-Patterns

- Jumping to implementation without identifying boundaries
- Services that exist because generation was easy, not because they represent a coherent capability
- Domain types that depend on framework, HTTP, or persistence details
- Broad default permissions or fail-open authorization at architecture level
- Changes to public contracts without compatibility analysis
- Silent coupling through shared database tables or implicit event dependencies
- Architecture proposals without reversibility or rollback analysis
- AI-generated microservice splits without communication overhead assessment

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction, data-ownership-boundary, public-contract-stability
- `clean_code`: no-mixed-responsibilities (when architecture concerns leak into wrong layer)
- `security_boundary`: authorization checks at boundaries
- `operations_readiness`: deployment and rollback coverage for architecture changes
