# Clean Code Guidance

## Purpose

This guidance defines language-agnostic clean code principles that should shape planning, implementation, refactoring, testing, and review.

It is intended for Boundline experts, implementers, reviewers, and guardians.

## Authority Classification

Default strength: recommended  
May become mandatory when promoted by Canon or overridden by workspace policy.

## Core Principles

### Intent-Revealing Names

Names must express the reason a thing exists, not merely its implementation mechanism.

Prefer:

```text
TokenRotationPolicy
InvoiceApprovalWindow
CustomerEligibilityRule
```

Avoid:

```text
Manager
Processor
Handler
Util
Data
Thing
```

A name is weak when a reviewer must inspect implementation to understand business intent.

### Small Cohesive Units

Functions, classes, modules, and services should have one coherent reason to change.

This does not mean every file must be tiny. It means responsibilities must not be mixed.

Common violations:
- persistence and business policy in the same object
- validation and side effects mixed in the same function
- HTTP transport details leaking into domain logic
- orchestration logic hidden inside low-level helpers
- UI state, API shape, and domain decisions coupled together

### Explicit Boundaries

Validate at system boundaries.

Examples:
- HTTP request input
- CLI arguments
- message bus payloads
- database read models
- third-party API responses
- AI-generated structured data

Internal code should not repeatedly defend against invalid states that should have been rejected at the boundary.

### No Magic Values In Domain Logic

Avoid unexplained numbers, strings, flags, and enum-like literals.

Prefer named values, domain-specific types, or configuration that expresses intent.

Bad:

```text
if retry_count > 3
if status == "A"
if amount > 9999
```

Better:

```text
if retry_count > MAX_AUTH_RETRY_ATTEMPTS
if status == AccountStatus.Active
if amount > HIGH_VALUE_TRANSFER_THRESHOLD
```

### Error Ownership

Errors must be owned at the correct layer.

Rules:
- domain errors should be typed and meaningful
- infrastructure errors should not leak as business semantics
- public APIs should expose stable errors
- internal implementation errors should preserve causal chain
- do not both log and return the same error unless there is a clear ownership boundary

### Log Or Return, Not Both

Logging and returning the same error at multiple layers creates duplicate noise.

Preferred pattern:
- low-level code returns typed/contextual errors
- boundary code logs once with request/session context
- observability layer attaches correlation metadata

### Primitive Obsession

Avoid raw primitives for domain concepts.

Bad:
```text
user_id: string
amount: number
country: string
```

Better:
```text
UserId
Money
CountryCode
```

This is especially important for identifiers, quantities, permissions, time windows, and ownership boundaries.

### Side Effects Must Be Visible

A function that mutates state, sends messages, writes files, calls external systems, or changes persistent data must make that behavior clear through naming and placement.

Hidden side effects are especially dangerous in AI-generated code because generated diffs often look plausible while changing operational behavior.

### Temporal Coupling Must Be Explicit

If operations must happen in a specific order, the design should make the order hard to violate.

Examples:
- state machine
- typed workflow stage
- builder that prevents invalid construction
- transaction boundary
- explicit orchestration object

### Comments Explain Why, Not What

Comments should explain:
- constraints
- decisions
- tradeoffs
- invariants
- workarounds
- domain rules

They should not narrate obvious implementation.

Bad:
```text
// Increment i by one
```

Good:
```text
// Keep this threshold below the provider timeout to preserve retry budget.
```

## Guardian Hooks

Recommended guardians:
- clean-code-guardian
- naming-intent-guardian
- responsibility-boundary-guardian
- primitive-obsession-guardian
- error-ownership-guardian

## Structured Finding Examples

```json
{
  "guardian": "clean-code",
  "rule": "responsibility-boundary",
  "disposition": "concern",
  "summary": "The new service combines persistence, policy evaluation, and transport mapping.",
  "evidence_refs": ["src/auth/token_service.rs"],
  "recommended_action": "Split persistence operations from token rotation policy."
}
```

## Lifecycle Usage

Planning:
- check if task boundaries are coherent
- detect hidden side effects early

Implementation:
- guide naming, error handling, and validation placement

Testing:
- ensure behavior is testable through public boundaries

Review:
- emit findings for unclear ownership, hidden side effects, and mixed responsibilities

Refactoring:
- guide extraction sequencing without expanding functional scope
