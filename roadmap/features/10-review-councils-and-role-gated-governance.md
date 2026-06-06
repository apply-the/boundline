# Review Councils And Role-Gated Governance

## Integration Update

This roadmap item should absorb **Guardian Activation Router** as either a prerequisite or the first hardening slice.

The router must not become a separate governance engine. It only decides which guardians or review roles should activate for a stage.

Councils decide how findings are reviewed, grouped, adjudicated, and escalated.

`11-adaptive-governance-calibration.md` decides how strongly findings enforce or degrade.

## Relationship To Other Roadmap Files

| Related file | Relationship |
|---|---|
| `../specs/072-evals-runtime-observability/feat-evals-and-runtime-observability.md` | Provides event/eval substrate for council and guardian decisions |
| `11-adaptive-governance-calibration.md` | Consumes council/guardian findings to decide control level |
| `14-ai-gateway-and-inference-economics.md` | Owns route cost policy for council and guardian execution |
| `12-recursive-stage-refinement-profiles.md` | May reuse council findings but does not redefine councils |
| `18-completion-verification-runtime.md` | Provides proof-gated completion signals councils may review |

## Added Scope

Add guardian activation routing:

- activated guardians
- skipped guardians with reasons
- mandatory guardians
- optional guardians
- missing guardian capability findings
- trace-visible activation plan

## Guardian Activation Router

The router determines which guardians should run for a stage based on structured runtime signals.

### Inputs

- lifecycle phase
- changed files
- language
- framework
- risk classification
- authority zone
- active contracts
- Canon packet references
- guidance pillars
- touched architecture boundaries
- test changes
- security-sensitive files
- public API changes

### Outputs

- activated guardians
- skipped guardians with reason
- mandatory guardians
- optional guardians
- escalation recommendation
- missing guardian capability finding

## Example Rules

### Rust Runtime Change

```text
files: src/domain/**/*.rs, src/orchestrator/**/*.rs
stage: run or review
activate:
  - rust-guardian
  - error-handling-guardian
  - traceability-guardian
```

### Documentation-Only Change

```text
files: docs/**/*.md
stage: review
activate:
  - docs-consistency-guardian
  - release-surface-guardian
```

### Contract Change

```text
files: specs/**/*.md, contracts/**/*.md
stage: plan or review
activate:
  - contract-drift-guardian
  - migration-guardian
  - traceability-guardian
```

### Security-Sensitive Change

```text
files: auth, secrets, permissions, sandbox
risk: high
activate:
  - security-guardian
  - threat-model-guardian
  - approval-gate-guardian
```

## Trace Requirements

Council traces should include:

- authority zone
- active council profile
- guardian activation plan
- activated guardians
- skipped guardians with reasons
- findings
- voting or adjudication policy
- dissent
- human gate state
- final decision

## Acceptance Criteria Additions

- Boundline can activate guardians based on stage and change surface.
- Mandatory guardians cannot be silently skipped.
- Skipped guardian reasons are inspectable.
- Red-zone changes activate mandatory safety guardians.
- Documentation-only changes do not run irrelevant runtime-only guardians.
- Contract changes activate contract and migration guardians.
- Council decisions include guardian activation context.

## Risks

- Councils become slow and expensive.
- Review theater without useful findings.
- Voting hides minority dissent.
- Too many roles for low-risk work.
- Guardian router skips a needed guardian.

## Hard Rules

- Councils exist to improve judgment, not simulate a meeting.
- Guardian routing must be inspectable.
- Skipped mandatory checks must become findings, not silent omissions.
