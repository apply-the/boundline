# Canon Promotion Notes

## Purpose

This document explains how guidance catalog content may become Canon-governed knowledge.

## Canon Role

Canon may promote selected guidance into:

```text
docs/project/
docs/evidence/
docs/integration/
```

depending on project memory and promotion profiles.

Canon-governed guidance becomes the highest external governed authority for Boundline resolution.

## Promotion Candidates

Good candidates for Canon promotion:

- project-specific clean code rules
- architecture standards
- domain modeling standards
- language policies adopted by the team
- testing strategy
- security boundaries
- supply-chain policy
- operations readiness policy

## What Canon Should Not Own

Canon should not own:

- guardian execution
- tool invocation
- linter timeouts
- LLM routing
- runtime error handling
- workspace override mechanics

## Promotion Metadata

A Canon-promoted guidance artifact should preserve:

```text
source
version
promotion_state
authority
scope
applicable phases
owner
review status
```

## Boundline Runtime Behavior

When Boundline resolves Canon-promoted guidance, it must:

- disclose Canon as authority source
- preserve lineage reference where available
- allow trace-visible workspace override
- avoid treating Canon absence as runtime failure
