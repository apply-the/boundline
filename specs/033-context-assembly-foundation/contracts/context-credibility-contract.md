# Contract: Context Credibility And Failure Handling

## Goal

Context assembly failures are explicit, bounded, and inspectable.

## Required States

- `credible`
- `insufficient`
- `stale`

## Rules

- `credible` permits plan confirmation.
- `insufficient` blocks plan confirmation and surfaces the missing bounded context story.
- `stale` blocks reliance on the previous context pack until a bounded recovery action or replanning step refreshes the context.

## Required Failure Visibility

When credibility is not sufficient:

- the terminal or follow-through output must mention context credibility explicitly
- inspect surfaces must preserve the reason the pack is insufficient or stale
- the operator must receive one bounded next action rather than a generic failure message
