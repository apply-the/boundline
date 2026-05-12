# Delivery Pilot Model

Boundline supports large initiatives by piloting them as bounded stages and
bounded work units.

Large work is supported by decomposition, not by unbounded autonomy.

Boundline does not try to complete a whole project in one unchecked run. It
keeps the active workspace state in `.boundline/session.json`, records trace and
checkpoint evidence, and stops when the next action needs better context,
approval, validation, or a narrower boundary.

## Pilot Loop

```text
observe -> decide -> act -> verify -> update context
```

Observe: collect workspace evidence, authored inputs, active goal, briefs,
recent traces, validation output, changed files, checkpoints, and reusable Canon
artifacts.

Decide: choose the next bounded action or governed stage using explicit
evidence, current session state, flow constraints, risk policy, and available
capabilities.

Act: run the selected agent, tool, command, code mutation, test command, or
Canon governance stage call.

Verify: check validation output, diff state, stage readiness, governance packet
state, voting results, and blocked conditions.

Update context: persist session state, trace events, checkpoint references,
Canon packet refs, voting refs, next command, and any clarification or approval
requirement.

## Stop Rules

Boundline stops instead of guessing when:

- context is insufficient
- governance is blocked
- validation is exhausted
- risk exceeds policy
- the next action would exceed the current boundary
- approval, voting, or clarification is required

Boundline can continue after confirmation, approval, clarification, validation
repair, or context repair.

## Project-Scale Example

User goal:

```text
Build a customer onboarding capability with audit logging.
```

Boundline proposed path:

1. discovery if the problem is unclear
2. requirements
3. system-shaping
4. architecture with C4 and ADR coverage
5. backlog
6. implementation slice 1
7. verification
8. pr-review
9. implementation slice 2
10. verification
11. final review

Each implementation slice is bounded and can have its own checkpoint,
validation, trace, review, and Canon packet refs when governance is enabled.
