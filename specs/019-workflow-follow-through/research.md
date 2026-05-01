# Research: Workflow Follow-Through

**Feature**: 019-workflow-follow-through  
**Date**: 2026-05-01

## R1: Execute review and govern by extending the existing session-native control plane

**Decision**: Make workflow review and govern executable by compiling them onto the existing session-native review and governance behavior instead of introducing a second workflow runtime or a workflow-specific controller.

**Rationale**: The delivery value comes from completing the already-started workflow slice, not from adding a new orchestration engine. Reusing the existing session-native control plane keeps routing, terminal states, and inspectability consistent.

**Alternatives Considered**:
- Add a workflow-only execution runner for review and govern: rejected because it duplicates state and control flow.
- Keep review and govern declaration-only for another release: rejected because it leaves the workflow slice incomplete for governed delivery work.

## R2: Add one bounded workflow discovery surface instead of assistant-only heuristics

**Decision**: Add one operator-facing workflow discovery surface that exposes available named workflows, their intended summary, and invocation guidance directly from Synod.

**Rationale**: Discovery should be part of the product surface, not something assistants must guess from file contents or external docs. A bounded discovery surface improves assistant ergonomics while preserving one authoritative CLI story.

**Alternatives Considered**:
- Rely only on repository documentation: rejected because it leaves invocation guidance stale or out of band.
- Let assistants inspect workflow files directly without a product surface: rejected because it makes discovery inconsistent and harder to trust.

## R3: Keep workflow discovery metadata minimal and optional

**Decision**: Support minimal operator-facing workflow metadata for discovery guidance, while keeping existing registry definitions valid by falling back to workflow name and declared phases when no extra summary is provided.

**Rationale**: Discovery needs more than a raw workflow name, but the slice should not force a large schema expansion or a registry migration. Optional metadata gives maintainers a better discovery story without widening the model.

**Alternatives Considered**:
- Require rich metadata for every workflow: rejected because it turns the slice into a registry redesign.
- Derive all guidance from phase order alone: rejected because it does not explain when a workflow should be chosen.

## R4: Preserve explicit blocked and non-success follow-through semantics

**Decision**: Review and govern outcomes remain bounded and explicit. If a workflow cannot continue because prerequisites, approval, reviewer outcome, or credible next action are missing, the workflow must stop in a visible paused, blocked, or failed state with actionable next guidance.

**Rationale**: The constitution requires explicit failure handling and visible control flow. Review and governance work is only credible if blocked and non-success paths are inspectable instead of hidden.

**Alternatives Considered**:
- Auto-skip blocked review or govern phases: rejected because it hides important quality-control behavior.
- Retry review or governance implicitly in the background: rejected because it violates bounded sequential execution.

## R5: Treat authored workflow guidance as part of the feature, not post-release polish

**Decision**: Ship README, docs, roadmap, changelog, and assistant guidance updates as part of the same slice, including examples for authored workflow registries that use review and govern.

**Rationale**: Workflow follow-through changes the operator story. Without updated examples and guidance, maintainers and assistants would have no clear reference for the supported bounded model.

**Alternatives Considered**:
- Defer documentation to a later release: rejected because it would leave the feature underspecified for real use.
- Document only the CLI change and skip authoring examples: rejected because discovery and registry guidance are part of the requested scope.

## R6: Reserve the release closeout explicitly for 0.19.0

**Decision**: Start the implementation plan with the version bump to `0.19.0` and close with docs, roadmap, changelog, assistant guidance, coverage refresh, clippy, fmt, and final validation.

**Rationale**: This repository treats each roadmap slice as a versioned delivery unit. Encoding release hygiene directly into the feature plan reduces mismatch between implementation and release artifacts.

**Alternatives Considered**:
- Handle versioning and release docs after implementation: rejected because it creates avoidable end-of-slice drift.
- Treat release validation as optional polish: rejected because the repository already expects executable validation and aligned docs for shipped slices.