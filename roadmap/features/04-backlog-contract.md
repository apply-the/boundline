# Boundline Backlog Contract

## Summary

Define the Speckit tasks analogue as a Boundline validation contract over Canon backlog output. Boundline should not add a `/boundline-tasks` command in this slice. Instead, Canon `backlog` mode remains the governed backlog producer for delivery workflows, and Boundline validates the backlog packet before considering planning complete.

If implementation requires any Canon output or schema change, stop Boundline work and create a Canon Speckit feature in `/Users/rt/workspace/apply-the/canon`: branch, spec, plan, tasks, then implement Canon changes.

## Public And Runtime Interface Changes

Add optional backlog projection fields to session status, orchestrate snapshots, and rendered output when a backlog packet is expected or available:

- `backlog_quality_state`: `ready`, `clarification_required`, or `blocked`
- `backlog_quality_findings`: concise labels for missing or invalid backlog structure
- `backlog_task_count`: total validated task count
- `backlog_mvp_scope`: the first independently deliverable slice
- `backlog_unmapped_items`: tasks or requirements that could not be mapped

These fields are additive and must not break existing status/orchestrate consumers.

## Runtime Behavior

Boundline validates Canon backlog packets before advancing from planning to execution.

A valid backlog packet should provide:

- stable task IDs
- dependency ordering
- phase or user-story grouping when applicable
- explicit file paths or artifact refs for implementation work
- independent test criteria for the MVP or each story-sized slice
- MVP or first-slice marker
- optional parallelization markers
- traceability to goal requirements, success criteria, plan decisions, or acceptance criteria when available

If the packet is incomplete, Boundline should surface `backlog_quality_state: blocked` or `clarification_required` and stop on the existing planning `phase_request` mechanism. It must not silently translate an incomplete backlog into executable work.

## Assistant Asset Updates

Update plan/run assistant assets so they treat backlog quality as a real planning gate:

- preserve `backlog_quality_state`, `backlog_quality_findings`, `backlog_task_count`, `backlog_mvp_scope`, and `backlog_unmapped_items`
- do not route to `/boundline-run` when backlog quality is blocked
- present the emitted `phase_request` or planning-stage resume command as the only next route
- explain that Canon backlog is governed source material, while Boundline validates execution readiness

## Tests

Add unit, contract, and integration coverage for:

- Canon backlog packet without stable task IDs blocks planning completion
- backlog without dependency order or MVP scope produces quality findings
- valid backlog packet advances planning and exposes task count and MVP scope
- orchestrate emits backlog quality fields in the planning-stage session snapshot
- assistant assets preserve backlog gates and do not jump to run while blocked

Run:

- `cargo test --test unit`
- `cargo test --test contract`
- `cargo test --test integration init_bootstrap_flow`

## Canon Boundary

No Canon files are changed as part of this Boundline planning document.

Future Canon-impacting work must use Speckit in `/Users/rt/workspace/apply-the/canon`:

1. create or switch a feature branch with Speckit
2. write the feature spec
3. produce the technical plan
4. produce tasks
5. implement Canon output/schema changes

## Assumptions

- Boundline can initially validate only fields already available from Canon backlog packets.
- Any missing Canon data that requires schema changes is deferred to a Canon Speckit feature.
- Backlog validation is a planning gate, not an execution command.
