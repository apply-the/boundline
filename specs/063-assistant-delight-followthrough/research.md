# Research: S7.1 Assistant Delight Follow-Through

## Outcome

Phase 0 resolved the planning unknowns for this slice without leaving any
`NEEDS CLARIFICATION` markers. The follow-through remains a Boundline-side
extension of the shipped delight layer.

## Decision 1: Keep the slice Boundline-only

- Decision: Implement S7.1 entirely in Boundline unless implementation later
  exposes a concrete Canon contract gap.
- Rationale: The roadmap follow-through explicitly excludes new Canon provider
  artifact classes, and the existing repository already carries the reasoning
  posture and delight-provider contract coverage needed for profile-aware
  explanation and inspect closure.
- Alternatives considered: Open a parallel Canon spec now. Rejected because no
  missing Canon-side capability is visible from the current Boundline runtime,
  contracts, or roadmap constraints.

## Decision 2: Reuse existing reasoning and trace authority

- Decision: Use the existing `ProfileActivationRecord`, `ActiveSessionRecord`,
  and `TraceSummaryView` surfaces as the authoritative inputs for S7.1
  explanation disclosure.
- Rationale: `src/domain/reasoning.rs` already persists profile identity,
  trigger, activation reason, posture, independence, outcome, and confidence.
  `src/domain/trace.rs` already exposes context, decision, review, governance,
  step, recovery, and reasoning fields through one flattened trace summary.
  Reusing those models keeps the slice sequential, inspectable, and compatible
  with existing status and inspect output.
- Alternatives considered: Re-scan raw trace events on every command or create
  a new persistence record just for delight output. Rejected because both add
  avoidable projection drift and duplicate state authority.

## Decision 3: Implement inspect closure as named projections over current trace summaries

- Decision: Build `inspect context`, `inspect council`, and `inspect timeline`
  from existing summary fields such as `context_summary`,
  `context_provenance`, `review_timeline`, `governance_timeline`,
  `decision_timeline`, `executed_steps`, and `recovery_events`.
- Rationale: `src/cli/inspect.rs` already rehydrates a flattened
  `TraceSummaryView` and already converts review and governance events into
  operator narrative lines. Closing the remaining inspect surfaces should extend
  that projection layer instead of introducing a new trace-reading surface.
- Alternatives considered: Expose raw trace JSON directly or add a dedicated
  timeline store. Rejected because raw payload inspection violates the feature
  goal and a new store would widen scope beyond the smallest useful slice.

## Decision 4: Keep host support states explicit

- Decision: Preserve explicit host support states for delight follow-through,
  using the current repository surfaces as the baseline: Claude and Codex keep
  their repository-managed command assets plus copy-ready bootstrap support,
  Copilot keeps prompt-backed repository assets plus manual global fallback,
  Cursor stays copy-ready asset driven, and Gemini stays explicit manual
  fallback unless the implementation work proves higher parity is worth the
  complexity.
- Rationale: `assistant/global/manifest.json` already models host support as
  `copy_ready_assets` or `manual_fallback`, and the repository already contains
  explicit Cursor and Gemini fallback docs. S7.1 should make parity decisions
  visible, not ambiguous.
- Alternatives considered: Claim full parity for every host or leave Cursor and
  Gemini in an undocumented middle state. Rejected because both hide the real
  operator boundary.

## Decision 5: Capture usefulness signals inside existing session or trace authority

- Decision: Record lightweight delight usefulness signals in existing session
  or trace authority and render them through status or inspect projections.
- Rationale: The feature only needs session-scoped operator usefulness signals,
  such as time to first useful answer, source-attribution completeness, and
  next-action acceptance or override behavior. Those can remain independently
  testable if they live near existing session and trace state.
- Alternatives considered: Add a separate analytics file, external database, or
  background telemetry pipeline. Rejected because they violate the minimal-slice
  and external-separation constraints.

## Decision 6: Keep the bundled model catalog unchanged

- Decision: Carry forward the no-change catalog result from specification into
  planning.
- Rationale: Public provider documentation checked during specification still
  matches the families already bundled in
  `assistant/catalog/model-catalog.toml`, including GPT-5.5 and GPT-5.4
  variants, Claude Opus 4.7 and Sonnet 4.6, and Gemini 2.5 and 3.1 entries.
  S7.1 changes delight projections and host behavior, not routing-model
  compatibility.
- Implementation re-check: A second provider-doc review on 2026-05-19 during
  `/speckit.implement` confirmed the same no-change result, so this slice still
  does not require a bundled catalog delta.
- Alternatives considered: Refresh the catalog as part of this slice anyway.
  Rejected because there is no provider-model delta to encode.

## Implementation Surfaces To Carry Into Phase 1

- `src/cli/output.rs`: extend delight explanation projections with explicit
  reasoning-profile contribution and fallback semantics.
- `src/cli/inspect.rs`: extend inspect output into context, council, and
  timeline closures using the existing flattened trace summary.
- `src/domain/reasoning.rs`: reuse profile activation and confidence fields as
  disclosure inputs.
- `src/domain/trace.rs`: reuse flattened trace summary fields for context,
  council, timeline, and feedback projections.
- `assistant/global/manifest.json` plus host docs under `assistant/global/` and
  `assistant/gemini/`: encode explicit host parity or fallback boundaries.
- `tests/contract/assistant_command_pack_contract.rs`: validate that any
  expanded host command coverage stays aligned with the assistant package
  contract surface.