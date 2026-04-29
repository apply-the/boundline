# Research: Session-Native Surface Unification

**Feature**: 016-session-native-surface-unification  
**Date**: 2026-04-29

## R1: Unification should happen in the session-owned summary model, not in a new runtime layer

**Question**: Where should Synod unify the remaining operator-facing surfaces after 015-runtime-refoundation?

**Decision**: Extend the existing session-owned status and trace summary model so route explanation, execution condition, optional review or adaptive or governance state, and compatibility explanations are all projected through the same operator-facing surfaces.

**Rationale**: The product problem is not missing runtime capability. The problem is that optional bounded modes still feel like separate stories. A shared summary model fixes the operator experience without adding a second abstraction layer.

**Alternatives Considered**:
- Create a new meta-summary layer outside session and trace models: rejected because it adds another projection surface and weakens inspectability.
- Leave each optional mode with dedicated rendering rules: rejected because that preserves the fragmented product story.

## R2: Execution condition needs an explicit operator-facing projection

**Question**: How should Synod describe running, blocked, waiting, and terminal states consistently across the CLI?

**Decision**: Introduce an explicit execution-condition projection derived from session and trace state that can explain whether the session is running, blocked, waiting on governance or review, succeeded, failed, exhausted, or has no actionable next step.

**Rationale**: Existing fields expose many details, but operators still have to infer the current state from a mix of route, review, governance, and trace cues. A normalized condition projection makes blocked and waiting behavior first-class without changing the bounded engine semantics.

**Alternatives Considered**:
- Continue inferring condition implicitly from `latest_status` plus optional fields: rejected because the inference remains brittle and inconsistent across `run`, `status`, `next`, and `inspect`.
- Expand the core session status enum to absorb every optional state directly: rejected because the runtime enum should stay focused on execution lifecycle, while operator-facing explanation can remain a derived projection.

## R3: Optional bounded modes should extend the primary session story, not replace it

**Question**: How should review, adaptive execution, and governance appear after unification?

**Decision**: Treat review, adaptive, and governance state as bounded projections attached to the same session-native summary model, while preserving their underlying evidence and trace details.

**Rationale**: These capabilities already have bounded value. The missing piece is consistent projection. Attaching them to the same summary model preserves both inspectability and one coherent product story.

**Alternatives Considered**:
- Flatten every optional field into one generic blob of metadata: rejected because it weakens meaning and makes guidance less actionable.
- Keep optional modes as independent rendering branches: rejected because it keeps the operator mental model fragmented.

## R4: Compatibility precedence must stay explicit and subordinate to a ready native plan

**Question**: How should Synod reconcile `.synod/execution.json` with a ready session-native plan in operator-facing surfaces?

**Decision**: Keep compatibility as an explicit path that remains available when intentionally selected or when it is the only credible route, but preserve ready session-native plans as the default authoritative route.

**Rationale**: This preserves backward compatibility without letting the old path retake control of the product story. It also keeps route explanations stable across all surfaces.

**Alternatives Considered**:
- Let compatibility and session-native routes compete implicitly based on whatever state is present: rejected because it leads back to guesswork.
- Remove compatibility behavior entirely: rejected because existing users and tests still rely on explicit manifest-backed execution.

## R5: `inspect` should reuse the unified semantics while remaining trace-specific

**Question**: Should `inspect` become a separate diagnostic surface or share the same summary semantics as `status` and `next`?

**Decision**: `inspect` should reuse the same route and execution-condition semantics as the session summary while adding trace-specific detail such as decision timeline, failure evidence, and ordered recovery history.

**Rationale**: `inspect` is still the detailed read-side surface, but it should not force operators to learn a second explanation model.

**Alternatives Considered**:
- Keep `inspect` fully independent and trace-centric only: rejected because route and condition explanations would drift from the session-owned story.
- Collapse `inspect` into a simplified status view: rejected because trace-level reasoning and failure evidence still matter.

## R6: Release rollout should include version, Canon compatibility target, docs, and coverage in one pass

**Question**: What release hygiene belongs inside this slice rather than as follow-up cleanup?

**Decision**: Make `0.16.0` the release target, update the documented Canon compatibility target to `0.24.0`, and reserve the final implementation tasks for staged-file coverage, docs and template updates, clippy cleanup, and `cargo fmt`.

**Rationale**: The feature is primarily about operator surfaces. If docs, assistant assets, and release expectations lag the implementation, the unification does not land as a coherent product change.

**Alternatives Considered**:
- Defer docs and release cleanup to a later slice: rejected because that would leave the product story inconsistent at release time.
- Change Canon behavior more deeply while updating the version target: rejected because deeper escalation is explicitly out of scope for this slice.