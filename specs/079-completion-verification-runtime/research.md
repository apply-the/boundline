# Research: Boundline Completion Verification Runtime

## Provider Catalog Refresh

Public provider documentation was rechecked on 2026-06-12 as required by the
constitution:

- OpenAI's current API models documentation still exposes the GPT-5.5 and
  GPT-5.4 families already represented in
  `assistant/catalog/model-catalog.toml`, including the `Latest: GPT-5.5`
  entry on the current models page:
  [OpenAI models](https://developers.openai.com/api/docs/models).
- Anthropic's current Claude models overview still documents the current Opus,
  Sonnet, and Haiku families already represented in the bundled catalog:
  [Claude models overview](https://platform.claude.com/docs/en/about-claude/models/overview).
- Google's current Gemini models page still documents the Gemini 2.5 and 3.x
  families already represented in the bundled catalog, including Gemini 2.5
  Pro / Flash / Flash-Lite and Gemini 3.x family entries:
  [Gemini models](https://ai.google.dev/gemini-api/docs/models).

Result: no feature-driven catalog change is required for this planning packet.
The bundled `assistant/catalog/model-catalog.toml` already covers the coding-
relevant OpenAI, Anthropic, and Gemini families this feature depends on for
assistant guidance and route references.

## Decision 1: Keep proof ownership at task scope and aggregate upward

**Decision**: Treat task-level claims as the authoritative proof unit in the
first slice. Stage and run closeout aggregate child verification readiness and
surface unresolved child findings instead of inventing a new independent proof
path unless an explicit parent claim is declared.

**Rationale**: The spec explicitly prioritizes one claim-matched proof command
per claimed task outcome. Reusing task proof as the unit of truth keeps the
slice sequential, inspectable, and consistent with the existing task lifecycle.
Parent aggregation provides safe closeout without duplicating proof semantics.

**Alternatives considered**:

- Always require a new parent-scope proof: rejected because it duplicates child
  verification and expands the slice unnecessarily.
- Ignore child verification once a parent closes: rejected because it hides the
  actual blocking conditions and violates observability.

## Decision 2: Use explicit-claim-first resolution with bounded runtime inference

**Decision**: Resolve completion claims using explicit task or stage metadata
first, then bounded runtime inference from the completion action, task title,
description, changed files, declared outputs, selected proof command, and
recent execution trace when metadata is absent.

**Rationale**: Existing tasks may not yet carry structured completion-claim
metadata, so inference is necessary for backward compatibility. Making the
inferred claim explicit before proof selection preserves inspectability and
keeps proof selection subordinate to the claim rather than the reverse.

**Alternatives considered**:

- Metadata only: rejected because the first slice must support existing tasks.
- Derive the claim from the proof command alone: rejected because that reverses
  the proof contract and makes the runtime validate the wrong thing.
- Always ask the operator: rejected because it adds too much friction for the
  normal closeout path.

## Decision 3: Represent stale proof as a blocking finding, not a new state

**Decision**: Keep the top-level `completion_verification_state` vocabulary
limited to `ready`, `proof_required`, `blocked`, and `failed`. Model stale
proof as a structured finding (`kind = stale_proof`) attached to `blocked` or
`proof_required`.

**Rationale**: The public projection stays smaller and backward-compatible when
staleness is a reason rather than a fifth state. This still makes stale proof
explicit, testable, and actionable through the findings contract.

**Alternatives considered**:

- Add a dedicated `stale` state: rejected because it expands the public state
  contract for a condition that is already expressible as a blocking reason.
- Treat stale proof as failure: rejected because stale proof means rerun is
  required, not that the proof itself failed.

## Decision 4: Use normalized workspace content fingerprints for freshness

**Decision**: Record a normalized workspace content fingerprint immediately
before and after each proof run, then treat proof as fresh only while the
current fingerprint matches the fingerprint captured for the most recent
passing proof.

**Rationale**: Freshness must be conservative in the first slice because
untracked edits can still affect code execution, generated output, and runtime
behavior. Excluding Boundline-owned runtime artifacts and configured volatile
paths avoids self-invalidating proofs while keeping the safety boundary broad.

**Alternatives considered**:

- Fingerprint tracked files only: rejected because untracked workspace content
  can still affect the claim.
- Invalidate only when specific claim-related files change: rejected because the
  runtime does not yet have sufficiently precise dependency knowledge.
- Timestamp-only freshness: rejected because timestamps are too weak and do not
  explain why proof became stale.

## Decision 5: Make claim confirmation policy-driven and risk-aware

**Decision**: Proceed automatically only for a single high-confidence inferred
claim with no conflicts or risky surfaces. Require confirmation or override
when confidence is low, multiple plausible claims remain, proof coverage is
partial, risky surfaces are involved, or metadata conflicts with runtime
signals.

**Rationale**: The runtime must stay low-friction for routine closeout but
cannot silently prove the wrong thing. A confidence-and-risk gate is the
smallest slice that balances usability and correctness.

**Alternatives considered**:

- Always require confirmation: rejected because it would slow every closeout.
- Never require confirmation: rejected because ambiguous or risky claims would
  become unsafe.
- Confirmation only for one claim family: rejected because ambiguity and risk
  are cross-cutting, not limited to a single claim type.

## Decision 6: Define one structured projection contract for all closeout surfaces

**Decision**: Use one additive completion-verification projection model that
feeds `status`, `inspect`, and `orchestrate`, with scope-specific fields for
task, stage, and run aggregation.

**Rationale**: The operator already relies on those surfaces to decide what to
do next. A single typed projection keeps field meanings aligned across CLI
rendering, JSON surfaces, and tests while avoiding divergent per-command
contracts.

**Alternatives considered**:

- Separate per-command view models: rejected because it would duplicate finding
  semantics and increase drift risk.
- Evidence-only representation with no status projection: rejected because
  blocked closeout must be visible before the operator inspects raw traces.
