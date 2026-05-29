# S15 - Review Councils And Role-Gated Governance

## Owner

Boundline, using Canon governance vocabulary

## Status

B-level, after S2.1 guidance and guardian findings are operational

## Strategic Role

This feature makes review credible.

Boundline should not copy swarms. It should implement bounded, role-gated councils that are visible, cost-bounded, and traceable.

## Problem

A single model planning and self-approving is not enough for high-risk work.

Need structured review for:

- red/yellow authority zones
- security-sensitive changes
- public contracts
- migrations
- domain invariants
- architecture boundaries
- large refactors

## Core Scope

- Council profiles
- Role-gated review
- Reviewer capability matching
- Guardian finding intake
- Voting/adjudication
- Human gate states
- Council rejection reasons
- Cost controls
- Trace-visible decisions

## Council Profiles

Examples:

### Green

- no council by default
- optional reviewer
- guardians advisory

### Yellow

- required reviewer
- relevant guardian checks
- majority or adjudicator policy
- human confirmation for blockers

### Red

- multiple reviewers
- security/domain/architecture roles as needed
- required human gate
- no self-approval
- strict evidence requirements

## Algorithms And Techniques

### Reviewer Matching

Match reviewers using:

- lifecycle phase
- changed files
- guidance pillars
- risk classification
- authority zone
- Canon packet references
- previous findings

### Voting Methods

Support several simple algorithms:

- unanimous approval for red blockers
- majority vote for normal yellow review
- weighted vote by role authority
- adjudicator model for tie or conflict
- dissent preservation even when approved

### Finding Aggregation

Normalize:

- guardian findings
- reviewer comments
- provider findings
- Canon evidence gaps

Group by:

- severity
- affected surface
- guidance source
- lifecycle phase
- required action

### Producer Response Protocol

A plan or implementation author must respond to findings:

- accept
- reject with rationale
- defer with owner/date
- ask for clarification
- change plan

## Acceptance Criteria

- Boundline can create council from risk/zone.
- Council decision is inspectable.
- Findings are grouped and deduplicated.
- Voting/adjudication rule is trace-visible.
- Red zone cannot self-approve.
- Human gate state is explicit.
- Council cost is bounded.
- Rejected findings remain visible.

## Risks

- Councils become slow and expensive.
- Review theater without useful findings.
- Voting hides minority dissent.
- Too many roles for low-risk work.

## Hard Rule

Councils exist to improve judgment, not to simulate a meeting.
