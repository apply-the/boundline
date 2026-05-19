# Contract: Delight Projection And Inspect Closure

## Scope

This contract governs the user-facing behavior for the S7.1 follow-through
surfaces:

- profile-aware explanation surfaces such as `challenge`, `hidden-impact`, and
  `explain-plan`
- named inspect closures for `context`, `council`, and `timeline`

The contract is about projection semantics and operator-visible behavior. The
final selector shape may be a flag, named view, or subcommand, but the external
view names and disclosure guarantees are stable.

## Authoritative Inputs

| Input | Authority | Required Behavior |
| --- | --- | --- |
| Active session state | `.boundline/session.json` | Remains the source of truth for current workspace state and next-step context. |
| Latest or explicit trace | `.boundline/traces/` | Supplies the flattened evidence needed for inspect closure and delight projection. |
| Reasoning profile activation | `TraceSummaryView.reasoning_profile` and session governance lifecycle | Explains whether advanced reasoning was active and what it contributed. |
| Governance and review evidence | `governance_timeline`, `review_timeline`, approval provenance, terminal reason | Must remain visible when it affects the answer. |

## Shared Output Invariants

- Every delight or inspect closure response MUST identify the authoritative
  state source it used.
- Every response MUST distinguish runtime evidence, governed evidence when
  present, and missing or degraded evidence.
- When a reasoning profile is active, the output MUST disclose the profile,
  selection rationale, and profile-specific contribution.
- When a reasoning profile is missing, degraded, or incompatible, the output
  MUST disclose the fallback condition explicitly.
- Human-facing projections MUST not require raw trace JSON as the primary
  operator surface.
- Terminal or blocked state MUST stay visible; the projection must not flatten a
  non-success outcome into a success narrative.

## Explanation Surface Requirements

### Profile-Aware Disclosure

- `challenge`, `hidden-impact`, and `explain-plan` MUST disclose whether a
  reasoning profile is active for the current answer.
- Active-profile output MUST include:
  - profile identity
  - selection rationale
  - what changed because the profile was active
- Fallback output MUST include:
  - why the advanced reasoning path was unavailable or degraded
  - what bounded fallback path was used instead

### Source Attribution

- Explanation surfaces MUST preserve source-attribution buckets rather than
  merging them into one confidence claim.
- Missing Canon or weak context state MUST stay visible instead of being
  inferred away.

## Inspect Closure Requirements

### Inspect Context

- MUST explain context assembly in operator language.
- MUST surface provenance, credibility, and weak-context or stale-context
  signals.
- MUST identify what additional evidence would make the answer stronger when
  context is incomplete.

### Inspect Council

- MUST explain whether review or council activity was used, skipped, or
  unavailable.
- MUST preserve any authority or review boundary already present in the trace.
- If no council activity occurred, the response MUST say so explicitly rather
  than treating the view as an error.

### Inspect Timeline

- MUST preserve the order of decision, review, governance, step, and recovery
  events.
- MUST preserve the authoritative terminal status and terminal reason.
- MUST explain recovery attempts and non-success outcomes without flattening the
  sequence into a success-only story.

## Failure Contract

- Missing trace reference MUST return corrective guidance, not an empty view.
- Missing latest trace MUST preserve the current inspect failure guidance.
- Empty evidence for a specific view MUST return an explicit missing-state or
  fallback disclosure, not a silent omission.
- Unsupported advanced reasoning MUST still return a bounded answer when the
  base runtime evidence is sufficient.