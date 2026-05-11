# Boundline Advanced Architecture

This document is the second read level for Boundline. Read it after the quick path
in the README and [getting-started.md](getting-started.md) if you need the
deeper product model.

## Boundline Versus Canon

Keep this boundary explicit:

- Boundline owns orchestration, bounded planning, execution, validation, session
  continuity, and the primary operator-facing CLI.
- Canon owns governed stages, approvals, structured governed artifacts, and the
  machine-facing governance adapter when a Boundline route explicitly enables it.

Canon is not the orchestrator and not the product entrypoint. A Boundline install
can be perfectly usable without Canon when you stay on the default local and
session-native routes.

The current Boundline adapter documents Canon `0.45.0` support for the
`canon governance start|refresh|capabilities --json` `v1` surface. That is a
bounded compatibility target, not a claim of total Canon feature parity.

## Primary Runtime Model

The primary operator journey is still session-native:

1. `start`
2. `capture`
3. `plan`
4. `run`
5. `status`
6. `next`
7. `inspect`

Boundline persists that story in workspace-local state under `.boundline/` and keeps
traces alongside the same session model. `run`, `status`, `next`, and
`inspect` project the same route, follow-through, and evidence story instead of
making the operator infer state from logs.

## Compatibility Path

Boundline still supports explicit compatibility behavior, but it is subordinate.
Use it when you intentionally want a manifest-backed execution profile. Do not
treat it as the default product path.

That is why the quick path centers on `doctor -> start -> capture -> plan ->
run` rather than `init` or `.boundline/execution.json` authoring.

## Planning And Bounded Context

Planning in Boundline is evidence-driven:

- `capture` persists negotiated delivery state from authored goals and optional briefs.
- `plan` builds one bounded context pack from workspace evidence, authored input,
  recent traces, and any reusable Canon artifacts.
- authored brief file refs, failing validation paths, recent changed files, and
  other explicit evidence anchors are causal inputs; broad path similarity is
  only a bounded tie-breaker.
- Planning stops explicitly when the negotiation result or bounded context is
  not credible enough to support a real bounded change.

That explicit stop behavior is a feature, not an inconvenience. Boundline should
stop rather than pretend a plan is credible when the evidence is weak.

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

The `0.43.0` release keeps the repo-managed distribution surface introduced in
`0.39.0`, carries the same Boundline-plus-Canon pairing metadata, and now makes
Canon-ready setup, verification, and governed runs part of the same primary
operator surface:

- `distribution/channel-metadata.toml` pins the release-aligned Boundline plus Canon pairing and the tap-facing Homebrew channel metadata.
- `scripts/sync-distribution-metadata.sh` regenerates the tap-ready Homebrew formula and the winget manifests.
- `.github/workflows/sync-homebrew-tap.yml` syncs the generated Homebrew formula into `apply-the/homebrew-boundline`.
- `.github/workflows/release-windows-distribution.yml` builds the Windows release bundle that still carries both `boundline` and `canon` for the winget package surface.
- `boundline doctor --install` verifies the installed Boundline version, the supported Canon target, and the current pairing state.
- `boundline init`, `config show`, `config set-canon`, and the assistant command packs keep Canon mode-selection and governed entry guidance aligned with the CLI.
- `.boundline/checkpoints/` keeps local rollback manifests for mutating `run` and `step` without making Git a prerequisite.

The supported pairing states are explicit:

- `ready`: bundled Canon matches the documented support target.
- `already_satisfied`: a compatible Canon was already present on PATH.
- `blocked`: Boundline cannot determine a safe supported state.
- `repair_needed`: the machine is close to usable but the user needs to repair
  the Canon pairing or reinstall through the supported path.

Source install remains the fallback path when Homebrew or winget is not the
right fit for the current machine.

## Why The Docs Split Matters

The quick path exists so operators can install Boundline, verify the pairing, and
run one bounded session without absorbing the entire architecture first.

This document exists so advanced readers can understand the deeper model
without blurring the boundary between:

- first-run operator guidance
- bounded routing and follow-through design
- Canon as the governed companion rather than the orchestrator

When those layers blur, the product becomes harder to adopt and harder to
explain. The split is deliberate.