# Rails And Laravel Guidance

## Purpose

This guidance defines practices for Rails and Laravel applications in AI-assisted planning, implementation, testing, review, and refactoring.

It applies to product applications, admin systems, monoliths, background jobs, API backends, and full-stack web systems using Rails or Laravel.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, framework policy, architecture decision, or Canon-governed standard.

## Framework Posture

Rails and Laravel are productive frameworks with strong conventions.

Boundline should respect framework idioms instead of forcing generic layered architecture everywhere.

However, high-growth or high-governance systems need explicit ownership for:

- business workflows
- background jobs
- authorization
- data migrations
- domain boundaries
- integration contracts
- operational behavior

## Rails Guidance

Modern Rails applications may use:

- Hotwire/Turbo/Stimulus where product needs fit
- ActiveJob for background work
- ActiveRecord conventions
- service objects or domain objects when workflows outgrow models/controllers
- strong parameters and validation at boundaries

Guardians should watch for:

- fat controllers
- fat models containing unrelated policies
- callbacks hiding critical business workflows
- background jobs without idempotency
- migrations without rollback or expand/contract plan
- authorization scattered across controllers, views, and models
- N+1 queries in critical paths

## Laravel Guidance

Modern Laravel applications may use:

- Livewire/Volt where repository conventions allow
- Form Requests for validation
- Policies for authorization
- Jobs and queues for background work
- Eloquent conventions
- Pest or PHPUnit for tests

Guardians should watch for:

- controllers with business policy
- Eloquent models becoming global domain dumping grounds
- observers/events hiding critical side effects
- queued jobs without idempotency
- validation duplicated across Form Requests and services
- authorization checks missing or scattered
- migrations that assume no rollback or compatibility concerns

## Monolith Guidance

A monolith is not automatically a problem.

A monolith becomes risky when:

- capabilities lack ownership
- domain boundaries are invisible
- every model can touch every other model
- background workflows are hidden in callbacks
- deployments become risky due to shared state
- tests only verify framework happy paths

Use modularity inside the monolith before splitting services.

## Background Jobs

Background jobs must address:

- idempotency
- retries
- poison jobs
- dead-letter behavior
- observability
- progress tracking
- cancellation or stop criteria where applicable
- ownership

AI-generated jobs often miss idempotency and partial failure handling.

## Authorization

Authorization should be explicit and testable.

Check:
- ownership rules
- role/permission rules
- tenant isolation
- admin bypass paths
- policy coverage
- tests for negative cases

Avoid:
- authorization hidden only in views
- controllers that rely on UI to prevent unauthorized operations
- trusting client-provided ownership identifiers

## Database Migrations

Migrations must be treated as operational changes.

Check:
- lock risk
- data volume
- backward compatibility
- rollback or compensation
- expand/contract sequencing
- default value safety
- backfill monitoring
- deployment order

## Testing Guidance

Rails:
- model tests for domain rules
- request/system tests for behavior
- job tests for idempotency and retry
- factory discipline
- avoid over-mocking ActiveRecord behavior

Laravel:
- feature tests for user flows/API behavior
- policy tests
- job tests
- Pest or PHPUnit according to repo standard
- database isolation

Avoid:
- brittle callback-only tests
- tests that assert framework internals
- fixtures with hidden coupling
- no negative authorization tests

## Anti-Patterns

- business logic in controllers
- model callbacks with critical hidden workflows
- background jobs without idempotency
- migration without rollback/compatibility thought
- authorization only in views
- duplicated validation ownership
- N+1 in critical list/detail pages
- god models
- service objects that only hide unclear procedural code
- tests that only cover happy-path framework behavior

## Guardian Hooks

Recommended guardians:
- rails-laravel-controller-boundary-guardian
- active-record-domain-leakage-guardian
- callback-hidden-workflow-guardian
- queue-job-idempotency-guardian
- authorization-policy-guardian
- migration-safety-guardian
- n-plus-one-risk-guardian

## Structured Finding Example

```json
{
  "guardian": "callback-hidden-workflow",
  "rule": "critical-side-effect-in-model-callback",
  "disposition": "warning",
  "summary": "Order fulfillment publishes an external notification from a model callback, hiding an operational workflow behind persistence.",
  "evidence_refs": ["app/models/order.rb"],
  "recommended_action": "Move the workflow into an explicit application service or job with idempotency and observability."
}
```

## Lifecycle Usage

Planning:
- identify whether work touches authorization, migration, jobs, or domain workflow

Architecture:
- decide whether framework convention is sufficient or explicit boundary is needed

Implementation:
- guide controller/model/job placement

Testing:
- verify authorization, job idempotency, and migration safety

Review:
- check callbacks, god models, N+1, hidden workflows, and migration risk

Refactoring:
- extract workflows without breaking framework conventions
