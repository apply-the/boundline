# Changelog

All notable changes to Synod are documented in this file.

This changelog is reconstructed from feature-spec delivery under `specs/`, the
workspace version bumps recorded in `Cargo.toml`, and the corresponding release
commits in git history. The repository does not currently use release tags, so
each release below maps a published version to the spec directories first
introduced between that version bump and the previous one.

Synod follows Semantic Versioning. Before `1.0.0`, breaking changes may occur
in minor releases.

The repository history contains no release bumps for `0.2.0` or `0.6.0`, so
the adjacent feature slices are rolled into the next recorded workspace
version.

## [Unreleased]

Delivered specs:

- None recorded after `0.11.0`

Highlights:

## [0.11.0]

Delivered specs:

- `011-init-model-routing`

Highlights:

- Added `synod init` to scaffold bounded workspace defaults under `.synod/`
  without hand-authoring setup files.
- Added `synod config show|set|unset` for runtime/model routing at global and
  workspace scope.
- Added deterministic routing precedence (`CLI > workspace > global > built-in`)
  with effective-source visibility.
- Added initial runtime setup surface for Claude, Codex, Copilot, and Gemini
  CLI (CLI-only for Gemini in this slice).

## [0.10.0]

Delivered specs:

- `010-human-brief-ingestion`

Highlights:

- `synod capture` and `synod run` accept one or more `--brief <path>.md`
  arguments alongside (or instead of) `--goal`. Brief contents are normalized
  into a single goal text projected through the existing capture pipeline so
  developers no longer need to author free-text prose only on the command line.
- New `synod::domain::brief` module (`AuthoredBriefBundle`,
  `InputSourceReference`, `BriefIngestionError`, `normalize_inputs`) enforces
  workspace-bounded `.md`/`.markdown` sources, an upper bound of 10 brief
  files per invocation, and a 256 KiB per-source size cap.
- Multi-source resolution deduplicates explicit and referenced Markdown input
  into one persisted authored brief bundle with stable provenance across
  `capture`, `run`, `status`, and `inspect`.
- Clarification-aware task drafting blocks planning explicitly for unbounded
  requests and records an inspectable trace instead of guessing missing scope.
- Human governance intent (`--governance`, `--risk`, `--zone`, `--owner`)
  maps into the existing governed execution path and surfaces next-action
  guidance for blocked or approval-gated runs.

## [0.9.0] - 2026-04-27

Delivered specs:

- `009` - Canon Governance Adapter

Highlights:

- Add stage-level governance runtime selection between local execution and the
  Canon CLI.
- Record Canon run refs, packet readiness, approval state, packet provenance,
  and autopilot decision evidence across `run`, `status`, `next`, and
  `inspect`.
- Block explicitly when required governance cannot proceed and refresh approval
  state through later `step`, `run`, or `status` cycles.

## [0.8.0] - 2026-04-26

Delivered specs:

- `007` - Multi-Agent Review
- `008` - Adaptive Execution Engine

Highlights:

- Add bounded reviewer councils with findings, majority or weighted vote
  resolution, optional adjudication, and trace-visible review outcomes.
- Add adaptive workspace-slice selection, deterministic candidate synthesis,
  signature-based non-repeat behavior, and bounded replanning after failed
  validation.
- Surface both review and adaptive execution evidence through `run`, `status`,
  `next`, and `inspect`.

## [0.7.0] - 2026-04-26

Delivered specs:

- `006` - Execution Engine

Highlights:

- Add workspace execution manifests under `<workspace>/.synod/execution.json`
  with fallback to the legacy `<workspace>/.synod/fixture.json` shape.
- Let Synod apply bounded workspace changes, run validation commands, and take
  explicit retry or replan paths based on manifest policy.
- Persist changed-file evidence, validation results, and terminal outcomes in
  session state and file-backed traces.

## [0.5.0] - 2026-04-25

Delivered specs:

- `005` - Delivery Flows

Highlights:

- Add built-in `bug-fix`, `change`, and `delivery` flows with stage-aware task
  progression and recovery.
- Surface flow and stage context through the session-native CLI.
- Keep execution bounded while making stage transitions, retries, replans, and
  failures explicit in traces.

## [0.4.0] - 2026-04-25

Delivered specs:

- `004` - Session Model Unification

Highlights:

- Persist active workspace session state under `<workspace>/.synod/session.json`.
- Unify `start`, `capture`, `plan`, `step`, `run`, `status`, `next`, and
  `inspect` around one session-native CLI workflow.
- Tighten session validation and status projection so operators can resume work
  without reconstructing task state from raw traces.

## [0.3.0] - 2026-04-25

Delivered specs:

- `002` - Developer UX Orchestrator
- `003` - Assistant Command Packs

Highlights:

- Add the first local developer CLI surface for driving the orchestrator from
  the repository root.
- Ship assistant-native command packs for Copilot, Codex, and Claude as thin
  frontends over the same local CLI.
- Keep assistant workflows aligned with repo-local commands and trace-aware
  follow-up guidance instead of introducing a second runtime surface.

## [0.1.0] - 2026-04-24

Delivered specs:

- `001` - Delivery Orchestrator Core

Highlights:

- Introduce the bounded delivery orchestrator core as a Rust library crate.
- Establish deterministic planning, registry-based execution endpoints,
  bounded retries, bounded replanning, and explicit terminal states.
- Persist execution traces under `<workspace>/.synod/traces/` as the foundation
  for later CLI, session, and delivery surfaces.