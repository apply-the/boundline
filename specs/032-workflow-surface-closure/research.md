# Research: Product Unification And Surface Closure

**Feature**: 032-workflow-surface-closure  
**Date**: 2026-05-02

## R1: Promote named workflows to first-class assistant surfaces

**Decision**: Extend the shipped assistant assets so named workflow discovery
and continuation are expressed as first-class Boundline assistant commands or
guidance rather than leaving `boundline workflow ...` to raw fallback prose.

**Rationale**: The workflow layer already exists as a real operator entrypoint.
As long as assistants describe it as an undocumented manual escape hatch,
Boundline still presents multiple partially overlapping products.

**Alternatives Considered**:
- Keep workflow guidance only in the shared README: rejected because it leaves
  each assistant surface incomplete at the exact moment the operator needs a
  bounded next step.
- Create a separate workflow-only assistant package family: rejected because it
  would introduce another surface instead of closing the existing one.

## R2: Reuse the existing session projection vocabulary for workflow output

**Decision**: Keep workflow reports on the same rendering vocabulary already
used by session-native follow-through, including routing, execution condition,
route projection, assistant binding, and next-command cues.

**Rationale**: `workflow run|status|resume|inspect` already build on the shared
session view. Reusing that projection is the smallest coherent way to make
workflow routing inspectable without inventing a second output schema.

**Alternatives Considered**:
- Add a workflow-only rendering format: rejected because it would make workflow
  output drift away from `status`, `next`, and `inspect` at the point where the
  product is supposed to converge.
- Add a separate workflow routing-inspection command: rejected because the
  operator already expects route and binding cues on the current workflow
  surfaces.

## R3: Keep assistant capability enforcement inside the existing native runtime

**Decision**: Continue to enforce assistant capability mismatches through the
existing native session-runtime validation path rather than adding a separate
workflow-only capability mechanism.

**Rationale**: Workflow execution already compiles onto the same native
session-owned runtime. Reusing the current validation path preserves one
authoritative place where unsupported assistant bindings fail explicitly.

**Alternatives Considered**:
- Add workflow-only capability checks in assistant assets: rejected because it
  would turn guidance files into a second enforcement layer.
- Skip capability enforcement for workflows: rejected because it would leave a
  hidden fallback path exactly where `0.32.0` is meant to remove ambiguity.

## R4: Keep workflow and direct native execution primary, and compatibility explicit

**Decision**: Treat named workflows and direct session-native commands as the
two primary Boundline entry styles that share one execution model, while keeping
explicit compatibility follow-up visibly subordinate and trace-authoritative.

**Rationale**: The roadmap goal is product closure, not route proliferation.
Operators need one clear answer to "what product am I using now?" and that
answer must remain "Boundline" unless they explicitly chose compatibility.

**Alternatives Considered**:
- Treat workflows as a third parallel execution mode: rejected because it would
  preserve the exact product ambiguity the feature is meant to close.
- Collapse compatibility into workflow guidance by default: rejected because it
  would hide authority and contradict previous continuity slices.

## R5: Keep Gemini CLI-first but aligned to the same workflow vocabulary

**Decision**: Continue to treat Gemini as CLI-first for this release, but align
its guidance to the same workflow, routing, and product-identity vocabulary
used by Claude, Codex, and Copilot.

**Rationale**: The current shipped artifact for Gemini is documentation rather
than a chat-native pack. That can still participate in product closure as long
as it uses the same bounded Boundline-owned story and does not imply a special
runtime.

**Alternatives Considered**:
- Ship a full Gemini command pack in this slice: rejected because it expands
  scope beyond the smallest credible closure.
- Leave Gemini on older wording: rejected because the release would still ship
  two conflicting product stories.

## R6: Treat release closeout as part of the feature, not post-hoc cleanup

**Decision**: Include version bump, impacted docs, assistant guidance,
changelog, coverage refresh for touched Rust files, clippy cleanup, and
formatting as first-class tasks in the feature.

**Rationale**: `0.32.0` is explicitly a product-closure slice. It cannot be
considered done if runtime changes land while release-facing artifacts still
describe Boundline as a set of disconnected surfaces.

**Alternatives Considered**:
- Defer release surfaces until after implementation: rejected because the
  product identity work is part of the value, not incidental polish.
- Skip touched-file coverage refresh: rejected because the requested delivery
  bar for this repository is explicit and release-blocking.