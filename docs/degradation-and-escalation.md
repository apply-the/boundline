# Degradation And Escalation In Boundline 0.56.0

Boundline must not silently weaken governance when runtime conditions are no
longer sufficient.

When evidence, confidence, reviewer coverage, or contract compatibility falls
short, Boundline either degrades explicitly or escalates explicitly.

## Degradation Means Narrower Runtime Autonomy

Degradation is the controlled reduction of governance ambition so the runtime
can stay honest about what it can safely support.

Representative degradation modes for this slice are:

- `advisory_fallback`
- `smaller_council`
- `human_gate`
- `reduced_autonomy`
- `verification_only`
- `execution_block`

These are runtime mechanisms. They are not a second stop-semantics vocabulary.

## Mapping To Existing Stop Semantics

Boundline continues to map degradation outcomes onto the existing S3 stop
posture vocabulary instead of inventing a competing control language.

Representative mappings include:

- advisory fallback to `proceed_with_advisory`
- bounded warning paths to `proceed_with_warning`
- degraded governed continuation to `degraded_proceed`
- mandatory human intervention to `human_gate_required`
- incompatible or unsupported required governance to `hard_stop`

## Typical Degradation Triggers

Boundline may degrade when it encounters conditions such as:

- low confidence
- weak reviewer credibility
- missing evidence
- incomplete traces
- insufficient context
- degraded runtime support
- optional companion semantics that are unavailable but not required

The key rule is visibility: degradation must remain inspectable and
explainable.

## Escalation Means Authority Transfer

Escalation occurs when the runtime should not keep deciding locally with the
current evidence or authority level.

Representative escalation triggers include:

- missing required reviewer or approval
- policy uncertainty
- critical disagreement
- unsupported required contract line
- missing required Canon baseline
- required adaptive companion unavailable under stage policy
- high-risk ambiguity or restricted operation

Escalation may require stronger councils, higher-trust reviewers, human
approval, or a more authoritative governance path.

## Human Authority Remains Intact

Even in stronger governance states, Boundline keeps human authority visible.

- stronger progression remains operator-approved
- escalation paths remain available
- override events must remain explicit and traceable
- blocked execution must explain why the runtime could not continue

## What Operators Should See

Normal runtime surfaces should make these answers clear:

- why governance degraded
- why governance escalated
- which stop posture or gate now applies
- whether execution may continue, pause, wait, or stop
- what evidence or approval is needed next

Those answers belong in the same operator path:

```text
plan -> run -> status -> next -> inspect
```

## Current Slice Boundary

This S4 slice keeps degradation and escalation runtime-owned inside Boundline.

Canon may explain posture and compatibility inputs, but it does not choose the
degradation mode, escalation target, or final stop behavior.