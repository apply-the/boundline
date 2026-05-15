# Security Boundaries Guidance

## Purpose

This guidance defines security boundary expectations for AI-assisted delivery.

It applies to authentication, authorization, tenant isolation, secret handling, PII handling, audit logging, session/token lifecycle, security-sensitive integrations, and privileged operations.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, security policy, Canon security assessment, or governed architecture decision.

## Core Thesis

Security is a boundary problem before it is an implementation detail.

AI-generated code often appears correct locally while bypassing:

- authorization checks
- tenant isolation
- ownership validation
- secret handling
- auditability
- token/session invariants

Security guidance must be applied before implementation and verified after implementation.

## Authentication Boundary

Authentication establishes identity.

Check:

- who is the caller?
- how is identity verified?
- where is authentication performed?
- what token/session mechanism applies?
- how is token expiry handled?
- how is refresh handled?
- how is identity propagated?
- how are failed authentication attempts handled?

Avoid:

- ad hoc identity parsing
- trusting client-provided identity fields
- using authentication as authorization
- accepting unsigned or unvalidated tokens
- missing expiry validation
- refresh token rotation without replay protection

## Authorization Boundary

Authorization decides whether an authenticated identity may perform an action.

Check:

- action
- resource
- owner
- tenant
- role/permission
- policy source
- denial behavior
- audit trail

Authorization must be enforced server-side or at the authoritative boundary.

Do not rely on:

- UI hiding controls
- client-provided role fields
- route naming conventions
- inferred ownership without verification

## Tenant Isolation

Multi-tenant systems must preserve tenant boundaries.

Check:

- tenant ID source
- tenant ID validation
- database query filters
- cache keys
- object storage paths
- background jobs
- events/messages
- search indexes
- logs and metrics
- admin bypass paths

Common failures:

- tenant filter missing in one query
- tenant ID accepted from request body
- shared cache key without tenant namespace
- background job loses tenant context
- admin path accidentally exposed to normal users

## Secret Handling

Secrets include:

- API keys
- tokens
- private keys
- passwords
- connection strings
- signing secrets
- session secrets
- OAuth client secrets

Rules:

- never log secrets
- never commit secrets
- never send secrets to client bundles
- avoid secrets in command arguments
- avoid secrets in URLs
- use secret managers or repository-approved mechanisms
- rotate secrets where policy requires
- scope secrets narrowly

## PII Handling

PII and sensitive data must be classified and minimized.

Check:

- what data is collected?
- why is it needed?
- where is it stored?
- who can access it?
- how long is it retained?
- is it logged?
- is it embedded?
- is it sent to external providers?
- is it encrypted or tokenized where required?

AI and embedding systems require special caution because sensitive data may leak into prompts, logs, traces, vector stores, or external APIs.

## Token And Session Lifecycle

Token/session systems require explicit invariants.

Check:

- expiry
- refresh
- rotation
- revocation
- replay protection
- session fixation
- storage
- logout
- device/session ownership
- audit logging

Guardians should treat auth/session changes as high-risk.

## Audit Logging

Security-sensitive actions should produce audit events.

Audit events should include:

- actor
- action
- target
- outcome
- time
- source
- correlation ID
- policy decision where relevant

Do not log secrets or excessive PII.

Audit events should be stable enough for investigation and compliance.

## Secure Defaults

Default behavior should be safe.

Prefer:

- deny by default
- least privilege
- explicit allow lists
- strict validation
- short-lived credentials
- scoped tokens
- safe error messages

Avoid:

- fail-open authorization
- broad default permissions
- catch-all admin paths
- permissive CORS by default
- disabling TLS verification
- verbose security errors to users

## Security And AI-Assisted Delivery

AI-generated code often:

- adds endpoint without policy check
- trusts ownership field from client
- logs full request body
- bypasses tenant filter
- uses broad token scope
- forgets audit logging
- adds retry around non-idempotent privileged operation
- sends sensitive context to model/provider

Guardians should explicitly challenge these patterns.

## Anti-Patterns

- authentication treated as authorization
- tenant ID from request body trusted
- missing server-side ownership check
- fail-open security
- broad admin bypass
- token stored insecurely
- secret in client bundle
- PII in logs
- sensitive data embedded without policy
- audit event missing for privileged action
- disabled TLS validation
- permissive CORS without reason

## Guardian Hooks

Recommended guardians:

- security-boundary-guardian
- authn-boundary-guardian
- authz-ownership-guardian
- tenant-isolation-guardian
- secret-handling-guardian
- pii-flow-guardian
- token-lifecycle-guardian
- auditability-guardian
- secure-defaults-guardian

## Structured Finding Example

```json
{
  "guardian": "authz-ownership",
  "rule": "client-owned-resource-id-trusted",
  "disposition": "blocker",
  "summary": "The endpoint trusts the client-provided account ID without verifying that the authenticated actor owns or may access it.",
  "evidence_refs": ["src/accounts/routes.ts"],
  "recommended_action": "Resolve the account through an authorization-aware repository method or policy check using the authenticated actor."
}
```

## Lifecycle Usage

Planning:
- identify security boundaries before implementation

Architecture:
- define identity, authorization, tenant, and audit boundaries

Implementation:
- apply server-side checks, validation, secure defaults, and secret handling

Testing:
- verify negative authorization and tenant-isolation cases

Review:
- block missing security boundary evidence for high-risk changes

Verification:
- compare security claims to code, tests, logs, and policy evidence

Security-assessment:
- assess threat model, mitigations, and residual risk
