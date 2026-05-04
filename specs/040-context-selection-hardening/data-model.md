# Data Model: Context Selection Hardening

## Context Evidence Anchor

- **Purpose**: Represents the direct bounded reason a file or artifact may enter
  the active planning context.
- **Examples**:
  - failing test target
  - recent validation path
  - compiler or linter path
  - authored brief reference
  - workflow-owned target
  - recent mutation record
  - Canon artifact reference
- **Validation rules**:
  - Must be specific enough to explain why the input is relevant now.
  - Must not point outside the active workspace or registered cluster scope
    unless the operator already chose that scope explicitly.

## Context Input

- **Purpose**: Represents one selected planning input persisted inside
  `ContextPack`.
- **Existing fields reused**:
  - `kind`
  - `reference`
  - `rationale`
  - `source`
  - `primary`
- **Expanded responsibilities**:
  - Encode which evidence anchor justified the input.
  - Differentiate direct evidence-backed primary inputs from secondary context.
  - Keep rationale text precise enough for CLI projection and trace inspection.
- **Validation rules**:
  - `reference`, `rationale`, and `source` must remain non-empty.
  - A credible primary workspace input must describe direct bounded relevance,
    not only a generic goal match.

## Context Candidate Set

- **Purpose**: Planner-local normalized set of possible files or artifacts before
  final bounded selection.
- **Derived from**:
  - workspace scans
  - authored brief file references
  - validation or trace paths
  - recent mutation hints
  - workflow targets
  - Canon artifact references
- **Fields**:
  - `reference`
  - `kind`
  - `evidence_anchors`
  - `priority`
  - `cluster_scope`
- **Validation rules**:
  - Candidates without evidence anchors cannot make the pack credible.
  - Cross-workspace candidates require an explicit scope-safe anchor.

## Context Pack

- **Purpose**: Remains the authoritative persisted context bundle for one goal
  plan revision.
- **Existing fields reused**:
  - `pack_id`
  - `summary`
  - `credibility`
  - `inputs`
  - `selected_targets`
  - `staleness_reason`
- **Expanded responsibilities**:
  - Summarize why the pack is credible, stale, or insufficient.
  - Keep selected targets aligned with the evidence-backed primary inputs.
  - Preserve enough detail for session and trace projections to explain the
    pack without recomputing it.
- **Validation rules**:
  - A credible pack must have at least one evidence-backed primary input or
    selected target.
  - A stale pack must record an explicit stale reason.
  - An insufficient pack must preserve the missing or conflicting evidence story.

## Context Projection

- **Purpose**: The operator-facing summary surfaced through `plan`, `run`,
  `status`, `next`, and `inspect`.
- **Derived fields**:
  - `context_summary`
  - `context_credibility`
  - `context_primary_inputs`
  - `context_provenance`
  - `context_staleness_reason`
- **Validation rules**:
  - Must preserve route authority when projected from compatibility follow-up.
  - Must explain at least one bounded recovery cue when the pack is not
    credible.

## Documentation Layer

- **Purpose**: Captures the release-facing explanation of the feature so first
  contact stays simpler than the full architecture story.
- **Artifacts**:
  - README quick path
  - advanced architecture references
  - roadmap and changelog updates
- **Validation rules**:
  - Quick path must stand alone for first-run success.
  - Advanced architecture remains available without polluting the first-run
    sequence.