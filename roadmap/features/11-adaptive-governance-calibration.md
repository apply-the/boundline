# S16 - Adaptive Governance Calibration

## Owner

Boundline, Canon policy-aware

## Status

B-level, after S15

## Speckit Seed Notes

- Seed role: adaptive-governance hardening over existing runtime confidence and
  control-graduation behavior.
- First slice: make one control-level decision fully inspectable, including
  confidence input, override policy, degradation, and terminal outcome.
- Depends on: existing adaptive governance, control graduation, degradation,
  escalation, and runtime confidence docs.
- De-duplication: do not create a second trust model or a new policy engine;
  future specs must refine the existing runtime-owned calibration path.

## Strategic Role

This feature makes governance adoptable in real teams.

Not every rule should block on day one. Controls must graduate from advisory to enforcement based on maturity, evidence, confidence, and risk.

## Problem

Hard controls too early create friction. Soft controls forever create theater.

Need a system for:

- advisory mode
- catch mode
- rule mode
- hook mode
- confidence
- degradation
- escalation
- trust evolution

## Control Levels

### Advisory

Finding is visible but does not block.

### Catch

Finding asks for attention or response, but human can bypass easily.

### Rule

Finding blocks unless override policy is satisfied.

### Hook

Finding is enforced automatically and cannot be bypassed except by privileged process.

## Inputs

- authority zone
- risk level
- lifecycle phase
- guidance strength
- authority source
- guardian confidence
- historical false positives
- eval performance
- human override history
- incident history
- Canon policy state

## Algorithms And Techniques

### Calibration Table

Maintain a policy table:

```text
rule_id
authority_source
default_level
green_level
yellow_level
red_level
confidence_threshold
override_policy
```

### Trust Evolution

Track:

- guardian true positive rate
- false positive reports
- accepted overrides
- repeated violations
- incident correlations
- eval pass rate

### Degradation

If provider/model/tool is unavailable:

- downgrade to advisory if safe
- require human gate if not safe
- block if mandatory evidence cannot be produced

### Escalation

Escalate when:

- repeated unresolved findings
- red zone
- low confidence but high impact
- missing evidence
- security/domain/contract boundary risk

## Acceptance Criteria

- Every control declares current level.
- Inspect explains why a control is advisory/rule/hook.
- Overrides are trace-visible.
- Degraded execution is explicit.
- Red-zone blockers cannot silently downgrade.
- Control graduation can be tested.
- Eval failures can prevent promotion to stricter levels.

## Risks

- Trust scores become fake precision.
- Teams use overrides as escape hatch.
- Adaptive policy becomes hard to explain.
- Too many knobs.

## Hard Rule

Adaptive governance must be more explainable than static governance, not less.
