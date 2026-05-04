# Research: Context Selection Hardening

## Decision 1: Select workspace context from explicit evidence anchors first

- **Decision**: Replace the current planner behavior that primarily scores
  repository paths from goal keywords with an evidence-first selection pass that
  admits workspace files and artifacts only when they are backed by explicit
  anchors such as authored brief references, validation output, failing test
  targets, recent trace evidence, workflow targets, recent workspace mutations,
  or reusable Canon artifacts.
- **Rationale**: The current `select_relevant_workspace_files` and
  `file_relevance_score` path can look plausible while still selecting files for
  textual coincidence rather than causal relevance. Evidence-first selection
  makes the admission rule explainable and bounded.
- **Alternatives considered**:
  - Expand keyword lists and score weights. Rejected because it preserves the
    core problem: a file can still look credible for the wrong reason.
  - Require operators to name files manually for every plan. Rejected because
    the planner should still help narrow context from existing bounded evidence.

## Decision 2: Keep heuristics only as bounded tie-breakers

- **Decision**: Preserve lightweight path and symbol heuristics only to rank or
  trim an already evidenced candidate set; heuristics alone must not be enough
  to mark a context pack credible.
- **Rationale**: Some bounded tie-breaking is still useful when multiple files
  share the same evidence anchor, but the credible or insufficient decision must
  come from surfaced evidence rather than hidden scoring.
- **Alternatives considered**:
  - Remove all heuristics entirely. Rejected because evidence-backed candidate
    sets can still need deterministic bounded narrowing.
  - Keep heuristics as a fallback credible path. Rejected because that is the
    behavior the slice is meant to remove.

## Decision 3: Preserve the existing ContextPack model and enrich ContextInput

- **Decision**: Keep `ContextPack` and `ContextInput` as the persisted planning
  primitives, but enrich the inputs with explicit evidence-anchor semantics and
  more precise rationale text instead of introducing a second persisted
  selection-state model.
- **Rationale**: `GoalPlan`, session projections, and trace summaries already
  consume `ContextPack`. Extending the existing model keeps the authoritative
  planning state in one place and avoids duplication across planner, session,
  and inspect surfaces.
- **Alternatives considered**:
  - Add a separate `ContextBuilderState` file. Rejected because it would create
    another authority source for the same planning decision.
  - Store richer provenance only in traces. Rejected because `status`, `next`,
    and `run` need durable session-facing context before a new trace is read.

## Decision 4: Reuse existing session and trace evidence instead of building a new index

- **Decision**: Derive evidence from data Boundline already persists or can
  collect cheaply during planning, including authored brief metadata, goal-plan
  context, trace summaries, latest validation or retry signals, and current
  workflow/cluster state.
- **Rationale**: The feature needs causal context, not a general search engine.
  Reusing existing state keeps the slice bounded and aligned with the current
  delivery model.
- **Alternatives considered**:
  - Add a repository-wide semantic indexing pass. Rejected because it introduces
    a new subsystem and background complexity outside the minimal slice.
  - Depend on Canon for context selection. Rejected because Boundline must stay
    independently usable and the constitution forbids core control flow from
    depending on external systems.

## Decision 5: Make non-credible context an explicit planning outcome

- **Decision**: When explicit evidence is absent, stale, contradictory, or too
  broad, planning should persist an insufficient or stale context pack and stop
  with a bounded operator-facing recovery cue.
- **Rationale**: The planner is only trustworthy if it refuses to pretend weak
  ambient matches are credible. Existing `ContextPackCredibility` already gives
  the right state vocabulary for this stop behavior.
- **Alternatives considered**:
  - Fall back to the old keyword-ranked file set with a warning. Rejected
    because it still lets the planner act on context it does not really trust.
  - Fail without persisting any context story. Rejected because operators need
    to inspect why the plan stopped.

## Decision 6: Keep clustered and compatibility authority explicit

- **Decision**: Reuse the same provenance vocabulary across native, cluster, and
  compatibility follow-up surfaces, but do not allow cross-workspace selection
  without a direct evidence anchor and do not blur which route is authoritative.
- **Rationale**: The repo already supports clusters and explicit compatibility
  follow-up. Context hardening must strengthen those surfaces, not obscure them.
- **Alternatives considered**:
  - Flatten all routes into one generic context story. Rejected because it would
    hide the continuity authority that existing CLI surfaces already explain.

## Decision 7: Treat documentation layering as part of the feature closeout

- **Decision**: Update the README and impacted docs so the first-run narrative is
  split into a brutal quick path and an advanced architecture layer, while also
  sharpening the Boundline-versus-Canon boundary.
- **Rationale**: The runtime behavior becomes more explainable with this slice,
  but the current README is still too dense for first contact. The documentation
  change is part of the user-visible contract the feature is meant to improve.
- **Alternatives considered**:
  - Leave docs unchanged and rely on inspect output alone. Rejected because the
    user specifically asked for a less intimidating entry path.

## Decision 8: Treat release validation as part of the deliverable

- **Decision**: Keep the feature incomplete until version bump, roadmap cleanup,
  changelog, touched-file coverage above 95%, `cargo fmt`, and clean `clippy`
  have all been completed.
- **Rationale**: The request is explicitly for feature-complete delivery, not a
  code-only slice.
- **Alternatives considered**:
  - Defer release closure and validation to a later pass. Rejected because it
    would leave the runtime and release surface out of sync.