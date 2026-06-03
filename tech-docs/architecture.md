# Boundline Advanced Architecture

Read this after the README and [getting-started.md](getting-started.md) when you
need the product boundary rather than the first-run workflow.

For the project-scale delivery model, read [delivery-model.md](delivery-model.md).

## Boundline Versus Canon

Keep this boundary explicit:

- Boundline owns orchestration, bounded planning, execution, validation,
  session continuity, traces, and the primary operator-facing CLI.
- Canon owns governed stages, approvals, governed artifacts, and the
  machine-facing governance adapter when a Boundline route explicitly enables
  it.

Canon is not the orchestrator and not the product entrypoint. A Boundline
install can stay fully usable without Canon on the default local path.

The current Boundline adapter documents Canon `0.63.0` support for the
`canon governance start|refresh|capabilities --json` `v1` surface. That is a
bounded compatibility target, not a claim of total Canon feature parity.

## Primary Runtime Model

The normal operator path is goal-first:

1. `init`
2. `goal`
3. `plan`
4. `run`
5. `status`
6. `next`
7. `inspect`

Boundline persists that story in workspace-local state under `.boundline/` and
keeps traces beside the same session model.

`run --goal "..."` remains an explicit fast path, but it does not replace the
primary product story above.

## Repo-Visible Document Boundary

Boundline keeps runtime state and repo-visible delivery knowledge separate:

- `.boundline/` owns session state, traces, checkpoints, and transient
  governance artifacts.
- `.boundline/context-intelligence/` owns the derived retrieval DB, companion
  manifest, and SQLite WAL/SHM sidecars used by local semantic retrieval.
- `.canon/` owns raw Canon run packets and Canon runtime payloads.
- `docs/project/` owns stable repo-visible project memory that planning and
  governed delivery can reuse.
- `docs/evidence/` owns durable feature outputs and evidence bundles that
  should remain readable after a bounded delivery slice completes.

See
[project-memory-and-evidence-structure.md](project-memory-and-evidence-structure.md)
for the operator-facing folder contract.

## Preflight Surfaces

Two read-side surfaces sit in front of the main runtime loop when you need
them:

- `boundline models auth ...` for user-scoped provider credential setup
- `boundline probe` for a read-only readiness answer before orchestration
- `boundline index ...` for explicit derived-index status, refresh, rebuild,
  cleanup, and doctor operations when local semantic retrieval is enabled

`probe` is intentionally non-mutating. It helps operators and assistant hosts
decide whether the next honest step is bootstrap, repair, or session work.

Planning and execution can also stop explicitly. The runtime may surface
planning gates such as `goal_quality_state`, `plan_quality_state`,
`backlog_quality_state`, and `planning_analysis_state`, plus structured host
handoffs such as `phase_request`, `assistant_resume_command`, and
`assistant_next_command`.

Plan quality is now the first planning-readiness gate in the 0.68.0 line. If
the active plan lacks a credible validation strategy or another blocking
planning input, the runtime keeps planning non-terminal, emits one
`phase_request`, and preserves the blocked assessment in status, inspect, and
orchestration snapshots until the operator answers.

## Host Surface Boundary

The CLI and generated assistant command packs are thin shells over the same
runtime.

- `init`, `goal`, `plan`, and `run` are the main state-changing commands.
- `status`, `next`, `inspect`, and `probe` are read-side runtime projections.
- Host packages must read `.boundline/session.json`, traces, and CLI output
  rather than treating chat history as authoritative state.

These surfaces may render summaries, evidence, findings, checkpoints, and
read-only governed references, but they do not own delivery state.

## Framework Adapter Boundary

Framework adapters extend the runtime without replacing it.

- Boundline remains the orchestrator and the default execution path.
- One workspace may select one adapter or none.
- The host owns capability validation, config persistence, routing decisions,
  and operator-visible status or inspect output.
- Adapters only own the stages they explicitly declare and successfully claim.

The V1 wire contract is deliberately bounded:

- one-shot trusted local subprocess commands only
- UTF-8 JSON over stdin/stdout only
- one standard success or error envelope on stdout for every command
- optional structured stderr lines that may be copied into traces, but never
  change result classification on their own
- no graceful shutdown, background daemon, or persistent transport lifecycle in
  this release

This makes transport inspection a first-class operator concern. The host can
block activation or fall back before claim when `describe` does not declare a
compatible stdin/stdout transport, and `adapter show --json` is the stable
surface for confirming that compatibility before a stage runs.

The shipped Speckit profile keeps ownership boundaries explicit. `goal` remains
Boundline-native, `plan` may be adapter-owned through workflow ID
`speckit-planning`, `run` may be adapter-owned through workflow ID
`speckit-implementation`, and `status` plus `inspect` remain Boundline-owned.
The runtime launches the split workflow assets
`.specify/workflows/speckit/planning.yml` and
`.specify/workflows/speckit/implementation.yml` by local YAML path, while the
persisted stage outcome and operator surfaces continue to use the semantic
workflow IDs.

A claimed Speckit `plan` stage has a bounded readiness loop: one initial
`speckit.analyze` pass plus at most two remediation or analyze re-check cycles.
If blocking findings remain after that budget, the adapter must return a
blocked outcome with the remaining findings and recovery guidance. A claimed
Speckit `run` stage is implementation-only and must not rerun planning
commands. This keeps the host's stage routing and the adapter's command surface
aligned to the same contract.

## Planning, Guidance, And Traceability

Planning in Boundline is evidence-driven:

- `goal` persists the bounded session objective from authored goals and briefs.
- `plan` builds one bounded context pack from workspace evidence, recent traces,
  and compatible Canon inputs.
- `run` executes bounded actions on that same session-owned runtime.
- `status`, `next`, and `inspect` project the persisted route, context,
  follow-through, and findings instead of recomputing a new story.

The same context pack also drives guidance and guardian selection. Capability
precedence, loaded and skipped sources, validation findings, and blocking
outcomes stay traceable through the runtime outputs.

## Review And Reasoning Boundaries

Boundline currently exposes two runtime-owned algorithm families:

- review-council assembly, independence guarding, vote resolution, and bounded
  adjudication projection
- reasoning-profile activation, independence assessment, bounded profile
  outcomes, and confidence handoff

These are part of the same session runtime. They do not create a second
orchestration system.

## Compatibility Path

Boundline still supports explicit compatibility behavior, but it is subordinate.
Use it when you intentionally want a manifest-backed execution profile:

```bash
boundline run --compatibility --goal "..."
```

That path is explicit. It is not the default product story.

## Routing, Workflows, And Clusters

Routing, workflow entrypoints, and clustered delivery all sit on top of the
same runtime rather than redefining it.

- `config` controls effective routing, capability policy, effort policy, and
  assistant bindings.
- `workflow` provides named entrypoints over the same session-owned runtime.
- `cluster` lets one primary workspace own the authoritative session for a
  bounded change that spans multiple repositories.

These are product layers over one runtime, not separate products.

## Distribution And Update Model

The release surface keeps Boundline and Canon pairing metadata explicit.

- `distribution/channel-metadata.toml` carries the release-aligned pairing.
- generated Homebrew and winget artifacts stay aligned to the same metadata.
- `boundline doctor --install` verifies the running Boundline version, the
  supported Canon target, and the current pairing state.

The pairing states stay explicit: `ready`, `already_satisfied`, `blocked`, or
repair-needed.

## When To Read More

- [configuration.md](configuration.md) for config precedence and auth/profile
  scope
- [assistant/README.md](../assistant/README.md) for assistant command-pack
  behavior
- [review-voting.md](review-voting.md) for review-council follow-through
- [reasoning-profile-algorithms.md](reasoning-profile-algorithms.md) for
  reasoning-profile behavior
