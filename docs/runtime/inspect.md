# Inspect

`inspect` explains why the Boundline 0.70.0 runtime chose the current plan or blocked
handoff.

## What To Read

Look for:

- context summary and credibility
- plan-quality state, findings, and assumptions
- backlog-quality state, findings, and additive scope fields
- planning-analysis state, source-attributed findings, and additive coverage
  metrics
- emitted `phase_request`
- withheld or recovered execution handoff
- trace-backed evidence for the next action

Use `status` first, then `inspect`, then `next` if you need the recovery
route. Inspect is the right place to confirm whether the blocked condition is a
validation gap, a backlog contradiction, or a governed producer contract gap.
