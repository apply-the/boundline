# Contract: Decision Recovery And Stop Authority

## Goal

Recovery, verification, clarification, and stop conditions are derived from
decision state and remain explicit throughout bounded execution.

## Required Recovery Paths

- retry a bounded selector when the evidence supports another attempt
- choose `replan` when the current path is no longer credible but bounded follow-up remains
- choose `ask` when the missing information is operator-owned rather than tool-owned
- stop explicitly when configured budgets or terminal conditions are reached

## Required Rules

- Verification outcomes can invalidate the previous selector rationale.
- Recovery state must reference the failed decision it is responding to.
- Exhaustion or no-credible-next-step outcomes must stay explicit and inspectable.
- Recovery text must identify one bounded next action or one explicit stop reason.

## Compatibility Rule

- Compatibility-authoritative follow-up may reuse the same recovery vocabulary
  when trace data exists, but must remain visibly trace-authoritative.