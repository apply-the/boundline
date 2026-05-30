# Boundline Goal Template

Use this template before `/boundline:goal` when the goal is still broad or when the operator needs a brief that planning can reason over without guesswork.

## Goal

- What are we trying to change?

## Intended Outcome

- What should be true when this is done?

## Success Criteria

- Which measurable user or business outcomes prove the goal is complete?
- What threshold, count, rate, or observable condition makes the outcome verifiable?

## Acceptance Scenarios

- What primary user/operator flow must work?
- What secondary flow or regression must stay working?

## Problem Domain

- What workflow, product area, or operator problem does this affect?

## Known Facts

- What is already known and verified?

## API Operations

- Which commands, endpoints, jobs, or user actions are in scope?

## Persistence Choice

- Which data store or persistence boundary is authoritative?

## Auth Boundary

- What authentication or authorization boundary matters here?

## Role Model Semantics

- Which actors can do what, and what must stay forbidden?

## Validation Target

- Which focused check proves the change is acceptable?

## Edge Cases

- Which boundary cases, errors, or conflicting inputs must be handled?

## Constraints

- What must stay bounded or unchanged?

## Unknowns

- Which facts are still missing?

## Assumptions

- Which assumptions are acceptable for now and should be called out?

## Reasonable Defaults

- Which low-impact details can Boundline infer without stopping for clarification?
- If not relevant, state that no new auth/privacy or persistence boundary is assumed.

## Goal Quality Checklist

- [ ] Bounded outcome and scope boundary are clear.
- [ ] Actors/actions/data or affected artifact are identified.
- [ ] Intended outcome is testable.
- [ ] Success criteria are measurable and technology-agnostic unless explicit validation evidence is required.
- [ ] Validation target or acceptance evidence is named.
- [ ] Security, privacy, auth, and role semantics are clarified only when materially relevant.
- [ ] Assumptions/defaults are documented.

## Done When

- Boundline can capture the goal without more than 3 prioritized clarification questions.
- Planning can proceed from runtime-confirmed goal quality rather than chat-only assumptions.

## Brief Files

- List any repo files or external documents that should be passed with `--brief <path>`.

When this is filled, record it with `boundline orchestrate --goal "<goal>" --brief <path> --until phase-request --json-stream`.
