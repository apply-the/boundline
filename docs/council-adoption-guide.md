# Council Adoption Guide For Boundline 0.56.0

Boundline `0.56.0` is designed so teams can adopt authority-zoned councils
incrementally. You do not need to turn every stage into a permanently governed
debate. The intended pattern is bounded escalation: use stronger council and
stop semantics only where the boundary actually justifies them.

## Start Small

Good first adoption targets:

- discovery or requirements work that may later promote into governed delivery
- implementation or refactor stages where you want lightweight review posture
- architecture, verification, migration, or security boundaries where the team
  already expects stronger human scrutiny

Keep the early rollout explicit and inspectable. If a stage does not need Canon
governance, leave it on the normal local path. If a stage does need Canon
governance, make that requirement explicit in the stage policy rather than
hoping a missing envelope will be ignored.

## Required Canon Stages Must Stay Required

For required Canon-governed stages, Boundline now fails closed when the runtime
cannot supply compatible `authority-governance-v1` metadata.

That protects teams from the most dangerous adoption failure mode: silently
continuing as if a governed decision had been made when the actual authority
envelope is absent or incompatible.

## Recommended Rollout Shape

1. Start with green low-impact discovery and requirements work.
2. Add light-single review posture to green low-impact implementation and refactor work.
3. Use paired review for yellow-band implementation or verification boundaries.
4. Reserve red-five adjudication or human-gate paths for systemic, structural, migration, and security work.
5. Treat restricted or destructive operations as manual-gate territory from the start.

This keeps the team inside the bounded matrix instead of jumping directly from
no governance to manual hard stops everywhere.

## What To Check During Rollout

After `boundline run`, verify:

- `status` shows the expected council profile
- `next` preserves the stop semantics and next action
- `inspect` explains the authority provenance and review posture
- blocked stages mention a concrete reason such as missing authority metadata,
  unsupported contract line, independence failure, or unresolved approval state

If those surfaces cannot explain why the boundary stopped, the rollout is too
opaque and should not be expanded yet.

## Common Operational Patterns

- green low-impact discovery: no dedicated council, continue with normal bounded work
- green low-impact implementation: one bounded reviewer can satisfy the path
- yellow bounded implementation or verification: use paired review and preserve the council requirement
- yellow systemic structural work: expect adjudication, not a casual tie-break
- red structural work: expect an explicit human gate
- restricted work: expect automation to stop until the manual boundary is resolved

## Troubleshooting

If a governed stage stops unexpectedly:

- check whether the stage was marked `required`
- confirm the packet carries `authority-governance-v1`
- confirm the contract line is supported
- inspect packet readiness and approval state
- review the council profile, independence state, and stop semantics together

`status` is the fastest summary. `inspect` is the right place to confirm the
full reasoning trail before changing policy or rerunning the stage.