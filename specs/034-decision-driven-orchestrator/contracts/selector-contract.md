# Contract: Selector-Driven Decision Choice

## Goal

Each native loop iteration chooses one explicit bounded next-action selector
instead of relying on implicit static step order.

## Required Selectors

- `read`
- `search`
- `modify`
- `test`
- `ask`
- `replan`

## Required Outcomes

- Each persisted decision carries exactly one selector.
- Each selector includes a bounded target, rationale, and expected outcome.
- Each selector includes the evidence basis or bounded context that justified it.
- Selector choice is visible before the action result is interpreted.

## Failure Contract

- If no credible selector exists, Synod must surface `ask`, `replan`, or an
  explicit terminal stop instead of silently walking a fallback plan step.