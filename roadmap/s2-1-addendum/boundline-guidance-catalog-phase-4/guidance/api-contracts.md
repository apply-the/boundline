# API And Integration Contract Guidance

## Purpose

This guidance defines expectations for public APIs, internal service contracts, event schemas, message payloads, command/query boundaries, and integration contracts.

It applies to architecture, implementation, review, verification, migration, and incident work.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, API governance, architecture decision, or Canon-governed contract evidence.

## Core Thesis

Contracts are expensive to change.

AI-generated code can make contract changes look local, but consumers make them systemic.

Contract guidance exists to protect:

- compatibility
- consumer trust
- versioning
- error semantics
- data ownership
- integration boundaries
- rollback safety

## Contract Types

Contracts include:

- HTTP APIs
- GraphQL schemas
- RPC contracts
- event schemas
- message payloads
- database schemas exposed to consumers
- CLI output formats
- file formats
- webhook payloads
- tool invocation schemas
- LLM structured outputs where consumers depend on shape

## Compatibility

Before changing a contract, classify:

- additive
- backward compatible
- backward incompatible
- behavior-changing
- semantic change without shape change
- error semantics change
- performance/limit change
- security/authorization change

Shape compatibility is not enough.

Changing meaning without changing schema can still break consumers.

## Versioning

Use versioning strategy appropriate to contract type.

Options:

- URI version
- header version
- schema version
- event type version
- feature negotiation
- compatibility window
- consumer opt-in
- deprecation period

Do not create a new version unless there is an adoption and sunset plan.

## Error Contracts

Error shape is part of API contract.

Define:

- stable error code
- safe message
- validation details
- correlation ID
- retryability
- authorization failure behavior
- rate limit behavior

Avoid:

- raw exception messages
- inconsistent error formats
- provider errors leaking through
- ambiguous 500 for expected failures
- changing error codes without notice

## Event Contracts

Events should represent meaningful facts.

Check:

- event name
- event version
- producer
- owner
- required fields
- optional fields
- ordering assumptions
- idempotency key
- replay behavior
- schema registry where applicable
- consumer list or discovery mechanism

Avoid event changes without consumer impact analysis.

## Consumer Awareness

For non-private contracts, know:

- who consumes it?
- what versions exist?
- what compatibility window applies?
- how will consumers migrate?
- how will breakage be detected?
- what telemetry shows adoption?

Unknown consumers increase risk.

## Contract Testing

Use contract tests where appropriate.

Options:

- Pact
- schema validation
- consumer-driven contracts
- golden files
- compatibility test suites
- replay tests for events

Do not rely only on provider unit tests for public contract safety.

## AI-Assisted Delivery Risks

AI-generated code often:

- changes response shape while updating only local tests
- removes fields that look unused
- changes error shape
- modifies event payload without consumer analysis
- introduces new required fields
- changes nullability
- changes enum semantics
- assumes all consumers update at once

Guardians should challenge all public contract changes.

## Anti-Patterns

- removing field without compatibility window
- adding required field without default
- changing enum meaning silently
- inconsistent error shape
- raw provider error leaked as API response
- event schema changed without consumer analysis
- no contract tests
- version created without deprecation plan
- private database schema used as integration contract without ownership
- LLM/tool output schema changed without validation update

## Guardian Hooks

Recommended guardians:

- api-contract-compatibility-guardian
- error-contract-guardian
- event-schema-compatibility-guardian
- consumer-impact-guardian
- versioning-policy-guardian
- contract-test-guardian
- llm-tool-contract-guardian
- schema-compatibility-guardian

## Structured Finding Example

```json
{
  "guardian": "api-contract-compatibility",
  "rule": "new-required-field-without-compatibility-plan",
  "disposition": "warning",
  "summary": "The API response adds a required `region` field without documenting consumer compatibility or default behavior.",
  "evidence_refs": ["openapi/accounts.yaml"],
  "recommended_action": "Make the field optional during compatibility window or document versioning and consumer migration plan."
}
```

## Lifecycle Usage

Architecture:
- identify contract ownership and versioning strategy

Implementation:
- preserve compatibility and error semantics

Testing:
- add contract and compatibility tests

Review:
- challenge consumer impact and behavior-changing contracts

Verification:
- compare claimed compatibility to contract evidence

Migration:
- sequence contract changes with adoption and deprecation
