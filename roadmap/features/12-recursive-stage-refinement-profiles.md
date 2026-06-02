# S22 - Recursive Stage Refinement Profiles

## Owner

Boundline

## Status

Later, after council and adaptive-governance hardening

## Speckit Seed Notes

- Seed role: bounded, inspectable stage-refinement loops over the existing
  session-native runtime.
- First slice: add one opt-in sequential refinement profile for planning with
  `planner -> critic -> planner -> finalizer`, compact structured round packets,
  a hard round limit, no-progress detection, blocker stops, and trace-visible
  outcomes.
- Depends on: stable event and eval substrate from seed 08, council hardening
  from seed 10, and adaptive-governance calibration from seed 11.
- De-duplication: council review and finding resolution stay in seed 10;
  confidence, degradation, and escalation policy stay in seed 11; route cost
  policy stays in seed 14; this seed owns bounded recursive stage movement only.

## Inspiration And Boundary

The RecursiveMAS paper explores collaboration loops in which heterogeneous
agents exchange latent states through trained RecursiveLink modules and decode
text only at the final round:

- paper: `https://arxiv.org/abs/2604.25917`
- project: `https://recursivemas.github.io/`

Boundline should adopt the architectural lesson, not the ML mechanism.

Boundline does not own model hidden states, gradient training, or local
checkpoint execution. Its useful analogue is an inspectable stage loop that
passes compact structured state between bounded roles while preserving runtime
authority, stop semantics, and operator control.

The local `sqlite-vec` retrieval index remains derived retrieval infrastructure.
It must not become an authoritative hidden-state store or a communication bus
for opaque agent thoughts.

## Strategic Role

This feature lets selected delivery stages improve through controlled
refinement without turning Boundline into an open-ended agent debate runtime.

The runtime should make repeated collaboration understandable:

```text
stage candidate
  -> structured critique
  -> bounded revision delta
  -> closure check
  -> final stage outcome or explicit stop
```

## Problem

Some planning and review stages need more than one pass, but naive multi-agent
chat introduces avoidable cost and weakens inspectability:

- intermediate transcripts become large and repetitive
- roles can loop without measurable progress
- blockers may be discussed rather than enforced
- operators cannot tell why another round started
- stage ownership becomes ambiguous
- route cost and elapsed time can grow without a hard boundary

## Core Principle

Recursive refinement is stage movement, not hidden autonomy.

Boundline owns:

- whether a refinement profile may activate
- the active stage and round budget
- compact structured round packets
- findings and revision deltas
- stop, degradation, and escalation decisions
- trace persistence and operator projections
- the final authoritative stage outcome

Roles and providers may propose candidates, critiques, and deltas. They do not
approve their own recursion depth or bypass runtime stop rules.

## First Slice

Start with one opt-in sequential planning profile:

```text
planner -> critic -> planner -> finalizer
```

The first slice should:

- activate only for a configured planning stage
- reuse existing session, trace, finding, and stop-semantics surfaces
- persist one compact packet per round
- require a hard `max_rounds`
- stop when no material delta remains
- stop or escalate when a blocking finding remains unresolved
- publish one final stage artifact or one explicit incomplete outcome
- expose the active profile, current round, stop reason, and next action in
  `status`, `next`, and `inspect`

## Structured Round Packet

The packet is an inspectable runtime record, not a latent thought:

```json
{
  "profile": "plan_refinement",
  "stage": "plan",
  "round": 2,
  "candidate_ref": "trace://plan-candidate-2",
  "findings": [],
  "requested_deltas": [],
  "applied_deltas": [],
  "confidence": "sufficient",
  "stop_reason": null
}
```

Packets should remain compact and reference artifacts instead of copying full
transcripts or source files into every round.

## Activation And Stop Policy

The runtime should evaluate profile activation before the first recursive
round. It should not start a loop merely because multiple roles are available.

Required boundaries:

- profile is explicitly enabled for the stage
- stage ownership is already resolved
- maximum rounds are configured
- maximum elapsed time is configured
- maximum route cost is enforced when cost telemetry is available
- unresolved blockers stop or escalate according to the active governance
  posture
- no material improvement ends refinement
- malformed or missing packets fail visibly
- provider failure follows the existing claimed-stage failure boundary

## Later Expansion

After the sequential planning profile proves useful, later slices may evaluate:

- implementation refinement
- architecture decision refinement
- bounded specialist aggregation
- deliberation between a reflector and a tool-calling provider
- adapter-owned stages that opt into the same host-governed refinement contract

Each expansion needs its own eval evidence and must preserve the same hard
boundaries.

## Explicitly Out Of Scope

- model hidden-state transfer
- trained RecursiveLink modules
- gradient-based credit assignment
- use of `sqlite-vec` as an agent communication channel
- unbounded autonomous debate
- self-selected recursion depth without host limits
- multi-adapter recursive composition
- new council voting or calibration engines
- a second session or trace store

## Acceptance Criteria

- An operator can enable one bounded sequential planning-refinement profile.
- Every refinement round produces one compact trace-linked packet.
- The runtime stops at the configured round limit.
- The runtime stops when no material improvement remains.
- An unresolved blocker cannot silently become a successful final outcome.
- `status`, `next`, and `inspect` explain profile activation, current round,
  findings, stop reason, and final outcome.
- The feature reuses existing runtime-owned session, council, and stop
  semantics instead of creating a parallel orchestration system.
- Tests prove that the feature remains useful without `sqlite-vec`.

## Risks

- Recursive refinement becomes prompt theater.
- Extra rounds increase cost without improving artifacts.
- Compact packets hide context that reviewers need.
- A loop duplicates council or adaptive-governance behavior.
- Providers leak opaque state into authoritative runtime decisions.

## Hard Rule

Every recursive round must earn its existence through an inspectable delta.
