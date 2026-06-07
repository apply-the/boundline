# Changelog

All notable changes to Boundline are documented in this file.

This changelog is reconstructed from feature-spec delivery under `specs/`, the
workspace version bumps recorded in `Cargo.toml`, and the corresponding release
commits in git history. The repository does not currently use release tags, so
each release below maps a published version to the spec directories first
introduced between that version bump and the previous one.

Boundline follows Semantic Versioning. Before `1.0.0`, breaking changes may occur
in minor releases.

The repository history contains no release bumps for `0.2.0`, `0.6.0`,
`0.12.0`, or `0.14.0`, so the adjacent feature slices are rolled into the next
recorded workspace version.

## [Unreleased]

Highlights:

- Corrected the shipped Speckit adapter semantics around the split workflow
  boundary: `plan` now routes through `speckit-planning`, `run` routes through
  `speckit-implementation`, the planning readiness loop remains explicit and
  bridge-owned, and the legacy combined workflow surface plus hidden fallback
  have been retired from the active packet.
- Recorded a 2026-06-01 post-correction validation rerun across the host,
  template, and Speckit repos, including a passing `/speckit.analyze` result
  for the corrected 066 packet slice.
- Retired the unshipped terminal UI planning artifacts and removed the related
  product-line references from the active roadmap.
- Added `boundline models auth login|status|remove` so provider-backed routes
  can reuse user-scoped authentication outside any one repository; the current
  public login surface supports `github-copilot`.
- Aligned runtime planning handoffs around explicit gate and follow-up outputs,
  including planning-quality projections and assistant-safe resume or next
  commands instead of implicit host continuation.
- Added the read-only `boundline probe` preflight surface, plus bootstrap-safe
  assistant guidance and contract coverage so goal and plan hosts can detect
  init versus doctor versus session-ready states before orchestration.
- Closed the remaining cross-host assistant contract gaps so Copilot prompt
  packs and repo-local Claude, Codex, and Antigravity command surfaces describe
  the same probe and follow-through behavior.
- Realigned the active Canon compatibility boundary and related reasoning
  contracts to Canon `0.67.0`.

Release metadata note:

- The current Boundline release line is `0.74.0`; published package metadata,
  distribution metadata, assistant plugin manifests, and docs are aligned to
  that version while Canon compatibility remains `0.71.0`.

## [0.75.0] - 2026-06-07

Delivered specs:

- `075` - Adaptive Governance Calibration

Highlights:

- Added graduated control levels (advisory → catch → rule → hook) with
  calibration policy stored in `.boundline/calibration-policy.toml`.
- Added `boundline override` command for operator bypass of catch and rule
  findings with trace-visible override records.
- Added guardian trust metric accumulation (true/false positive counting,
  TPR/FPR computation) with configurable evidence window for promotion/demotion.
- Added degradation rules (provider/model/tool unavailability) and escalation
  triggers (repeated unresolved, red zone, low-confidence/high-impact).
- Integrated calibration into `boundline council adjudicate` and `boundline inspect`
  with human-readable and JSON output.

## [0.74.0] - 2026-06-06

Delivered specs:

- `074` - Review Councils And Governance

Highlights:

- Added `boundline council adjudicate`, a guardian-activation-router CLI that
  reads `.boundline/guardian-rules.toml` (or falls back to built-in defaults),
  activates the correct guardians for the current change surface, and produces a
  clean/blocked council decision with human-readable and JSON output.
- Added guardian activation ruleset with file, stage, language, and risk filters
  plus contradiction detection that fails closed on conflicting rules.
- Added single-adjudicator council decision model with accepted/rejected/deferred
  findings, mandatory guardian evidence rules, and binary clean/blocked outcome.
- Emitted `council.decision.produced` structured runtime event for trace
  visibility.

## [0.73.0] - 2026-06-05

Delivered specs:

- `072` - Evals And Runtime Observability
- `073` - Contextual Help And Documentation Architecture (Boundline)

Highlights:

- Added `boundline help-next`, a read-only operator guidance command that
  inspects workspace state across six lifecycle phases (uninitialized,
  initialized, active, blocked, failed, ready) and returns the next recommended
  action with an exact command, reason, and documentation link via a versioned
  `.boundline/help-links.toml` link map.
- Added `--json` flag for CI/automation consumption and `--all` flag for
  complete diagnostics listing.
- Added `boundline.help_next.requested` structured event to the runtime event
  vocabulary for operator-friction observability.
- Added a local eval suite covering planning quality, context selection quality,
  guardian finding quality, council rejection behavior, provider call failure
  handling, and trace compaction survival of accepted decisions and rejection
  reasons, runnable both locally and in CI with per-eval pass/fail summaries.
- Added a trace compaction policy with five retention classes (lossless,
  structured, summary, index-only, discardable), conservative tiebreaking, hard
  survival guarantees for accepted decisions and rejection reasons, and
  oversized-trace detection.
- Added structured runtime event vocabulary with per-event-type `schema_version`,
  JSONL export with event deduplication, sensitive-data filtering, and empty-
  export handling.
- Added runtime metrics collection (compaction counts, class distribution, trace
  sizes, context dimensions, provider latency, stop reasons, finding counts)
  exposed through status, inspect, and JSONL export surfaces.

## [0.72.0] - 2026-06-05

Delivered specs:

- `071` - Capability Provider Protocol

Highlights:

- Added the first Boundline-owned external capability-provider protocol with
  explicit operator registration, setup-requirement projection, activation, and
  readiness dry-runs before a provider may be trusted at runtime.
- Added bounded `capabilities`, `health`, `prepare`, `execute`, and
  `collect_evidence` transport flows over command or HTTP providers while
  keeping provider output proposal-only until Boundline validates the result.
- Enforced fail-closed permission admission, lifecycle-phase support,
  metadata-conflict handling, and accepted-versus-rejected evidence tracking so
  provider-backed execution cannot silently mutate Boundline-owned state.
- Surfaced additive provider identity, activation state, capability IDs, setup
  requirements, failure class, and evidence projections through status,
  inspect, host JSON, traces, assistant assets, and release-aligned docs for
  Boundline `0.72.0` and Canon `0.67.0`.

## [0.71.0] - 2026-06-05

Delivered specs:

- `070` - Large Codebase Context Substrate

Highlights:

- Added a typed large-codebase context substrate with fidelity tiers, inclusion
  modes, omission findings, repository-map state, digest-backed artifact refs,
  and patch-safe edit attempt projections.
- Blocked planning when critical context is missing, downgraded, omitted, or
  would require unsafe oversized full reads, while preserving the stop reason
  through plan quality, status, inspect, traces, and assistant surfaces.
- Surfaced large-repository context decisions explicitly across status and
  inspect, including repository-map readiness, snapshot-cache freshness,
  compacted artifacts, omission reasons, and patch-safe edit constraints.
- Kept the snapshot cache explicitly derived and non-authoritative while
  aligning release metadata, docs, assistant assets, Homebrew, and WinGet to
  Boundline `0.71.0` and Canon `0.67.0`.

## [0.70.0] - 2026-06-04

Delivered specs:

- `069` - Plan Analysis Contract

Highlights:

- Added the first read-only planning-analysis gate after plan quality and
  backlog quality, so execution handoff now depends on end-to-end planning
  coherence rather than backlog readiness alone.
- Persisted additive `planning_analysis_state`,
  `planning_analysis_findings`, and `planning_analysis_coverage` through
  status, inspect, traces, orchestration, and assistant-facing runtime
  projections.
- Blocked execution handoff on uncovered success criteria, missing validation
  coverage, selected-slice sequencing contradictions, missing execution-handoff
  inputs, and explicit governed producer contract gaps.
- Kept Canon compatibility aligned to `0.67.0` while updating release
  metadata, docs, roadmap surfaces, Homebrew, WinGet, and assistant manifests
  to Boundline `0.70.0`.

## [0.69.0] - 2026-06-04

Delivered specs:

- `068` - Backlog Contract

Highlights:

- Added the first formal backlog-quality planning gate after plan quality and
  before planning analysis or execution handoff.
- Switched backlog admission from the legacy checklist-style `backlog.md`
  heuristic to the Canon multi-document backlog packet, including
  closure-limited packet blocking and full-packet clarification for missing
  execution handoff evidence.
- Preserved additive backlog-quality projections across status, inspect,
  orchestration, traces, and assistant plan or run surfaces.
- Aligned the workspace, assistant manifests, distribution metadata, Homebrew,
  WinGet, and reasoning compatibility surfaces to Boundline `0.69.0` and Canon
  `0.67.0`.

## [0.68.0] - 2026-06-03

Delivered specs:

- Ollama-ready init presets

Highlights:

- Added `boundline init --ollama-profile small|medium|large` to seed local
  Ollama model routes for planning, implementation, verification, and review.
- Tuned `ollama-small` for Apple Silicon with 16 GB unified memory using
  Qwen2.5 7B models.
- Tuned `ollama-medium` for 64 GB local workstations using Qwen2.5 14B models.
- Tuned `ollama-large` for 96/128 GB unified-memory or workstation hosts using
  Qwen2.5 Coder 32B and Qwen2.5 72B.
- Released aligned Boundline `0.68.0` workspace, docs, assistant metadata,
  Homebrew, and WinGet metadata.

## [0.67.0] - 2026-06-02

Delivered specs:

- `067` - Plan Quality Contract

Highlights:

- Added the first planning-readiness gate so plans without a credible
  validation strategy stop before execution handoff.
- Persisted additive `plan_quality_state`, `plan_quality_findings`, and
  `plan_quality_assumptions` through status, orchestration, inspect, and trace
  projections.
- Preserved the one-question `phase_request` recovery path across the CLI and
  assistant surfaces.
- Released aligned metadata, docs, roadmap entries, and bundled manifests for
  the 0.67.0 line.

## [0.66.0] - 2026-05-31

Delivered specs:

- `066` - Agentic Framework Integration

Highlights:

- Added one explicit framework-adapter slot per workspace with operator-facing
  `boundline adapter add|show|remove` commands, known-profile activation for
  `speckit`, and custom-adapter registration with guided required-field setup.
- Formalized the V1 framework-adapter subprocess contract around one-shot JSON
  over stdin/stdout, standard stdout success or error envelopes, declared
  supported transports, and structured stderr that remains trace-only
  enrichment instead of changing ownership or result classification.
- Extended runtime routing, status, inspect, host JSON, and config output so
  adapter execution source, claimed-stage ownership, hook delivery, transport
  compatibility, config completeness, and guided-setup recovery stay explicit.
- Bootstrapped the sibling `boundline-framework-template` and
  `boundline-adapter-speckit` repositories with contract-tested setup,
  preflight, partial override, and failure-example scaffolds aligned to the
  released compatibility line.

Validation notes:

- Focused host contract and integration suites passed for the released
  compatibility line, including `framework_adapter_protocol_contract`,
  `runtime_routing_contract`, `framework_adapter_activation`,
  `framework_adapter_override_flow`, and `framework_adapter_config_flow`.
- Cross-repo sibling validation passed for the template and Speckit repos with
  `cargo test --manifest-path ../boundline-framework-template/Cargo.toml --test contract`,
  `cargo test --manifest-path ../boundline-adapter-speckit/Cargo.toml --test contract`,
  and `cargo test --manifest-path ../boundline-adapter-speckit/Cargo.toml --test config_flow`.
- The release evidence confirms V1 stdio transport declarations, standard
  stdout-envelope handling, unsupported-transport blocking before stage claim,
  stderr remaining trace-only even when present, and the explicit no-graceful-
  shutdown scope of the one-shot command surface.
- The provider-catalog refresh remained a no-change result for `0.66.0`; the
  catalog update from 2026-05-30 already covered the current supported model
  families.

## [0.65.0] - 2026-05-30

Delivered specs:

- `065` - Activate SQLite Vec And DB Merge Strategy

Highlights:

- Activated the sqlite-vec-backed local semantic retrieval path over the
  existing single derived SQLite store while preserving bounded authority order,
  explicit fallback reasons, and traceable selected or rejected evidence.
- Added manifest-backed `boundline index status|refresh|rebuild|clean|doctor`
  lifecycle commands, incremental refresh behavior, and clear stale,
  incompatible, degraded, and corrupt recovery guidance.
- Formalized derived-index hygiene around `.boundline/context-intelligence/`
  with manifest sidecars, explicit WAL/SHM handling, tracked-file diagnosis,
  and optional lightweight Git stale-mark hooks for checkout, merge, and
  rewrite events.
- Extended operator-facing `status`, `inspect`, `probe`, `init`, and
  diagnostics surfaces so semantic capability, lifecycle state, fallback
  disclosure, and recovery actions remain explicit.

## [0.64.0] - 2026-05-20

Highlights:

- Added a separate `boundline-terminal` workspace component for dashboard
  snapshot projection and terminal rendering groundwork while keeping normal
  CLI commands and session-native state authoritative.
- Introduced typed dashboard snapshots, action contracts, diagnostics, and
  degraded fallbacks over existing session, trace, checkpoint, finding, and
  governed-reference projections.
- Added initial release documentation for dashboard launch, snapshot JSON
  validation, degraded mode, and terminal-safe `boundline` wordmark branding.
- Bumped release metadata to Boundline `0.64.0`, Canon `0.61.0`, and refreshed
  the bundled assistant model catalog from current provider documentation.

Validation notes:

- Dashboard-targeted contract, integration, unit, and crate-local UI suites all
  passed; `cargo nextest run` also exited `0` for the full workspace.
- `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
  produced file-level LCOV evidence above the repository target for the new
  dashboard files, including `36/37` lines in `crates/boundline-terminal/src/app.rs`
  and `382/385` lines in `src/adapters/dashboard_state.rs`.
- In this environment the bare `cargo test` harness still stalls after
  compilation, so release evidence relies on `cargo test --no-run --all-targets`
  plus the targeted dashboard suites for deterministic validation.

## [0.63.0] - 2026-05-19

Delivered specs:

- `063` - Assistant Delight Follow-Through

Highlights:

- Extended the S7 delight surfaces so `status` and `inspect` now disclose the
  active reasoning profile, the selection reason, the contribution summary,
  and the fallback disclosure from authoritative session and trace state.
- Closed the remaining inspect follow-through gaps with human-facing
  `inspect_context`, `inspect_council`, and `inspect_timeline` views backed by
  the flattened trace model.
- Added session-scoped delight usefulness signals, including
  `time_to_first_useful_answer_ms`, explanation attribution rate, and next
  action acceptance rate, without introducing a second telemetry runtime.
- Made Cursor and Gemini support modes explicit across assistant metadata,
  package docs, and global bootstrap guidance while keeping the CLI
  authoritative.

## [0.62.0] - 2026-05-18

Delivered specs:

- `062` - Reasoning Profile Closure

Highlights:

- Closed the residual S6.1 reasoning-profile claims with terminal positive-path
  runtime evidence for `independent_pair_review`,
  `heterogeneous_security_review`, and `bounded_reflexion` inside the existing
  session-native governance loop.
- Froze debate as bounded substrate and adjudication as a shared reasoning
  primitive, and added typed capability classification checks so those claims
  stay aligned across runtime vocabulary and the 062 contract surface.
- Refactored the remaining governance-view and reasoning-independence
  maintainability hotspots without changing runtime semantics, and updated the
  runtime-facing tests to match the shipped completed-state behavior.
- Aligned the active release metadata, distribution surfaces, assistant package
  manifests, and Canon compatibility fixtures to Boundline `0.62.0` and Canon
  `0.60.0`.

## [0.61.0] - 2026-05-18

Delivered specs:

- `061` - Reasoning Profile Contracts

Highlights:

- Added the first S6 reasoning-profile runtime contract so governed stages can
  activate bounded challenge profiles inside the existing Boundline session
  runtime rather than spawning a second orchestrator.
- Persisted reasoning-profile state through session status, run output, and
  trace inspection, including blocked and degraded follow-up guidance.
- Added additive reasoning lifecycle and confidence trace emission, and kept
  contract-line drift blockers explicit with disagreement and remediation text
  on the existing `status` and `inspect` surfaces.
- Aligned the active release metadata to Boundline `0.61.0` and Canon `0.57.0`,
  including distribution surfaces, assistant package manifests, and Canon
  compatibility fixtures.

## [0.60.0] - 2026-05-17

Delivered specs:

- `060` - S7 Assistant Delight Layer

Highlights:

- Re-scoped the 060 feature line around the Boundline-side runtime and
  assistant-surface implementation for S7.
- Aligned the Boundline S7 feature to consume Canon provider signals through
  `057-s7-delight-provider` instead of treating 060 as a contract-only slice.
- Kept the S7 support artifacts under
  `specs/060-assistant-delight-layer/` as inputs to the runtime
  implementation and validation work.
- Bumped workspace version to `0.60.0`.

## [0.60.0] - 2026-05-17

Delivered specs:

- `059` - Semantic Acceleration

Highlights:

- Added the first bounded S5.v2 semantic-acceleration runtime slice on top of
  the shipped advanced-context baseline: Boundline now refreshes semantic
  chunks on the shared retrieval index and can expand or rerank the V1
  candidate set when local semantic capability is ready.
- Surfaced semantic policy, capability, hybrid outcome, match origin, and
  rejected semantic candidates through the normal `plan`, `status`, trace, and
  `inspect` projection path instead of a separate report.
- Added typed semantic trace records plus explicit Canon compatibility and skip
  reasons, while preserving Canon contract line and provenance metadata on the
  normal projection path.
- Refreshed focused coverage and validation evidence for the semantic runtime,
  including owner-local LCOV attribution for the retrieval, domain, and CLI
  output files.
- Bumped workspace version to `0.60.0`.

## [0.58.0] - 2026-05-16

Delivered specs:

- `058` - Advanced Context Intelligence

Highlights:

- Added the S5 V1 local advanced-context baseline with a workspace-local
  SQLite + FTS5 retrieval index and structured fallback ordering for bounded
  context assembly.
- Introduced typed advanced-context projection state for selected evidence,
  relationships, impact findings, retrieval budgets, and degraded or terminal
  outcomes.
- Persisted advanced-context output through goal plans, task context, session
  status, trace summaries, and CLI `status` or `inspect` rendering.
- Added typed configuration for disabled or local advanced-context policy and
  surfaced the effective policy through `boundline config show`.
- Realigned the 058 feature spec and implementation plan to the S5 V1 local
  SQLite + FTS5 baseline while deferring semantic acceleration to S5.v2.
- Bumped workspace version to `0.58.0`.

## [0.57.0] - 2026-05-16

Delivered specs:

- `057` - Adaptive Governance

Highlights:

- Added the full adaptive governance model (S4) with `GovernanceRuntimeState`
  stages (Advisory, Catch, Rule, Hook) and `GovernanceRolloutProfile` variants
  (minimal, guided, governed, strict).
- Introduced `GovernanceDegradationMode`, `GovernancePostureResolution`,
  `GovernanceTransitionDirection`, and `GovernanceStartupContext` types.
- Added `resolve_governance_posture_from_context` to derive the active posture
  from workspace and session evidence.
- Surfaced active governance state through `status`, `next`, `inspect`, and
  trace projection alongside existing bounded planning and review evidence.
- Added four new docs: `adaptive-governance`, `control-graduation-model`,
  `degradation-and-escalation`, and `runtime-confidence-and-calibration`.
- Refactored `SessionStatusView::validate` into 12 focused private helpers to
  reduce complexity (see `TD-001-complexity-hotspots.md`).
- Hardened workspace resolution with an explicit `NotFound` error for missing
  `--workspace` paths.
- Retired the S3 and S4 roadmap spec files; `ROADMAP.md` now marks S4 as
  fully delivered.
- Bumped workspace version to `0.57.0`.

## [0.56.0] - 2026-05-16

Delivered specs:

- `056` - Authority Zoned Councils

Highlights:

- Added typed authority council-profile and stop-semantics resolution aligned
  to the first-slice Canon `authority-governance-v1` contract.
- Added stage-aware authority control resolution, structured producer-response
  review records, and explicit council assembly outcomes with quorum and
  independence projection.
- Surfaced review council profile, independence state, and stop semantics
  through review trace payloads plus session-native `status` and `inspect`
  output.
- Bumped workspace version to `0.56.0` while keeping the verified Canon
  compatibility target on `0.53.0`.

## [0.55.0] - 2026-05-15

Delivered specs:

- `055` - Guidance Catalog And Guardian Rule Packs

Highlights:

- Added the guidance-catalog packaging, indexing, and validation slice so
  Boundline can consume contract-validated guidance packs instead of only flat
  pack manifests and loose Markdown collections.
- Introduced canonical pillar taxonomy, authority metadata, lifecycle labels,
  and normalized reference artifacts derived from the phase-7 roadmap addendum.
- Expanded the bundled guardian catalog from 10 to 16 entries: added
  python-type-safety, go-error-handling, api-contract-stability,
  migration-safety, concurrency-safety, and data-validation guardians.
- Added Dart language guidance and a dart-delivery pack for Flutter/Dart
  projects.
- Added ecosystem library recommendations to all language guidance files and
  recommended tools to testing guidance files.
- Added an extending-guidance-catalog guide documenting how to register new
  languages, guardians, and delivery packs.
- Bumped workspace version to `0.55.0` for the guidance-catalog feature branch.

## [0.54.0] - 2026-05-14

Delivered specs:

- `054` - Guidance And Guardian Capabilities

Highlights:

- Added the first typed guidance-and-guardian capability slice so Boundline can
  resolve shared, workspace-local, and optional Canon-governed engineering
  standards before planning and bounded execution.
- Introduced structured guardian findings, explicit degraded outcomes, and
  deterministic-before-LLM verification as bounded runtime behavior instead of
  implicit agent heuristics.
- Bumped workspace version to `0.54.0` for the guidance-and-guardian feature
  branch.

## [0.53.0] - 2026-05-14

Delivered specs:

- `053` - Expert Pack Selection

Highlights:

- Added the bounded expert-pack-selection slice so Boundline can project
  built-in expert packs and runtime-role recommendations before planning
  continues.
- Narrowed Canon integration to optional governed expertise inputs with an
  explicit `expertise_input` compatibility surface instead of generic runtime
  influence.
- Bumped workspace version to `0.53.0` and aligned active release metadata to
  Canon `0.52.0` for this feature branch.

## [0.52.0] - 2026-05-14

Delivered specs:

- `052` - Runtime Intelligence Substrate

Highlights:

- Formalize the local runtime substrate as the planning precondition for
  deterministic context packs, explicit credibility states, and inspectable
  substrate provenance.
- Extend session-native status, next, inspect, and run-trace surfaces to
  explain local-versus-Canon context selection with source-labelled
  provenance.
- Keep Canon enrichment optional while aligning active compatibility and
  distribution surfaces to Canon `0.51.0`.
- Revalidate the bundled model catalog against current provider docs and keep
  the bundled catalog accurate at `0.52.0` without adding new route families
  for this slice.
- Bumped workspace version to `0.52.0` across crates, assistant plugin
  metadata, and bundled catalog metadata.

## [0.51.1] - 2026-05-13

Highlights:

- Extend Canon project-memory consumption with explicit hard-stop conditions for
  blocked governance, missing required approval, and missing required source
  artifacts instead of collapsing those cases into generic stale memory.
- Surface Canon-memory consumer compatibility, Canon run refs, recommended next
  actions, and managed-block producer attribution through session-native
  status, inspect, and run-trace projections.
- Parse shared `project-memory:managed` blocks from repo-visible
  `docs/evidence/...` roots and preserve `producer` plus `source_ref`
  attribution in compacted Canon memory evidence summaries.
- Realign the documented and distributed Canon companion target to `0.50.0` and
  keep the release metadata on Boundline `0.51.1`.

## [0.51.0] - 2026-05-13

Delivered specs:

- `050` - Project Memory Delivery Integration

Highlights:

- Added consumer-side types for reading Canon-promoted project-memory and
  evidence surfaces: `PromotionStateView`, `LineageRef`,
  `CompatibilityOutcome`, `ProjectMemoryContext`, `ProjectMemorySurface`.
- Implemented a Canon-shaped project-memory reader for named `docs/project/*.md`
  surfaces with adjacent `<surface>.packet-metadata.json` sidecars, while
  deriving supporting evidence roots under `docs/evidence/<mode>/<RUN_ID>/`.
- Integrated current contract-version compatibility checking for Canon's active
  major-1 (`v1`) project-memory contract line and future major-version rejection.
- Stage planning now reuses stable Canon project memory through the existing
  compacted Canon memory projection while treating pending or evidence-only
  surfaces as non-authoritative.
- Session-native Canon memory summaries now carry project-memory artifact refs
  sourced from published `docs/project/*.md` and `docs/evidence/...` surfaces.
- Updated the documented and distributed Canon companion target to `0.50.0`.
- Bumped workspace version to `0.51.0`.

## [0.50.0] - 2026-05-12

Delivered specs:

- `049` - Boundline Project-Scale Delivery UX

Highlights:

- Define project-scale delivery as bounded decomposition from idea intake to
  verified code changes, including global assistant bootstrap before workspace
  initialization.
- Add the full Canon `0.45.0` mode set behind `/boundline:govern` and the
  Boundline-owned governed stage catalog, while keeping Canon as the packet
  runtime rather than the orchestrator.
- Scope review voting to risky quality boundaries and document the Delivery
  Pilot Model with the observe-decide-act-verify-update-context loop.

## [0.49.1] - 2026-05-11

Highlights:

- Fix guided `boundline init` so fresh workspaces create `.boundline/` before
  writing execution and config files, including repo-discovery flows that omit
  an explicit workspace override.
- Seed guided no-assistant route selection from the bundled catalog defaults
  instead of starting every slot unset.
- Expand the bundled GitHub Copilot runtime catalog to include Anthropic and
  Gemini models available through Copilot so model selection is not
  OpenAI-only.

## [0.49.0] - 2026-05-11

Delivered specs:

- `048` - Assistant Plugin Packages

Highlights:

- Add host-specific assistant package folders for Claude Code, Codex, Cursor,
  and a Copilot prompt-pack boundary so chat surfaces can discover Boundline as
  a session-native delivery runtime.
- Expose namespaced `/boundline:*` commands for start, goal, plan, run,
  status, inspect, recover, and conditional governance while keeping
  `.boundline/session.json` and CLI output authoritative.
- Validate plugin manifests, shared metadata, command coverage, referenced
  paths, version alignment, and prohibited positioning before release closeout.

## [0.48.0] - 2026-05-10

Delivered specs:

- `047` - Catalog Freshness, Independent Voting, and File-Backed Inputs

Highlights:

- Refresh the bundled assistant model catalog with the current mainstream
  route-capable models documented by OpenAI, Anthropic, and Google, then align
  built-in verification and review defaults to that refreshed bundle.
- Treat a single Markdown path or ordered Markdown-path array passed through
  prompted authored input as file-backed input instead of persisting the raw
  shorthand as direct goal text.
- Reject review councils that collapse onto the same effective runtime/model
  route and persist the resolved effective route for each counted reviewer.

## [0.47.0] - 2026-05-09

Delivered specs:

- `046` - Guided Init TUI and Runtime Catalog

Highlights:

- Replace the fragile guided `init` freeform prompts with a bounded terminal
  wizard that shows visible defaults, slot-by-slot route editing, and a final
  summary before writes.
- Ship a bundled runtime/model catalog with explicit source metadata, custom
  model fallback warnings, and assistant-pack scaffolding status grouped by
  selected surface.
- Add top-level `--version` support, non-interactive `init` parity, and
  spinner-style progress feedback for time-consuming bootstrap steps.
- Adopt Canon `0.43.0` as the supported Boundline companion across install
  diagnostics, distribution metadata, governed runtime snapshots, and operator
  guidance.

## [0.46.0] - 2026-05-09

Delivered specs:

- `045` - Chat-First Runtime

Highlights:

- Add a stable global `--json` host envelope for the session-native lifecycle,
  `run`, and `inspect` commands while keeping the rendered plain-text output
  as the default human-facing surface.
- Let shell-enabled assistant flows consume structured `session_status`,
  `trace_summary`, and `trace_location` fields directly, then align the Claude,
  Codex, Copilot, and Gemini guidance to the same structured shell path.
- Adopt Canon `0.43.0` as the supported Boundline companion across install
  diagnostics, distribution metadata, governed runtime snapshots, and operator
  guidance.

## [0.45.0] - 2026-05-08

Highlights:

- Adopt Canon `0.41.0` as the supported Boundline companion across install
  diagnostics, distribution metadata, governed runtime snapshots, and operator
  guidance.
- Push Homebrew tap updates directly to `homebrew-boundline/main` from the
  repo-managed sync workflow instead of staging a separate tap pull request.
- Align the release surface, generated distribution artifacts, and contract
  coverage to the same `0.45.0` delivery line.

## [0.44.0] - 2026-05-07

Delivered specs:

- `044` - CLI Init UX

Highlights:

- Make guided `boundline init` self-sufficient by surfacing supported assistant
  values, supported route slots, valid `SLOT=RUNTIME:MODEL` examples, and
  blank-input/default behavior directly in prompts and `init --help`.
- Turn init routing failures and overwrite previews into more actionable,
  example-backed recovery guidance while keeping workspace mutation bounded and
  explicit.
- Group `init` and `doctor` operator output into clearer summary, route,
  assistant-setup, check, and action sections, then align docs and assistant
  guidance to the same first-run story.

## [0.43.0] - 2026-05-07

Delivered specs:

- `043` - Stack-Neutral Workspace Entry

Highlights:

- Make generic workspace diagnostics and native direct-run bootstrap
  stack-neutral so empty and non-Rust repositories can enter the primary
  session-native path without adding a `Cargo.toml`.
- Let `boundline init --assistant claude|copilot|codex|gemini` seed
  deterministic planning, implementation, verification, and review routes from
  the maintained assistant default-model catalog while preserving explicit
  route overrides.
- Add bounded workspace hygiene defaults that merge universal, domain-family,
  and tool-specific ignore patterns only when selected domains or repository
  cues make them credible.

## [0.43.0] - 2026-05-06

Delivered specs:

- `042` - Native Canon CLI Surface

Highlights:

- Make Canon the default governed route for Canon-ready workspaces, add
  workspace-local Canon mode-selection preferences, and support explicit
  `--no-canon` opt-out.
- Add all canonical Canon modes to the CLI/chat surface, forward authored briefs
  into Canon `input_documents`, preserve governed packet context across stages,
  and surface incomplete Canon packets as clarification prompts.
- Extend `boundline init`, `config set-canon`, `doctor --install`, docs, and
  assistant command packs so CLI and chat workflows use the same Canon-default
  commands.

## [0.41.0] - 2026-05-04

Delivered specs:

- `041` - Checkpoint Rewind

Highlights:

- Create one implicit bounded checkpoint before mutating `run` and `step`, persist rollback manifests under `.boundline/checkpoints/`, and keep restore semantics explicit for pre-existing, newly created, deleted, and already-modified files.
- Add `boundline checkpoint list` and `boundline checkpoint restore <id>` with safe refusal by default, `--force` override, and grouped clustered restore through the primary workspace.
- Project `latest_checkpoint_id`, `latest_checkpoint_scope`, and `latest_checkpoint_restore_command` through `run`, `status`, `next`, and `inspect`, while refounding the Rust workspace around `boundline-core`, `boundline-adapters`, and `boundline-cli`.

## [0.40.0] - 2026-05-03

Delivered specs:

- `040` - Context Selection Hardening

Highlights:

- Replace keyword-first context admission in the goal planner with evidence-selected context inputs that can be anchored by authored brief file references, recent changed files after failed validation, bounded source-test cue pairing, and reusable Canon evidence.
- Keep `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, and `context_staleness_reason` aligned to the same authoritative context story while making weak or contradictory context an explicit planning stop.
- Tighten README, getting-started, architecture, assistant guidance, roadmap, and changelog so the first-run quick path is clearer and the Boundline-versus-Canon boundary stays explicit in the `0.40.0` release.

## [0.39.0] - 2026-05-03

Delivered specs:

- `039` - Distribution & Bundling

Highlights:

- Add repo-managed Homebrew and winget metadata plus a release workflow that assembles Boundline bundles with a compatible Canon companion for the documented install surface.
- Introduce `boundline doctor --install` so operators can verify the running Boundline version, the documented Canon support target, and whether the local pairing is ready, already satisfied, blocked, or repair-needed.
- Split public docs and assistant guidance into a brutal quick path plus a separate advanced architecture layer while keeping Boundline as the orchestration owner and Canon as the bounded governance companion.

## [0.38.0] - 2026-05-03

Delivered specs:

- `038` - Domain Agent Templates

Highlights:

- Add workspace-, cluster-, and global-scoped domain-template settings so `boundline init` can infer or accept active domain families, seed layered standards, and persist optional or required external context bindings in `.boundline/config.toml`.
- Make planning apply the right active domain family for the bounded target, surface the winning standards source and supporting-input status through context summary and provenance, and stop explicitly when enabled domain templates do not match or required domain inputs are missing or stale.
- Extend `config show`, assistant guidance, roadmap, configuration docs, README, and changelog for the `0.38.0` release while keeping Canon and other external systems as supporting bounded inputs rather than template owners.

## [0.37.0] - 2026-05-03

Delivered specs:

- `037` - Bounded Delegated Execution

Highlights:

- Add explicit runtime capability profiles plus slot-level effort policies, surface them through `config show`, `plan`, `run`, `status`, `next`, and `inspect`, and persist the same route-policy snapshot on native and compatibility traces.
- Replace opaque blocked native-route failures with explicit handoff, escalation, resolved, and stuck delegation packets that remain authoritative in session-owned continuity state and trace summaries.
- Resolve stale delegation explicitly during replanning when route declarations or bounded evidence change materially, then update README, getting-started, configuration, assistant guidance, roadmap, contributor docs, and changelog for the `0.37.0` release.

## [0.36.0] - 2026-05-03

Delivered specs:

- `036` - Canon-Grounded Memory

Highlights:

- Treat Canon capability snapshots, governed packet summaries, recommended actions, and compact governed memory as live planning and follow-through inputs instead of stage-end audit output alone.
- Persist compact Canon-grounded memory on the native goal-plan path and the compatibility/governed task-context path, then reuse it on `run`, `status`, `next`, and `inspect` with explicit credibility, provenance, staleness, and `governance_next_action` projection.
- Revalidate the Canon adapter against Canon `0.39.0`, then update README, getting-started, configuration, assistant guidance, roadmap, contributor docs, and changelog for the `0.36.0` release.

## [0.35.0] - 2026-05-02

Delivered specs:

- `035` - Dynamic Planning And Flow Inference

Highlights:

- Replace keyword-first native planning with an evidence-driven `infer -> propose -> confirm` loop that persists `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, and `verification_strategy` across `plan`, `run`, `status`, `next`, and `inspect`.
- Add explicit `boundline plan --confirm`, block native `run` while the current proposal is unconfirmed, and allow bounded replanning by superseding the active proposal revision when workspace evidence changes materially.
- Align direct native `run --goal` with the same proposal-plus-confirm planning contract, then update README, getting-started, configuration, assistant guidance, roadmap, contributor docs, and changelog for the `0.35.0` release.

## [0.34.0] - 2026-05-02

Delivered specs:

- `034` - Decision-Driven Orchestrator

Highlights:

- Make the native `observe -> decide -> act -> verify` loop authoritative by
  selecting one explicit bounded selector per iteration from decision state
  instead of replaying a mostly static task order.
- Persist and surface selector-driven follow-through on `run`, `status`,
  `next`, and `inspect`, including explicit `read`, `search`, `modify`,
  `test`, `ask`, and `replan` vocabulary, rationale, evidence basis, and
  bounded stop reasoning.
- Update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release.

## [0.33.0] - 2026-05-02

Delivered specs:

- `033` - Context Assembly Foundation

Highlights:

- Assemble one bounded context pack before native planning from workspace
  signals, authored briefs, negotiated delivery state, recent traces, and
  reusable Canon artifacts instead of relying on ambient workspace state.
- Surface `context_summary`, `context_credibility`, primary inputs,
  provenance, and any explicit staleness reason through `plan`, `run`,
  `status`, `next`, and `inspect`, including persisted native
  `GoalPlanCreated` trace payloads.
- Stop planning explicitly when bounded context is insufficient instead of
  silently producing a coarse plan, and update README, getting-started,
  configuration, contributor docs, assistant guidance, roadmap, and changelog
  for the release.

## [0.32.0] - 2026-05-02

Delivered specs:

- `032` - Product Unification And Surface Closure

Highlights:

- Promote workflow discovery and continuation to first-class assistant surfaces
  for Claude, Codex, and Copilot while aligning Gemini CLI guidance to the same
  workflow-first Boundline vocabulary.
- Keep workflow follow-through on the same primary Boundline product surface as
  direct native execution, with explicit `route_owner`,
  `route_config_projection`, and bounded next-command guidance.
- Keep explicit compatibility follow-up visibly subordinate instead of letting
  it read like a second primary product path.
- Update README, getting-started, configuration, contributor docs, assistant
  guidance, roadmap, and changelog for the release.

## [0.31.0] - 2026-05-02

Delivered specs:

- `031` - Canon Delivery Loop

Highlights:

- Keep Canon inside the same bounded delivery loop for governed `bug-fix` and
  `change` work so verify-stage `security-assessment` packet lineage, approval
  state, and governed follow-through stay on the same `run`, `status`, `next`,
  and `inspect` surface.
- Fail delivery completion explicitly when a bounded `bug-fix` or `change`
  path reaches the end of the plan without both a material workspace diff and
  passed validation evidence.
- Update README, getting-started, configuration, contributor docs, assistant
  guidance, roadmap, and changelog for the release.

## [0.30.0] - 2026-05-02

Delivered specs:

- `030` - Native Direct Run

Highlights:

- Make direct `boundline run --goal ...` bootstrap the
  native session route by default, including negotiated goal intake, executable
  planning, decision-loop execution, and persisted follow-up through `status`,
  `next`, and `inspect`.
- Preserve declarative execution profiles as an explicit subordinate route via
  `boundline run --compatibility --goal ...`, and block
  native direct run from silently overwriting meaningful active session state.
- Update README, configuration, getting-started, assistant guidance, roadmap,
  contributor docs, and changelog for the release.

## [0.29.0] - 2026-05-02

Delivered specs:

- `029` - Next Command Continuity

Highlights:

- Keep surfaced `next_command` aligned with authoritative follow-through,
  workflow-owned resume commands, and explicit stop conditions on the existing
  CLI surfaces.
- Preserve prerequisite and compatibility authority boundaries instead of
  implying resumable native execution when the next step is `status`,
  `inspect`, or `workflow resume`.
- Update README, configuration, getting-started, assistant guidance, roadmap,
  contributor docs, and changelog for the release.

## [0.28.0] - 2026-05-01

Delivered specs:

- `028` - Decision Continuity And Guided Follow-Through

Highlights:

- Make `status`, `next`, and `inspect` surface one explicit guided follow-through
  story via `follow_through_guidance`, `follow_through_evidence_source`,
  `follow_through_next_action`, and `follow_through_stop_reason` when the bounded
  task has enough persisted session or trace evidence to explain what should happen next.
- Reuse authoritative trace evidence on explicit compatibility follow-up without
  blurring native versus compatibility continuity ownership.
- Update README, configuration, getting-started, assistant guidance, roadmap,
  contributor docs, and changelog for the release.

## [0.27.0] - 2026-05-01

Delivered specs:

- `027` - Inspectable Routing And Assistant Decoupling

Highlights:

- Make effective slot routing and assistant binding explicit on `config show`,
  `run`, `status`, `next`, and `inspect`, including persisted routing
  snapshots on native and explicit compatibility traces.
- Preserve route snapshots when configuration changes after execution so
  inspection keeps the historical backend story instead of replaying the
  current workspace config.
- Fail native execution explicitly when the active implementation or
  verification route requires an assistant runtime outside the declared
  `assistant_runtimes` capability list.
- Update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release.

## [0.26.0] - 2026-05-01

Delivered specs:

- `026` - Goal Negotiation And Constraint Modeling

Highlights:

- Derive one negotiated delivery packet during `goal` from direct goals,
  authored briefs, and governance context before planning begins.
- Gate `plan` on a credible negotiation result and keep
  `negotiation_goal_summary`, `negotiation_resolution`, and
  `negotiation_acceptance_boundary` visible in `GoalPlan`, `run`, `status`,
  `next`, and `inspect` on both native and explicit compatibility routes.
- Update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release.

## [0.25.0] - 2026-05-01

Delivered specs:

- `025` - Multi-Workspace Delivery

Highlights:

- Extend the session-native commands with `--cluster <primary-workspace>` so
  one authoritative session can plan and deliver a bounded change across
  registered member repositories without splitting orchestration ownership.
- Persist clustered delivery participation plus member-local traces while
  keeping the active clustered session authoritative in the primary workspace.
- Surface clustered authority, execution condition, participating workspaces,
  and blocking-member guidance through `run`, `status`, `next`, `inspect`,
  assistant guidance, roadmap, contributor docs, and release notes.

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

- Make bounded `review` and `govern` phases executable from `boundline workflow`
  so named workflows can complete or stop in explicit paused, blocked, failed,
  or completed states on the same session-owned route.
- Add `boundline workflow list` plus optional workflow-registry discovery metadata
  (`summary`, `recommended_when`) so operators and assistants can choose the
  correct named workflow without reading raw registry files.
- Keep direct session-native commands and explicit compatibility routing
  available when no named workflow is invoked, even when a workspace defines
  `.boundline/workflows.toml`.
- Update README, getting-started, configuration, assistant guidance, roadmap,
  and contributor docs for the completed workflow follow-through slice.

## [0.18.0] - 2026-04-30

Delivered specs:

- `018` - Workflow Layer

Highlights:

- Add workspace-local `.boundline/workflows.toml` as a bounded named-workflow
  registry compiled onto Boundline's existing session-native phases.
- Add `boundline workflow run`, `status`, `resume`, and `inspect` so named
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

- Refound Boundline around `goal -> plan -> run -> status -> inspect`
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

- Added `boundline init` to scaffold bounded workspace defaults under `.boundline/`
  without hand-authoring setup files.
- Added `boundline config show|set|unset` for runtime/model routing at global and
  workspace scope.
- Added deterministic routing precedence (`CLI > workspace > global > built-in`)
  with effective-source visibility.
- Added initial runtime setup surface for Claude, Codex, Copilot, and Gemini
  CLI (CLI-only for Gemini in this slice).

## [0.10.0]

Delivered specs:

- `010-human-brief-ingestion`

Highlights:

- `boundline goal` and `boundline run` accept one or more `--brief <path>.md`
  arguments alongside (or instead of) `--goal`. Brief contents are normalized
  into a single goal text projected through the existing goal-intake pipeline so
  developers no longer need to author free-text prose only on the command line.
- New `boundline::domain::brief` module (`AuthoredBriefBundle`,
  `InputSourceReference`, `BriefIngestionError`, `normalize_inputs`) enforces
  workspace-bounded `.md`/`.markdown` sources, an upper bound of 10 brief
  files per invocation, and a 256 KiB per-source size cap.
- Multi-source resolution deduplicates explicit and referenced Markdown input
  into one persisted authored brief bundle with stable provenance across
  `goal`, `run`, `status`, and `inspect`.
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

- Add workspace execution manifests under `<workspace>/.boundline/execution.json`
  with fallback to the legacy `<workspace>/.boundline/fixture.json` shape.
- Let Boundline apply bounded workspace changes, run validation commands, and take
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

- Persist active workspace session state under `<workspace>/.boundline/session.json`.
- Unify `start`, `goal`, `plan`, `step`, `run`, `status`, `next`, and
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
- Persist execution traces under `<workspace>/.boundline/traces/` as the foundation
  for later CLI, session, and delivery surfaces.
