# Inspect

`inspect` explains why the Boundline 0.69.0 runtime chose the current plan or blocked
handoff.

## What To Read

Look for:

- context summary and credibility
- plan-quality state, findings, and assumptions
- backlog-quality state, findings, and additive scope fields
- emitted `phase_request`
- withheld or recovered execution handoff
- trace-backed evidence for the next action

Use `status` first, then `inspect`, then `next` if you need the recovery
route.
