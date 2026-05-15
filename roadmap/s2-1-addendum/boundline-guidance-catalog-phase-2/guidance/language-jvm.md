# JVM And Java Guidance

## Purpose

This guidance defines idiomatic JVM and Java practices for AI-assisted planning, architecture, implementation, testing, review, and refactoring.

It is intended for Java services, libraries, backend systems, and JVM-based applications.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, expert pack configuration, or Canon-governed standards.

## Version Posture

Active support window:
- Java 11+

Target excellence:
- Java 21 LTS where repository constraints allow
- virtual threads where they simplify blocking I/O workloads
- sealed classes for constrained hierarchies
- pattern matching where it improves clarity
- modern switch expressions

Legacy warnings:
- Java 8 in active development
- broad checked exception hierarchies for expected business flow
- heavy thread management where modern concurrency tools fit better
- deep inheritance hierarchies
- static utility sprawl

## Core Design Principles

### Model Domain States Explicitly

Use types to model constrained domain states.

Good:
- enums for stable finite states
- sealed interfaces/classes for closed variants
- records for immutable data carriers
- value objects for domain identifiers and quantities

Avoid:
- stringly typed state
- raw `Map<String, Object>` as domain data
- boolean flag combinations that represent hidden states

### Error Handling

Avoid using checked exceptions for expected business decisions.

Prefer:
- typed domain result objects
- sealed error hierarchies
- explicit validation results
- exception handling for exceptional infrastructure failures

For libraries:
- expose stable error semantics
- avoid leaking framework exceptions as public contract

### Optional Usage

Use `Optional` primarily as a return type to indicate absence.

Avoid:
- Optional fields
- Optional parameters
- nested Optional
- using Optional as a serialization shape

### Virtual Threads

Virtual threads can simplify concurrent blocking code, but do not remove the need for correct synchronization.

Avoid:
- `synchronized` blocks around long operations on virtual threads
- assuming virtual threads fix all blocking design problems
- unbounded resource usage behind cheap thread creation

Prefer:
- structured concurrency where available
- bounded resource access
- explicit timeouts
- connection pool awareness

### Framework Boundaries

For Spring Boot, Micronaut, Quarkus, or similar frameworks:

Keep:
- domain logic framework-independent
- controllers thin
- persistence isolated
- transaction boundaries explicit
- configuration validated

Avoid:
- business policy in controllers
- framework annotations across domain model
- service classes that only pass through to repositories
- entity classes as public API DTOs

### Resilience

For distributed systems:
- use timeouts
- circuit breakers
- bulkheads
- retries with budgets
- idempotency where retrying writes

Libraries:
- Resilience4j
- Failsafe

Do not add retries without understanding idempotency.

### Functional Helpers

Libraries like Vavr can be useful, but do not introduce functional abstractions that the team cannot maintain.

Use:
- Either/Try/Validation when they clarify expected failure
- pattern matching or sealed types for domain state

Avoid:
- wrapping every operation in functional abstractions without design payoff

## Testing Guidance

Recommended:
- JUnit 5
- AssertJ
- parameterized tests
- Testcontainers for integration where appropriate
- contract tests for service boundaries

Avoid:
- brittle mocks of internal implementation
- `Thread.sleep`
- hidden shared database state
- tests that require execution order
- testing framework wiring instead of behavior unless wiring is the target

## Anti-Patterns

- Java 8 style in new code where modern features are available
- `Optional` as field or parameter
- checked exceptions for expected business paths
- controller-driven business logic
- leaking JPA entities as API contracts
- synchronized long-running virtual thread sections
- static utility dumping grounds
- null as domain state
- repository methods with no domain meaning

## Guardian Hooks

Recommended guardians:
- java-modernization-guardian
- java-error-modeling-guardian
- java-optional-guardian
- java-framework-boundary-guardian
- java-virtual-thread-guardian
- java-contract-boundary-guardian

## Structured Finding Example

```json
{
  "guardian": "java-framework-boundary",
  "rule": "entity-leaked-as-api-contract",
  "disposition": "concern",
  "summary": "The controller exposes a persistence entity directly as the response contract.",
  "evidence_refs": ["src/main/java/com/acme/accounts/AccountController.java"],
  "recommended_action": "Introduce an explicit response DTO and keep persistence annotations out of the public API shape."
}
```

## Lifecycle Usage

Planning:
- identify Java version and framework constraints

Architecture:
- validate module boundaries, contracts, and domain ownership

Implementation:
- guide records, sealed types, error modeling, and framework boundaries

Testing:
- choose unit, integration, or contract tests appropriately

Review:
- check entity leakage, null state, exception modeling, and virtual-thread misuse

Refactoring:
- modernize safely without mixing behavior changes and syntax migration
