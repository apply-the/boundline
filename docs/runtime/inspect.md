# Inspect

`inspect` explains why the Boundline 0.80.0 runtime chose the current plan or blocked
handoff.

## What To Read

Look for:

- context summary and credibility
- plan-quality state, findings, and assumptions
- backlog-quality state, findings, and additive scope fields
- planning-analysis state, source-attributed findings, and additive coverage
  metrics
- context-pack entry projections with fidelity tiers and inclusion modes
- omission findings, repository-map state, snapshot-cache state, and patch-safe
  edit attempts
- capability-provider validation disposition, accepted evidence refs, rejected
  evidence refs, and limitations when provider-backed execution participated
- emitted `phase_request`
- withheld or recovered execution handoff
- trace-backed evidence for the next action

Use `status` first, then `inspect`, then `next` if you need the recovery
route. Inspect is the right place to confirm whether the blocked condition is a
validation gap, a backlog contradiction, or a governed producer contract gap.
It is also the right place to confirm whether Boundline compacted a large
artifact to a digest, omitted archived or unsafe context, or refused a large
edit because the patch-safe boundary drifted.

When provider-backed execution ran or was blocked, inspect is also the right
place to confirm whether Boundline stopped on readiness, permission admission,
prepare-time missing evidence, execution failure, or post-execution validation.
