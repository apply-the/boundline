# Changelog

All notable changes to Synod are documented in this file.

This changelog is reconstructed from feature-spec delivery under `specs/`, the
workspace version bumps recorded in `Cargo.toml`, and the corresponding release
commits in git history. The repository does not currently use release tags, so
each release below maps a published version to the spec directories first
introduced between that version bump and the previous one.

Synod follows Semantic Versioning. Before `1.0.0`, breaking changes may occur
in minor releases.

The repository history contains no release bumps for `0.2.0`, `0.6.0`,
`0.12.0`, or `0.14.0`, so the adjacent feature slices are rolled into the next
recorded workspace version.

## [Unreleased]

Delivered specs:

- None recorded after `0.24.0`

Highlights:

## [0.24.0] - 2026-05-01

Delivered specs:

- `024` - Unify Route Summaries And Config Projection

Highlights:

- Add explicit `route_owner` and shared `route_config_projection` cues across
  `run`, `status`, `next`, `inspect`, and compatibility follow-up so aligned
  summary wording no longer hides which route currently owns the work.
- Project material route inputs such as workspace-local routing defaults,
  workflow or flow cues, and requested governance intent onto the same
  operator-facing summary family while preserving `continuity_authority` and
  explicit compatibility ownership.
- Update README, getting-started, configuration, adaptive-execution,
  assistant guidance, roadmap, contributor docs, and changelog for the
  release.

## [0.23.0] - 2026-05-01

Delivered specs:

- `023` - Broaden Bounded Adaptive Repair

Highlights:

- Broaden bounded adaptive repair beyond arithmetic, comparison, and boolean
  flips with new deterministic local families for ordering boundaries, result
  status, and bounded numeric literals.
- Surface adaptive `candidate_family`, selection credibility, rejected
  candidates, and explicit exhaustion reasons through `run`, `status`, `next`,
  and `inspect` while keeping the route explicitly on the compatibility path.
- Stop adaptive replans explicitly when validation evidence is absent or
  insufficient for another materially different bounded candidate, and update
  README, getting-started, configuration, adaptive-execution, assistant
  guidance, roadmap, contributor docs, and changelog for the release.

## [0.22.0] - 2026-05-01

Delivered specs:

- `022` - Session Compatibility Continuity

Highlights:

- Keep `status` and `next` usable after explicit compatibility `run` by
  surfacing `continuity_authority`, inspect-only compatibility follow-up, and
  the correct CLI-reported inspect command instead of failing on a missing
  active session.
- Reuse one route and `execution_condition` summary vocabulary across native
  session and compatibility follow-up surfaces without hiding which route ran.
- Update README, getting-started, configuration, adaptive-execution,
  assistant guidance, roadmap, contributor docs, and changelog for the
  continuity release.

## [0.21.0] - 2026-05-01

Delivered specs:

- `021` - Adaptive Repair Depth

Highlights:

- Re-rank bounded adaptive repair candidates from the latest validation record
  and failure evidence so replans can shift to a new manifest-declared target
  when the current slice is no longer credible.
- Keep adaptive execution on the explicit compatibility path while surfacing the
  latest workspace slice, validation-guided selection headline, and attempt
  lineage more clearly in `run` and `inspect`.
- Update README, adaptive-execution, getting-started, configuration, assistant
  guidance, roadmap, and contributor docs for the adaptive-repair-depth
  release.

## [0.20.0] - 2026-05-01

Delivered specs:

- `020` - Governed Stage Depth

Highlights:

- Extend the primary session-native route to govern `bug-fix:investigate`
  before later governed verify work, while keeping Canon bounded to stage
  governance rather than orchestration ownership.
- Preserve packet reuse lineage, approval refresh, and explicit blocked-state
  guidance across `run`, `status`, `next`, `inspect`, and workflow-aware
  projection surfaces.
- Allow `inspect` to summarize paused or blocked governance traces instead of
  failing when the trace is still running.
- Update README, getting-started, configuration, assistant guidance, roadmap,
  and contributor docs for the governed-stage-depth release.

## [0.19.0] - 2026-05-01

Delivered specs:

- `019` - Workflow Follow-Through

Highlights:

- Make bounded `review` and `govern` phases executable from `synod workflow`
  so named workflows can complete or stop in explicit paused, blocked, failed,
  or completed states on the same session-owned route.
- Add `synod workflow list` plus optional workflow-registry discovery metadata
  (`summary`, `recommended_when`) so operators and assistants can choose the
  correct named workflow without reading raw registry files.
- Keep direct session-native commands and explicit compatibility routing
  available when no named workflow is invoked, even when a workspace defines
  `.synod/workflows.toml`.
- Update README, getting-started, configuration, assistant guidance, roadmap,
  and contributor docs for the completed workflow follow-through slice.

## [0.18.0] - 2026-04-30

Delivered specs:

- `018` - Workflow Layer

Highlights:

- Add workspace-local `.synod/workflows.toml` as a bounded named-workflow
  registry compiled onto Synod's existing session-native phases.
- Add `synod workflow run`, `status`, `resume`, and `inspect` so named
  workflows reuse the same session, routing, trace, and next-command story as
  direct session-native delivery work.
- Persist workflow identity, active phase, lifecycle pauses, and next-action
  guidance in the active session while rejecting unsupported workflow semantics
  explicitly.
- Preserve direct session-native commands and explicit compatibility routing
  when no named workflow is invoked.

## [0.17.0] - 2026-04-29

Delivered specs:

- `017` - Canon Governance Expansion

Highlights:

- Add verify-stage Canon `security-assessment` coverage for the bounded
  `bug-fix` and `change` governed analysis path.
- Keep governed security analysis on the primary session-native route so
  `run`, `status`, `next`, and `inspect` continue to share one routing and
  execution-condition story.
- Surface selected Canon mode, approval state, packet provenance, and next
  action coherently across session and trace summaries.
- Update the documented Canon compatibility target to `0.25.0`.

## [0.16.0] - 2026-04-29

Delivered specs:

- `016` - Session-Native Surface Unification

Highlights:

- Unify `run`, `status`, `next`, and `inspect` around one session-owned
  summary model with consistent route, flow, and decision reporting.
- Surface review, adaptive execution, and governance state as bounded
  extensions of the same session-native operator story instead of separate
  runtime modes.
- Keep the compatibility path explicit when it is chosen without letting it
  overwrite the primary session-native explanation.

## [0.15.0] - 2026-04-29

Delivered specs:

- `015` - Runtime Refoundation

Highlights:

- Refound Synod around `start -> capture -> plan -> run -> status -> inspect`
  as the primary operator journey for bounded delivery work.
- Treat flow as confirmed policy constraints over bounded decisions rather than
  as a rigid script, while preserving failure evidence and recovery state for
  later inspection.
- Demote declarative execution profiles to an explicit compatibility path and
  keep Canon as a bounded stage-boundary input instead of the orchestration
  control plane.

## [0.13.0] - 2026-04-29

Delivered specs:

- `012` - Multi-Workspace Orchestration
- `013` - Session-Native Orchestrator
- `014` - Native Loop Integration

Highlights:

- Add cluster-aware multi-workspace registration, configuration precedence,
  and cross-workspace status or trace inspection.
- Introduce the session-native observe-decide-act-verify loop with bounded
  goal-derived planning, inferred flow confirmation, and inspectable decision
  objects.
- Route planned sessions through the real adapter-backed decision loop by
  default while preserving declarative execution manifests as an explicit
  compatibility path.

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