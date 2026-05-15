# Operations Readiness Guidance

## Purpose

This guidance defines operational readiness expectations for AI-assisted delivery.

It applies to features, services, migrations, incidents, release changes, jobs, and production-affecting work.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, operations policy, release policy, or Canon-governed standard.

## Core Thesis

Code is not done when it works.

Code is done when it can be operated, diagnosed, rolled back, and owned.

Operations readiness asks:

- who owns this in production?
- what happens when it fails?
- how do we know it is failing?
- how do we roll it back?
- how do we contain blast radius?
- how do we recover?
- what evidence supports readiness?

## Ownership

Every production-facing capability should have:
- owner
- escalation path
- operational contact or team
- known dashboards/logs/traces
- runbook or operating notes
- release/rollback owner if high risk

Avoid:
- orphaned systems
- ownership hidden in tribal knowledge
- production behavior without support path

## Runbooks

Runbooks should exist for important operational flows.

A useful runbook includes:
- symptoms
- detection signals
- likely causes
- immediate containment
- diagnosis steps
- rollback steps
- escalation path
- links to dashboards/logs/traces
- known false positives
- verification after recovery

A runbook that only describes deployment is not enough.

## Alertability

A system should alert on user-impacting or business-impacting failure, not every internal event.

Good alerts:
- actionable
- owned
- severity-classified
- tied to symptoms or SLOs
- include diagnosis links

Bad alerts:
- noisy
- unactionable
- no owner
- no threshold rationale
- no runbook
- duplicate symptoms

## SLO / SLI Thinking

For critical systems, define:
- service level indicators
- service level objectives
- error budget assumptions
- latency expectations
- availability expectations
- freshness expectations where relevant

Not every feature needs formal SLOs, but every critical feature needs operational expectations.

## Rollback Readiness

Before release, know:
- can this be rolled back?
- does rollback require data migration?
- are writes backward compatible?
- are clients forward/backward compatible?
- does rollback preserve data integrity?
- is feature flag rollback available?
- is the previous version compatible with new data?

Rollback is especially important for:
- schema changes
- migrations
- public APIs
- auth changes
- message schemas
- integration contracts

## Feature Flags

Feature flags can reduce risk when used well.

Good uses:
- progressive rollout
- kill switch
- A/B testing with ownership
- migration cutover control

Bad uses:
- permanent undocumented branches
- flags without cleanup owner
- flags that bypass security or consistency
- flags that create untestable combinatorics

## Incident Readiness

For high-risk work, define:
- expected failure modes
- containment plan
- rollback plan
- telemetry
- customer impact
- escalation path
- post-release monitoring window

## Batch Jobs And Workers

Operational concerns:
- idempotency
- retry behavior
- partial failure
- poison messages
- dead-letter queues
- progress tracking
- checkpointing
- replay safety
- backpressure
- cancellation

AI-generated worker code often ignores partial failure and replay safety.

## Migration Readiness

Migrations require:
- dry-run strategy
- compatibility plan
- data validation
- backfill monitoring
- rollback or compensation plan
- owner
- stop criteria
- success criteria

## Anti-Patterns

- no owner
- no rollback plan
- no runbook for critical feature
- no alert for user-impacting failure
- alerts without owner
- feature flag without cleanup
- permanent hidden flag
- migration without dry run
- worker without replay safety
- background job without progress visibility
- release plan that assumes success only

## Guardian Hooks

Recommended guardians:
- runbook-readiness-guardian
- rollback-readiness-guardian
- alertability-guardian
- ownership-guardian
- feature-flag-lifecycle-guardian
- worker-operability-guardian
- migration-readiness-guardian

## Structured Finding Example

```json
{
  "guardian": "rollback-readiness",
  "rule": "schema-change-without-rollback-plan",
  "disposition": "warning",
  "summary": "The change adds a new required column but does not document backward compatibility or rollback behavior.",
  "evidence_refs": ["migrations/202605_add_required_customer_ref.sql"],
  "recommended_action": "Use expand/contract sequencing or document why rollback is not required."
}
```

## Lifecycle Usage

Planning:
- identify operational ownership, rollout, and rollback needs

Architecture:
- define operability, telemetry, and ownership boundaries

Implementation:
- add runtime controls and observability hooks

Testing:
- test operational failure modes where feasible

Review:
- verify runbook, alertability, rollback, and flag lifecycle

Incident:
- use runbook and telemetry to guide containment and recovery

Migration:
- verify dry run, stop criteria, and compatibility
