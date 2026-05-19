# Reasoning Profile Algorithms

Boundline `0.63.0` ships reasoning profiles inside the existing session-native
runtime. Canon can require a challenge posture, but Boundline still owns
profile selection, participant assignment, independence checking, bounded
outcomes, trace emission, and operator-facing projection.

This document describes the current runtime algorithm, not an aspirational
future reasoning system.

## 1. Activation Contract

Reasoning activation starts from one typed `ReasoningProfileDefinition` and one
typed trigger.

The current trigger sources are:

- Canon required challenge
- stage-governance escalation
- operator policy
- local fixture activation for deterministic tests

Each profile definition carries:

- one concrete profile id and family
- allowed Canon-governed stages
- a bounded budget for participants, calls, tokens, and optional reflexion or adjudication steps
- participant roles with route preferences and independence requirements
- an adjudication mode
- a degradation policy that decides whether reduced independence can degrade or must block

## 2. Common Runtime Algorithm

Every shipped reasoning profile follows the same runtime-owned skeleton:

1. resolve the effective routing for the workspace
2. select participant roles from the profile definition, taking all required roles first and then any optional roles until the bounded participant budget is reached
3. map each role onto an effective route and provider family
4. aggregate the requested independence floor from the declared role requirements
5. measure observed distinctness across routes, providers, contexts, and prompting patterns
6. classify independence as `passed`, `degraded`, or `failed`
7. derive the bounded outcome for the profile when independence is sufficient, or a degraded or blocked outcome when it is not
8. derive one typed confidence contribution and admission effect
9. persist the activation record into session state and additive trace events so `run`, `status`, and `inspect` tell the same story

The important architectural rule is that reasoning does not create a second
workflow. It stays inside the normal stage lifecycle.

## 3. Independence Assessment

Independence is not inferred from agent count alone. Boundline checks whether
the selected participants satisfy the floor requested by the profile.

The current observed dimensions are:

- distinct routes
- distinct provider families
- distinct context bases
- distinct prompting patterns

The result is interpreted as:

- `passed`: the requested participant count and all requested distinctness dimensions are satisfied
- `degraded`: the profile can continue only when the degradation policy explicitly allows the missing participant or distinctness conditions
- `failed`: the profile must block and surface the remediation path

When the runtime is Canon-governed, a passed result produces a higher
confidence contribution than the same topology under local governance. A failed
result always gates progression.

## 4. Shipped Profile Inventory

### 4.1 `bounded_self_consistency`

This is the single-participant shipped profile.

Current runtime shape:

- family: `self_consistency`
- minimum participants: `1`
- independence posture: no multi-party distinctness requirement
- bounded behavior: branch exploration stays limited by the declared reasoning budget
- shipped-status note: `0.63.0` keeps the concrete shipped claim inherited from the earlier contract slice; the follow-through release extends the same reasoning story into explanation and inspect surfaces rather than promoting a second orchestration path

### 4.2 `independent_pair_review`

This is the concrete blind-review pair profile.

Current runtime shape:

- family: `blind_review`
- required roles: two blind reviewers
- independence posture: distinct route and provider families, with at least two participants
- adjudication mode: governance review
- bounded success shape: when independence passes, the profile completes with an `adjudicated` outcome
- bounded failure shape: when reviewer routes collapse, the profile blocks and tells the operator to configure distinct reviewer routes

### 4.3 `heterogeneous_security_review`

This is the shipped heterogeneous review profile.

Current runtime shape:

- family: `heterogeneous_review`
- required roles: heterogeneous reviewers resolved onto distinct review routes when available
- independence posture: distinct route and provider families, with at least two participants
- bounded success shape: when independence passes, the profile completes with a `converged` outcome and an approval-ready security-review summary
- bounded failure shape: when heterogeneity cannot be established, the runtime degrades or blocks according to the profile policy instead of silently pretending consensus

### 4.4 `bounded_reflexion`

This is the shipped critique-and-revise profile.

Current runtime shape:

- family: `reflexion`
- required roles: critic and reviser
- independence posture: the profile can run with one participant minimum at the family level, but it still records role-specific prompting patterns and route choices
- bounded success shape: the current success path records one `reflexion_revision` iteration where a critic challenges the candidate change and a reviser produces a bounded revision
- bounded failure shape: interruption, degradation, or exhaustion remain explicit terminal states rather than hidden retries

## 5. Supported But Not Shipped As Standalone Profiles

### 5.1 Debate

Debate is intentionally classified as `bounded_substrate`.

That means:

- the domain and trace model can represent debate-oriented rounds
- bounded debate vocabulary can support other reasoning or review flows
- Boundline `0.63.0` does not claim a standalone shipped V1 debate profile id

### 5.2 Adjudication

Adjudication is intentionally classified as `shared_primitive`.

That means:

- it can appear as an arbiter role, an adjudication step, or a terminal disagreement-resolution outcome
- it remains reusable across reasoning and review flows
- Boundline `0.63.0` does not claim adjudication as a standalone shipped reasoning profile

## 6. Operator Reading Guide

When a reasoning profile is active, read the operator surfaces in this order:

1. profile id and trigger: what challenge shape was activated and why
2. independence result: whether the participant topology is actually credible
3. outcome kind: whether the profile converged, adjudicated, degraded, blocked, or was interrupted
4. confidence contribution: how much the stage should trust the challenge result
5. next action and fallback disclosure: what the operator must fix when the profile could not complete credibly and what the runtime disclosed on `status`, `inspect`, `why`, `risk`, `evidence`, `next-best`, `challenge`, or `explain-plan`

That ordering keeps the runtime honest. A profile is not trustworthy just
because it ran; it is trustworthy when the bounded topology, outcome, and
confidence story line up.

## 7. Literature References

These references are influences for the bounded runtime patterns above, not a
claim that Boundline implements any paper verbatim.

- Wang, X., Wei, J., Schuurmans, D., Le, Q., Chi, E., Narang, S., Chowdhery, A., and Zhou, D. "Self-Consistency Improves Chain of Thought Reasoning in Language Models." ICLR 2023. Motivates bounded alternative reasoning-path selection for self-consistency.
- Shinn, N., Cassano, F., Berman, E., Gopinath, A., Narasimhan, K., and Yao, S. "Reflexion: Language Agents with Verbal Reinforcement Learning." 2023. Motivates bounded critique-and-revise loops rather than hidden unbounded retries.
- Du, Y., Li, S., Torralba, A., Tenenbaum, J. B., and Mordatch, I. "Improving Factuality and Reasoning in Language Models through Multiagent Debate." 2023. Motivates debate as a bounded coordination substrate rather than a guarantee of correctness by agent count alone.
- Dietterich, T. G. "Ensemble Methods in Machine Learning." 2000. Motivates the importance of diversity and error decorrelation for heterogeneous review rather than naive duplication of the same route.