# Authority And Strength Model

## Purpose

This document defines how guidance and guardian rules express authority and strength.

The goal is to avoid treating every best practice as a hard rule.

## Authority Source

Every resolved guidance or guardian capability must expose its authority source.

Supported sources:

```text
runtime-evidence
workspace-override
canon-governed
shared-pack
boundline-built-in
```

## Resolution Strength

Resolution strength, highest to lowest:

```text
runtime evidence for the active task
workspace overrides
Canon-governed standards
shared expert packs
Boundline built-ins
```

Canon-governed standards are the highest external governed authority.

Workspace overrides may override Canon-governed standards only as local repository policy, and that override must be trace-visible.

## Guidance Strength

Supported strength values:

```text
mandatory
recommended
legacy-warning
target-excellence
anti-pattern
deprecated
```

## Guardian Disposition

Supported dispositions:

```text
info
observation
concern
warning
risk
blocker
error
```

## Strength To Disposition Mapping

Default mapping:

| Guidance Strength | Default Guardian Disposition |
|---|---|
| mandatory | warning or blocker |
| recommended | concern |
| legacy-warning | observation or concern |
| target-excellence | observation |
| anti-pattern | warning |
| deprecated | warning or blocker |

S3 and S4 may strengthen or weaken operational consequences based on:

- authority zone
- change class
- evidence quality
- risk
- governance maturity
- council policy

## Important Rule

Text content alone must not decide operational severity.

Severity is derived from:

```text
guidance strength
+ authority source
+ lifecycle phase
+ affected surface
+ risk posture
+ governance policy
```
