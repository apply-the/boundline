# Phase 4 Manifest — Security, Domain, Contracts, And Migration

## Included Guidance

```text
guidance/security-boundaries.md
guidance/domain-language.md
guidance/domain-modeling.md
guidance/api-contracts.md
guidance/migration-safety.md
guardians/security-domain-compatibility-guardian-rule-seeds.md
```

## Pillars Covered

- authentication
- authorization
- tenant isolation
- secret handling
- PII handling
- auditability
- ubiquitous language
- bounded contexts
- aggregates
- domain invariants
- domain events
- anti-corruption layers
- public API compatibility
- event schema compatibility
- database schema migration
- expand/contract sequencing
- rollback and compensation
- migration blast radius

## Suggested Pack Mapping

```toml
[guidance.security_boundaries]
path = "guidance/security-boundaries.md"
applies_to = ["planning", "architecture", "implementation", "review", "verification", "security-assessment"]

[guidance.domain_language]
path = "guidance/domain-language.md"
applies_to = ["discovery", "requirements", "domain-language", "architecture", "implementation", "review", "refactor"]

[guidance.domain_modeling]
path = "guidance/domain-modeling.md"
applies_to = ["system-shaping", "domain-model", "architecture", "backlog", "implementation", "review", "refactor"]

[guidance.api_contracts]
path = "guidance/api-contracts.md"
applies_to = ["architecture", "implementation", "review", "verification", "migration"]

[guidance.migration_safety]
path = "guidance/migration-safety.md"
applies_to = ["planning", "architecture", "migration", "implementation", "review", "verification"]
```

## Suggested Guardians

```toml
[guardians.security_boundary]
rules = ["authn-boundary", "authz-boundary", "tenant-isolation", "secret-handling", "pii-flow"]

[guardians.domain_language]
rules = ["term-drift", "ambiguous-term", "new-term-without-definition", "implementation-term-in-domain-language"]

[guardians.domain_model]
rules = ["invariant-preservation", "aggregate-boundary", "bounded-context-leakage", "primitive-domain-modeling"]

[guardians.api_contract]
rules = ["breaking-api-change", "error-shape-drift", "schema-compatibility", "consumer-unknown"]

[guardians.migration_safety]
rules = ["expand-contract-missing", "rollback-missing", "dual-write-risk", "backfill-monitoring-missing"]
```

## Authority Strength

Default strength: recommendation.

Rules may become warning, blocker, or mandatory when promoted by:

- workspace override
- Canon-governed standard
- security policy
- architecture decision
- API contract policy
- S3/S4 governance posture
