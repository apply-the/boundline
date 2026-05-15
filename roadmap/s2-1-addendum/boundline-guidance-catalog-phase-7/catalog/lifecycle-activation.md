# Lifecycle Activation Model

## Purpose

This document defines when guidance and guardians should activate during Boundline delivery flows.

## Lifecycle Phases

Supported lifecycle labels:

```text
planning
system-shaping
architecture
backlog
implementation
testing
verification
review
refactor
migration
incident
supply-chain-analysis
```

## Guidance Activation

Guidance shapes work before or during action.

Examples:

| Phase | Guidance Examples |
|---|---|
| planning | clean-code, testing-core, architecture |
| architecture | architecture, resilience, observability |
| implementation | language, framework, clean-code |
| testing | testing-core, framework testing |
| review | all relevant active guidance |
| refactor | clean-code, architecture, testing |
| migration | migration safety, resilience, operations |
| incident | observability, operations readiness |

## Guardian Activation

Guardians verify after a relevant action or before crossing a boundary.

Examples:

| Phase | Guardian Examples |
|---|---|
| planning | testability, architecture-risk |
| architecture | architecture-boundary, contract-stability |
| implementation | language idiom, clean-code, framework boundary |
| testing | testability, brittle-mock, coverage-validity |
| review | all applicable guardians |
| migration | rollback-safety, compatibility, resilience |
| incident | observability, runbook-readiness |

## Activation Policy

S2.1 activates guardians based on manifest declarations and available context.

S3 decides whether findings influence councils or stop semantics.

S4 decides whether governance runs in advisory, rule, or hook mode.

## No Always-On Rule

Guidance can be loaded frequently.

Guardians should activate only when relevant to:

- changed files
- lifecycle phase
- selected expert
- active risk
- workspace policy
- Canon-governed context
- task goal
