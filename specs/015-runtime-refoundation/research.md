# Research: Runtime Refoundation

**Feature**: 015-runtime-refoundation  
**Date**: 2026-04-29

## R1: The bounded task draft remains the authoritative session-native plan

**Question**: What planning artifact should represent the primary session-native path after refoundation?

**Decision**: Keep a bounded task draft as the authoritative persisted planning record for the normal runtime path, and treat declarative execution profiles as an explicit compatibility contract rather than the default planning contract.

**Rationale**: The refoundation only changes the product if planning state belongs to the session-native runtime and survives into execution, inspection, and recovery. If declarative profiles remain the implicit source of truth, Boundline still behaves like a compatibility engine with extra metadata.

**Alternatives Considered**:
- Keep declarative execution profiles as the default plan source and attach session-native state as advisory metadata: rejected because it leaves the legacy control model in charge.
- Remove declarative execution profiles entirely: rejected because compatibility behavior still has bounded value and existing workflows depend on it.

## R2: Next-action selection must be driven by live state plus explicit terminal precedence

**Question**: How should the runtime choose the next bounded action without becoming opaque or unbounded?

**Decision**: Choose each next action from live workspace evidence, prior decision results, active flow constraints, and the current bounded task draft, while applying explicit precedence for terminal outcomes: no-actionable-state and exhausted states terminate immediately; failed decisions preserve evidence before bounded recovery or replan is attempted.

**Rationale**: This keeps decision selection adaptive but inspectable. Developers can understand why Boundline continued, stopped, or recovered because the runtime uses visible evidence and explicit precedence rules instead of hidden heuristics.

**Alternatives Considered**:
- Replay static step lists and call them “adaptive”: rejected because it does not change the control model.
- Use a generic planner with opaque self-modifying loops: rejected because it weakens trust and debugging.

## R3: Flow remains lightweight, explicit, and policy-shaped

**Question**: How should flow participate in the refounded runtime?

**Decision**: Planning may propose a flow from captured evidence, but execution only treats it as active policy after explicit operator confirmation or override. Confirmed flow constrains allowed decision families by stage, while skipped flow leaves the runtime unconstrained.

**Rationale**: The product review called for “infer, show, require lightweight confirmation” rather than silent auto-run. Flow becomes useful when it bounds decisions, not when it acts as hidden routing or a rigid script.

**Alternatives Considered**:
- Silent auto-confirmation of inferred flow: rejected because it hides a material execution choice.
- Flow as a decorative label with no runtime effect: rejected because it adds ceremony without control value.

## R4: Compatibility routing must be explicit and precedence-based

**Question**: How should Boundline choose between the session-native path and the compatibility path?

**Decision**: Use explicit routing precedence: a persisted session-native bounded task draft takes priority, an explicit operator opt-in can select compatibility mode, declarative execution profiles remain available when no session-native plan exists, and blocked states return remediation instead of silent fallback.

**Rationale**: The operator must be able to explain why a run used one path or the other. Hidden fallback is incompatible with the refoundation thesis.

**Alternatives Considered**:
- Keep compatibility mode as the implicit default: rejected because it preserves the wrong product story.
- Merge compatibility and session-native state into one blended runtime mode: rejected because it makes route choice and failure diagnosis harder to inspect.

## R5: Canon artifacts are bounded planning and stage-boundary inputs only

**Question**: What role should Canon play in the refounded runtime?

**Decision**: Use Canon artifacts only as bounded planning inputs and stage-boundary governance evidence. Per-action decision selection remains Boundline-owned and must continue to work when Canon artifacts are absent.

**Rationale**: This preserves the separation between orchestration and governance. Boundline remains independently testable and executable, while Canon still contributes governed context where it matters.

**Alternatives Considered**:
- Make Canon the per-action decision engine: rejected because it collapses Boundline’s control plane into an external dependency.
- Ignore Canon artifacts entirely: rejected because the governance boundary still has value and should remain available as evidence.

## R6: Rollout must include version, docs, templates, and examples in one final pass

**Question**: How should the refoundation be rolled out so product narrative matches runtime behavior?

**Decision**: Reserve a final cross-cutting phase for version bump to `0.15.0`, coverage recovery, and updates to README, ROADMAP, docs, assistant templates, and examples.

**Rationale**: The feature is an architectural and product-story refoundation. If documentation and templates lag behind runtime behavior, operators will continue following the pre-refoundation path.

**Alternatives Considered**:
- Update documentation opportunistically during implementation: rejected because the rollout work becomes fragmented and easy to miss.
- Defer templates and examples to a later slice: rejected because it leaves the primary usage story inconsistent at release time.