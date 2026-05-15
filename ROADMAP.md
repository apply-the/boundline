# Boundline Roadmap

Canon is downstream from Boundline in this roadmap: Boundline thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Boundline can reuse for reasoning.

## Objective

Evolve Boundline into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.55.0

Boundline now has its core session-native orchestration baseline, bounded workflow
follow-through, deeper governed-stage plus adaptive slices, explicit
continuity between session-native and compatibility follow-up, stronger
route-summary plus config projection, bounded multi-workspace clustered
delivery, negotiated delivery modeling, inspectable routing plus assistant
decoupling, guided decision follow-through, evidence-aligned next-command
selection, credible governed delivery completion, final product-surface
closure, explicit bounded context assembly, decision-driven bounded action
selection, evidence-driven dynamic planning, Canon-grounded structured
memory, bounded delegated execution, and a release-aligned distribution
surface in place, with Canon-default governed setup, runtime selection, and
assistant-surface alignment now carried through the same primary workflow. The
operator entry path is now stack-neutral for empty, non-Rust, and mixed
repositories, with a clearer first-run CLI UX. That baseline now extends into
Canon-promoted project-memory reuse. The `0.54.0` slice added guidance and
guardian capabilities across planning, implementation, verification, and
review. The current `0.55.0` work packages that guidance surface as a
catalog-aware assistant pack with explicit manifests, canonical taxonomy,
inspectable pack loading, and validation findings instead of relying only on
flat pack manifests.

### Delivered in 0.55.0

- Boundline now discovers directory-based guidance catalog packs under
  `assistant/packs/` alongside legacy flat `.toml` manifests, and it keeps
  loaded packs, skipped packs, and validation findings explicit in the runtime
  projection.
- The bundled `assistant/packs/guidance-catalog/` surface carries normalized
  manifest, guidance-index, and guardian-index contracts plus canonical pillar,
  strength, and disposition vocabulary derived from the S2.1 phase-7 addendum.
- Session-native `plan`, `status`, and `inspect` now surface catalog pack
  discovery and validation as operator-visible state rather than hidden loader
  behavior.

### Delivered in 0.54.0

- Boundline now resolves clean-code, architecture, security, language,
  framework, and testing guidance with explicit source precedence across
  built-in packs, shared packs, workspace overrides, and optional Canon
  governed inputs.
- Assistant guidance packs are no longer Rust-only: the bundled surface now
  covers shared engineering guidance plus JavaScript or TypeScript, Python,
  JVM, .NET, Go, PHP or Ruby, mobile, systems, and shell or automation
  delivery clusters.
- Session-native `run` and `status` now persist and project guardian timelines,
  findings, degradations, and blocking summaries instead of keeping them as
  transient execution detail.

### Delivered in 0.53.0

- Boundline now computes one deterministic built-in expert-pack selection
  outcome before planning continues and persists the selected packs,
  rejected candidates, and supporting signals in the goal plan.
- Session-native `status`, `next`, and `inspect` surfaces now expose
  expert-pack selection summaries, suggested runtime roles, and the boundary
  between local evidence and optional Canon expertise input.
- Canon remains supporting evidence only for this slice, while the active
  compatibility and distribution surfaces align the companion target to Canon
  `0.52.0` and keep the Boundline release metadata on `0.53.0`.

### Delivered in 0.52.0

- Boundline now formalizes the local runtime substrate as the planning
  precondition for deterministic context packs, explicit credibility states,
  and inspectable provenance.
- Session-native `status`, `next`, `inspect`, and run-trace surfaces now make
  local-versus-Canon context selection inspectable, including source-labelled
  provenance for bounded context inputs.
- Canon remains optional enrichment, while the active compatibility and
  distribution surfaces align the companion target to Canon `0.51.0` and keep
  the Boundline release metadata on `0.52.0`.
- The bundled model catalog was revalidated against current provider docs and
  stayed release-aligned without adding new route families for this slice.

### Delivered in 0.51.0

- Boundline now reads Canon-promoted `docs/project/*.md` surfaces with adjacent
  packet metadata sidecars and derives the supporting `docs/evidence/...`
  locations needed for later reuse.
- Project-memory compatibility is pinned to the Canon major-1 (`v1`) contract line,
  while pending or incomplete promotions remain visible without becoming
  authoritative planning input.
- Compacted Canon memory and governed-stage input assembly now carry project-memory
  provenance forward, and the documented Canon companion target is updated to
  `0.50.0`.

### Delivered in 0.50.0

- Boundline now has a project-scale delivery UX slice that treats broad
  initiatives as bounded stages and bounded work units rather than one
  unchecked autonomous run.
- Global assistant bootstrap, repo-local assistant packages, and the CLI runtime
  are now separated in the product model so `/boundline:init`, `/boundline:doctor`,
  `/boundline:continue`, and `/boundline:status` can be available before
  workspace initialization where hosts support it.
- `/boundline:govern` is the primary governed stage surface for the full Canon
  `0.45.0` mode set, with Canon capability checks and explicit unsupported-mode
  failure instead of per-mode Boundline aliases as the primary UX.
- Review voting is scoped to risky quality boundaries and projected through the
  same status, next, inspect, trace, and checkpoint surfaces.
- The Delivery Pilot Model explains that large work is supported by
  decomposition, not by unbounded autonomy.

### Delivered in 0.49.1

- Guided `boundline init` now scaffolds `.boundline/` reliably before writing
  release-critical workspace files, including relative `--workspace .` flows.
- Guided route review now starts from bundled catalog defaults when no
  assistant surfaces are selected, instead of presenting all slots unset.
- The bundled Copilot runtime catalog now exposes Anthropic and Gemini models
  that are available through Copilot, so cross-provider model selection is
  visible from the Copilot runtime.

### Delivered in 0.49.0

- Claude Code, Codex, and Cursor now have repository-local assistant plugin
  package folders, with Copilot represented as an honest prompt-pack boundary
  rather than an invented universal plugin format.
- Shared plugin metadata and command definitions expose Boundline as a local
  delivery orchestrator for bounded engineering work while keeping
  `.boundline/session.json` and CLI output authoritative.
- Namespaced `/boundline:*` command bindings cover start, capture, plan, run,
  status, inspect, recover, and conditional Canon governance without making
  Canon visible in normal delivery flow.
- Package validation now checks manifest JSON, required fields, referenced
  paths, command coverage, version alignment, and prohibited positioning.

### Delivered in 0.48.0

- shell-enabled lifecycle commands plus `run`, `status`, `next`, and `inspect`
  can now emit one stable host envelope with `command_name`, `exit_status`,
  `rendered_output`, `session_status`, `trace_summary`, and `trace_location`
- host-assisted `inspect` can now reuse the current workspace when no explicit
  trace selector is supplied, keeping the session-native trace follow-up path
  usable inside chat and other host integrations
- assistant command packs now prefer the structured shell path so the same
  persisted session and trace state can flow through Claude, Codex, Copilot,
  and Gemini without reparsing plain text first

- the bundled assistant catalog now exposes the current mainstream route-
  capable models for the supported runtimes instead of forcing custom ids for
  common current choices
- capture and compatibility input normalization now treat one Markdown path or
  an ordered Markdown-path array supplied as goal text as file-backed authored
  input rather than persisting the raw shorthand as direct text
- review vote resolution now rejects councils that collapse onto the same
  effective runtime/model route and persists the effective route on recorded
  review participants

- guided `boundline init` now explains supported assistants, supported route
  slots, blank/default behavior, and a valid `SLOT=RUNTIME:MODEL` example
  directly in the terminal and in `init --help`
- init success and preview output now group route setup, assistant scaffolding,
  and next-step guidance around `boundline config show --workspace ...` and
  `boundline doctor --workspace ...`
- `boundline doctor` now groups summary, checks, and actions so first-run
  readiness output is easier to scan without losing plain-text semantics

- the prior stack-neutral workspace-entry release is preserved underneath that
  UX layer:

- generic workspace diagnostics and direct native `run --goal` no longer
  require `Cargo.toml` before Boundline can capture and plan a bounded task
- `boundline init --assistant claude|copilot|codex|gemini` now seeds
  deterministic default model routes for planning, implementation,
  verification, and review unless a slot is explicitly overridden
- selected domain families and credible repository cues now drive merge-only
  `.gitignore`, `.dockerignore`, and related tool ignore defaults while
  preserving local rules

- Canon-ready workspaces now default `boundline run --goal` to the governed
  Canon runtime, with explicit `--mode` selection and `--no-canon` opt-out
- `boundline init`, `config show`, and `config set-canon` now keep Canon
  mode-selection preferences and workspace-local governance defaults visible on
  the same primary operator surface
- `boundline doctor --install` now verifies the actual Canon governance
  surface, including operations, supported modes, and authoritative binary path
- Copilot, Codex, Claude, and Gemini command packs now expose the same
  Canon-default workflow and mode aliases as the CLI

- Boundline now ships repo-managed Homebrew and winget metadata plus a release
  workflow that assembles Boundline bundles with a compatible Canon companion
- `boundline doctor --install` now verifies the installed Boundline version, the
  documented Canon target, and whether the local pairing is ready, already
  satisfied, blocked, or repair-needed
- README and getting-started now lead with a brutal quick path, while the
  deeper product model moves into a separate advanced architecture layer
- the Boundline-versus-Canon boundary is now explicit across install docs,
  assistant guidance, release metadata, roadmap, and changelog
- mutating `run` and `step` now create local reversible checkpoints under
  `.boundline/checkpoints/` before bounded workspace changes are applied
- `boundline checkpoint list` and `boundline checkpoint restore <id>` now make
  rollback explicit on both single-workspace and clustered session-native paths
- the repository now builds as a Rust workspace split across
  `boundline-core`, `boundline-adapters`, and `boundline-cli` while keeping the
  repo-root cargo entrypoints stable

- session-native orchestration remains the primary operator path
- `capture` now derives one negotiated delivery packet from direct goals, authored briefs, and governance context before planning begins
- `plan` now stops early when `negotiation_resolution` is not yet credible instead of silently inventing a bounded change
- `plan` now also assembles one explicit bounded context pack from relevant workspace files, authored input, recent traces, negotiated delivery state, and reusable Canon artifacts before proposing a goal plan
- planning now stops explicitly when that context pack is insufficient or stale instead of relying on ambient workspace state
- planning now admits primary workspace context from explicit evidence such as authored brief file refs, recent changed files after failed validation, and bounded source/test pairing cues instead of trusting keyword-ranked repository paths alone
- weak, stale, or contradictory context now stops planning explicitly rather than silently degrading to an ambient repository guess
- native planning now infers flow, targets, and verification strategy from scored workspace and session evidence instead of relying primarily on goal keywords
- default `plan` now persists one proposed goal plan with explicit `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, and `verification_strategy`, and `plan --confirm` now makes that proposal executable on the native route
- repeated `plan` can now supersede the active proposal revision when evidence changes flow, task targets, or verification strategy materially, while `run`, `status`, `next`, and `inspect` keep the revision lineage and blocking reason explicit
- `run`, `status`, `next`, and `inspect` now project `context_summary`, `context_credibility`, primary inputs, provenance, and any staleness reason from the active goal plan or authoritative native trace
- `boundline init` can now infer or accept active domain families and seed workspace-local domain template settings, layered standards, and optional external context bindings in `.boundline/config.toml`
- planning now stops explicitly when enabled domain templates do not match the bounded target or when a required external context binding is missing or stale
- `config show`, `plan`, `run`, `status`, `next`, and `inspect` now surface the selected domain family, winning standards source, and supporting-input status through the same bounded context story
- Canon capability snapshots plus compact Canon-grounded memory now feed context assembly, target selection, verification strategy inference, and explicit bounded stop conditions when governed evidence is stale or contradicted
- the native runtime now selects explicit bounded next actions such as `read`, `search`, `modify`, `test`, `ask`, and `replan` from persisted decision state and evidence instead of treating decisions as trace-only audit records
- `status`, `next`, and `inspect` now surface selector-driven guidance, rationale, evidence basis, verification intent, and explicit no-credible-next-step wording from the same persisted decision story
- `run`, `status`, `next`, and `inspect` now reuse compact Canon-grounded memory from the active goal plan or authoritative task context so governed provenance, stale-memory reasoning, and `governance_next_action` stay visible even when native plan projection is absent
- `run`, `status`, `next`, and `inspect` now project `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` across native goal-plan traces and explicit compatibility traces
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` still project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- Claude, Codex, and Copilot now ship first-class workflow assistant surfaces, while Gemini CLI guidance uses the same workflow-first vocabulary
- workflows and direct native runs now read as the two primary Boundline product entry styles, while explicit compatibility follow-up remains visibly subordinate
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- direct `run --goal` now bootstraps the same evidence-driven proposal-plus-confirm planning path before execution instead of shortcutting through keyword-only flow selection
- Canon remains a bounded stage-boundary governance runtime with governed `bug-fix:investigate` plus later verify-stage `security-assessment` reuse on the same operator surface
- bounded `bug-fix` and `change` delivery no longer succeed merely because the plan ran out of steps; they now require a material diff and passed validation evidence or stop explicitly as failed
- successful governed delivery now keeps `latest_changed_files`, `latest_validation_status`, and governed packet lineage visible on the same follow-through surfaces
- adaptive compatibility execution still stays inside manifest-declared `read_targets` while surfacing candidate credibility, rejection, and exhaustion reasons explicitly
- `status` and `next` still surface `continuity_authority`, compatibility follow-up mode, and inspect-only guidance when the latest authoritative follow-up state comes from an explicit compatibility trace instead of an active session
- `run`, `status`, `next`, `inspect`, and compatibility follow-up still surface explicit `route_owner` plus material `route_config_projection` cues when workflow metadata, governance intent, or workspace-local routing defaults explain the current follow-up story
- `config show`, `run`, `status`, `next`, and `inspect` now surface effective slot routing, assistant bindings, and persisted route snapshots instead of forcing operators to reconstruct backend ownership from current config files
- `config show`, `plan`, `run`, `status`, `next`, and `inspect` now also surface effective runtime capability profiles plus slot effort policies so route-policy decisions stay attributable to the same source as effective routing
- native execution now turns blocked implementation or verification routes into explicit handoff or escalation packets instead of returning opaque assistant-binding failures
- repeated blocked native continuity now escalates to explicit stuck state, while replanning can resolve stale delegation packets when the route policy or bounded evidence changes materially
- `status`, `next`, and `inspect` now surface guided next-action and stop-condition output derived from persisted session or authoritative trace evidence instead of generic lifecycle wording alone
- explicit compatibility follow-up now keeps continuity authority explicit while still projecting one evidence-backed next bounded action
- surfaced `next_command` now stays aligned with the same authoritative follow-through, workflow resume, or explicit stop condition instead of drifting back to a generic fallback
- session-native commands still accept `--cluster <primary-workspace>` so one authoritative primary-owned session can plan and deliver a bounded change across registered member repositories
- clustered `run`, `status`, `next`, and `inspect` still surface authoritative workspace, clustered execution condition, participating workspaces, and any blocking member without implying distributed orchestration ownership

## Post-0.46.0 Roadmap

`0.46.0` carries the chat-first structured host-runtime release forward while
updating the supported Canon companion target and keeping the distribution
automation aligned to the same operator story. Boundline now has guided Canon
setup, Canon-ready default governed runs, real surface verification, explicit
governed follow-through states, assistant command packs aligned to the same
primary workflow, a stack-neutral init/readiness surface for the supported
domain catalog, a clearer first operator pass through `init` and `doctor`,
direct Homebrew tap propagation from the managed release surface, and a stable
JSON host envelope over the primary session-native lifecycle.

The governing rule remains simple: Boundline is still the product and execution
owner. Canon stays a bounded, useful governed runtime inside that same delivery
path rather than drifting back into a parallel tool story.

### Delivered in 0.48.0

- refreshed the bundled assistant model catalog with current mainstream route-
  capable Copilot, Claude, Codex, and Gemini entries and aligned built-in
  verification/review defaults to that catalog
- taught authored-input normalization to treat one Markdown path or ordered
  Markdown-path arrays entered through prompt-style goal text as file-backed
  input with preserved provenance and order
- made review voting require distinct effective reviewer routes before a
  multi-review council can be counted as independent

### Delivered in 0.47.0

- replaced the fragile guided `boundline init` freeform prompts with a bounded
  terminal wizard that shows visible defaults, route review, slot-by-slot
  editing, and an explicit final summary before writes
- added a bundled runtime/model catalog with explicit source metadata,
  custom-model warnings, and grouped assistant-pack scaffolding status for the
  selected assistant surfaces
- added top-level `--version` support plus non-interactive init parity and
  time-consuming bootstrap progress feedback

### Delivered in 0.46.0

- added a stable `--json` host envelope across the session-native lifecycle,
  `run`, and `inspect` while preserving the existing rendered plain-text output
- aligned `inspect` and the assistant command packs to the same structured
  shell-first host flow, including current-workspace fallback for session-owned
  trace follow-up
- adopted Canon `0.43.0` as the documented supported companion version across
  doctor/install output, distribution metadata, and governed runtime evidence

### Delivered in 0.45.0

- adopted Canon `0.41.0` as the documented supported companion version across
  doctor/install output, distribution metadata, and governed runtime evidence
- switched the `homebrew-boundline` sync flow from PR creation to direct
  authenticated pushes to `main` using the repo-managed tap token
- aligned release metadata, generated package artifacts, and contract tests to
  the `0.45.0` distribution surface

### Delivered in 0.44.0

- made guided `boundline init` and `init --help` self-sufficient for assistant
  selection, route syntax, defaults, and follow-up inspection
- made init validation and preview output more actionable with example-backed
  route recovery and explicit overwrite guidance
- grouped init and doctor output into clearer summary/check/action sections

### Delivered in 0.43.0

- removed Rust-specific repository-root assumptions from generic workspace
  readiness checks and native direct-run bootstrap
- let empty and non-Rust repositories enter through the same primary native
  path, with planning credibility handled by captured goals, briefs, and
  bounded repository evidence instead of a stack-specific manifest
- seeded assistant-specific default routes during init for Claude, Copilot,
  Codex, and Gemini while preserving explicit route overrides
- added bounded, merge-only hygiene defaults for universal, domain-family, and
  tool-specific ignore files when selected domains or repository cues justify
  them

### Program Rules

- checkpoint storage remains local and bounded to declared workspace roots; it
  is not a general snapshot system
- Git remains useful but optional; checkpoint and rewind must work in dirty
  repositories and in workspaces that are not under version control
- generic workspace readiness, session bootstrap, and planning entry must stay
  stack-neutral until bounded evidence or explicit operator input selects a
  supported domain family; Rust-specific assumptions only belong on Rust-owned
  paths
- existing primary commands and assistant packs must keep working from the repo
  root even after the crate layout changes
- Canon remains downstream and optional except where governed artifacts are
  already explicitly modeled in the current delivery flow

### Delivered in 0.43.0

- default `run --goal` to Canon on Canon-ready workspaces while keeping
  `--no-canon` and explicit `--mode <mode>` operator-controlled
- assemble authored briefs, clarification answers, and bounded reused packet
  context into Canon-ready request payloads across governed stages
- extend `boundline init`, `config show`, `config set-canon`, and
  `doctor --install` so Canon mode-selection and surface verification stay on
  the same workspace-local operator path
- align Copilot, Codex, Claude, and Gemini command packs to the same
  Canon-default CLI workflow without manual manifest editing guidance

### Delivered in 0.41.0

- split the repository into `boundline-core`, `boundline-adapters`, and
  `boundline-cli` workspace members while keeping repo-root cargo commands and
  compatibility exports stable
- create one implicit checkpoint before mutating `run` and `step`, persist
  manifests under `.boundline/checkpoints/`, and keep captured-file semantics
  explicit for pre-existing, newly created, deleted, and already-modified paths
- ship `boundline checkpoint list` and `boundline checkpoint restore <id>` with
  safe refusal by default, `--force` override, and grouped clustered restore
  through the primary workspace
- project `latest_checkpoint_id`, `latest_checkpoint_scope`, and
  `latest_checkpoint_restore_command` through `run`, `status`, `next`, and
  `inspect` so rollback stays visible on blocked or failed mutating work
- update README, getting-started, architecture, assistant guidance,
  distribution metadata, roadmap, and changelog together for the `0.41.0`
  release

### Delivered in 0.40.0

- replace keyword-first context selection with evidence-selected primary inputs
  so planning can use authored brief file refs, recent changed files after
  failed validation, and bounded source/test pairing cues instead of relying on
  path scoring alone
- keep `context_summary`, `context_credibility`, `context_primary_inputs`,
  `context_provenance`, and `context_staleness_reason` aligned to the same
  authoritative planning story while turning weak or contradictory context into
  an explicit stop condition
- tighten README, getting-started, architecture, assistant guidance, roadmap,
  and changelog so the first-run quick path is clearer and the Boundline-versus-
  Canon boundary stays explicit for the `0.40.0` release

### Ongoing Compatibility Watch

Canon will continue to release versions after `0.38.0`, so Boundline keeps one
explicit maintenance track for compatibility drift on the machine-facing
governance adapter rather than reopening broad product-scope work:

- revalidate the documented Canon compatibility target against the latest
  released `canon governance start|refresh|capabilities --json` `v1` surface;
  the current documented target is Canon `0.43.0`
- preserve additive-field tolerance and capability-aware checks so intermediate
  Canon releases do not force unnecessary Boundline churn when the `v1` adapter
  contract remains stable
- schedule a new roadmap/spec slice only when Canon changes the required wire
  contract, introduces a new adapter schema version, or adds governed behavior
  that Boundline needs for its explicitly modeled bounded stages

Compatibility maintenance for newer Canon releases stays inside the watch above
unless it becomes contract-breaking. It does not consume a roadmap slot by
itself.

### Delivered in 0.39.0

- add repo-managed Homebrew and winget metadata plus a release-windows-distribution
  workflow that produces the Windows Boundline bundle carrying the documented Canon companion
- introduce `boundline doctor --install` so operators can verify the installed
  Boundline version, Canon support target, and repair state before entering a
  workspace
- split the public docs into a brutal quick path and a separate advanced
  architecture layer while keeping Boundline as the orchestration owner and Canon
  as the bounded governance companion
- update assistant guidance, changelog, roadmap, and release metadata together
  so the distribution story ships as one coherent `0.39.0` surface

### Delivered in 0.38.0

- infer or accept active domain families during `boundline init`, then persist
  workspace-local domain-template settings, scoped standards, and optional or
  required external context bindings in `.boundline/config.toml`
- layer built-in domain guidance with global, cluster, and workspace standards
  using explicit precedence and source attribution on `config show --scope effective`
- block planning explicitly when enabled domain templates do not match the
  bounded target or when a required external context binding is unavailable or
  stale relative to the selected target
- project selected domain family, winning standards source, and supporting-input
  status through `plan`, `run`, `status`, `next`, `inspect`, assistant
  guidance, roadmap, configuration docs, README, and changelog

### Delivered in 0.37.0

- add explicit runtime capability profiles plus slot effort policies on top of
  effective routing, and keep that route-policy story attributable across
  `config show`, `plan`, `run`, `status`, `next`, and `inspect`
- replace opaque blocked native-route failures with explicit handoff,
  escalation, resolved, and stuck delegation packets persisted in the active
  goal plan and authoritative trace summaries
- let repeated `plan` resolve stale delegation explicitly when route
  declarations or bounded evidence change materially, instead of leaving the
  old packet as the active source of truth
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.36.0

- treat Canon packets, governed artifacts, capability signals, and recommended
  actions as live input to context assembly and decision-making instead of only
  end-of-stage output
- promote packet reuse, artifact invariants, and governed constraints into the
  same bounded reasoning path used for planning, decision selection, and
  verification strategy inference
- persist compact Canon-grounded memory with explicit credibility, provenance,
  staleness, and next-action cues so long-running sessions can carry forward
  the important governed evidence without replaying the whole workspace
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.35.0

- replace keyword-first flow inference and stage-static planning with one
  evidence-driven `infer -> propose -> confirm` planning loop derived from
  context packs, selected targets, traces, workflow guardrails, and observed
  workspace evidence
- require explicit `plan --confirm` before native execution, keep proposed or
  confirmed plan state inspectable on `plan`, `run`, `status`, `next`, and
  `inspect`, and let repeated `plan` supersede the active proposal revision
  when evidence changes materially
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.34.0

- make the native `observe -> decide -> act -> verify` loop authoritative by
  selecting explicit bounded next actions from persisted decision state and
  evidence instead of replaying a mostly static task order
- evolve decisions into explicit selector-driven actions such as `read`,
  `search`, `modify`, `test`, `ask`, and `replan`, and keep recovery plus stop
  conditions explainable from that same state
- surface selector-driven guidance, rationale, evidence basis, and explicit
  stop reasoning through `run`, `status`, `next`, and `inspect`
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.33.0

- assemble one bounded context pack before native planning from workspace
  signals, authored briefs, negotiated delivery state, recent traces, and
  reusable Canon artifacts instead of relying on ambient workspace state
- project `context_summary`, `context_credibility`, primary inputs,
  provenance, and any staleness reason through `plan`, `run`, `status`,
  `next`, and `inspect` on the primary Boundline path
- stop planning explicitly when a credible bounded context cannot be built,
  keeping the recovery action and blocked state visible on the same session and
  trace surfaces
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.32.0

- unify assistant surfaces, routing slots, workflow entry points, and model
  bindings so Copilot, Codex, Claude, and Gemini guidance map onto the same
  Boundline-owned product story instead of provider-specific command drift
- keep workflow discovery and follow-through on the same primary Boundline path as
  direct native execution while preserving `route_owner`, `route_config_projection`,
  and bounded next-command cues
- keep explicit compatibility usage visibly subordinate instead of letting it
  read like a second primary product path
- close README, getting-started, configuration, assistant guidance, roadmap,
  contributor guidance, and changelog around the final product identity: users
  use Boundline; Canon is visible but secondary

### Delivered in 0.31.0

- keep Canon inside the same bounded delivery loop for governed `bug-fix` and
  `change` work instead of treating governance as a separate sidecar product
- make delivery completion credible by requiring a material diff and passed
  validation evidence before bounded `bug-fix` or `change` work can end in
  success
- keep governed and non-governed follow-through on the same `run`, `status`,
  `next`, and `inspect` surfaces, including governed packet lineage plus
  `latest_changed_files` and `latest_validation_status`
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.29.0

- keep surfaced `next_command` aligned with authoritative follow-through, workflow-owned resume commands, and explicit stop conditions on the existing CLI surfaces
- preserve prerequisite and compatibility authority boundaries instead of implying resumable native execution when the next step is `status`, `inspect`, or `workflow resume`
- update README, configuration, getting-started, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.30.0

- make direct `run --goal` bootstrap the native session path by default instead of routing implicitly through the compatibility runtime
- preserve declarative execution profiles as an explicit subordinate route via `run --compatibility --goal ...`
- block direct native run from silently overwriting meaningful active session state
- update README, configuration, getting-started, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.28.0

- make `status`, `next`, and `inspect` project `follow_through_guidance`, `follow_through_evidence_source`, `follow_through_next_action`, and `follow_through_stop_reason` when persisted session or trace evidence supports one bounded follow-up story
- keep explicit compatibility continuity authoritative while still surfacing one evidence-backed next bounded action
- update README, configuration, getting-started, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.27.0

- make effective routing plus assistant bindings explicit on `config show`, `run`, `status`, `next`, and `inspect`
- persist routing snapshots on native and explicit compatibility traces so inspection keeps historical backend ownership even after config changes
- stop native execution explicitly when the active implementation or verification route requires an assistant runtime outside declared `assistant_runtimes`
- update README, getting-started, configuration, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.26.0

- derive a negotiated delivery packet during capture from direct goals, authored briefs, and governance context before planning begins
- gate planning on a credible negotiation result and keep acceptance-boundary wording visible in `GoalPlan` state
- project negotiated delivery summary, resolution, and acceptance boundary through `run`, `status`, `next`, and `inspect` on both native and explicit compatibility routes
- update README, getting-started, configuration, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.25.0

- extend session-native commands with `--cluster <primary-workspace>` so one authoritative session can plan and deliver a bounded change across registered member repositories
- persist clustered delivery participation and member-local traces while keeping the active session authoritative in the primary workspace
- project clustered authority, execution condition, participating workspaces, and blocking member cues through `run`, `status`, `next`, `inspect`, assistant guidance, roadmap, contributor docs, and changelog

### Delivered in 0.24.0

- converge `run`, `status`, `next`, `inspect`, and compatibility follow-up around explicit `route_owner` plus one aligned route-summary vocabulary
- project material route/config cues such as workspace-local routing defaults, workflow or flow context, and requested governance intent through the same operator-facing surfaces
- update README, getting-started, configuration, adaptive-execution, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.23.0

- expand adaptive change kinds beyond arithmetic, comparison, and boolean flips while keeping generation deterministic and bounded
- surface `candidate_family`, selection credibility, rejected candidates, and explicit exhaustion guidance across `run`, `status`, `next`, and `inspect`
- stop bounded adaptive recovery explicitly when validation evidence is absent or insufficient for another materially different candidate
- update README, getting-started, configuration, adaptive-execution, assistant guidance, roadmap, contributor docs, and changelog for the release

### Delivered in 0.22.0

- define `continuity_authority` and inspect-only compatibility follow-up when the latest authoritative workspace state comes from an explicit compatibility trace
- make `status` and `next` usable after explicit compatibility `run` even when no active session exists, preferring workspace `inspect` instead of a false resumability story
- reuse the same route and `execution_condition` vocabulary across native session and compatibility follow-up surfaces while keeping routing ownership explicit
- update README, getting-started, configuration, adaptive-execution, assistant guidance, roadmap, and contributor docs for the release

### Delivered in 0.21.0

- use validation evidence to re-rank bounded adaptive repair candidates and shift the next selected file when a different manifest-declared target becomes more credible
- surface validation-guided adaptive selection headlines on the explicit compatibility route and in trace inspection output
- update adaptive-execution, README, getting-started, configuration, assistant, roadmap, contributor docs, and changelog for the release

### Delivered in 0.20.0

- govern `bug-fix:investigate` on the primary session-native route while preserving later governed verify reuse
- refresh approval state and blocked guidance through `status`, `next`, `inspect`, and the workflow-aware surfaces
- keep inspect summaries usable for paused or blocked governance traces instead of failing on non-terminal evidence

### Delivered in 0.19.0

- `boundline workflow list` exposes named workflows, phase chains, summary text, and invocation guidance from `.boundline/workflows.toml`
- `boundline workflow run <name>` and `resume` can now carry bounded `review` and `govern` phases to explicit paused, blocked, failed, or completed outcomes on the existing session-native route
- workflow progress persists in `.boundline/session.json` and reuses `.boundline/traces/` while direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- authored workflow registries now have clearer bounded guidance through optional `summary` and `recommended_when` metadata plus updated operator and assistant docs

Representative workflow registry shape:

```toml
[workflow.governed-delivery]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "review", "govern", "inspect"]
allow_review = true
allow_governance = true
summary = "bounded delivery path with review and governance before completion"
recommended_when = "the task needs explicit review and governance evidence"

[workflow.governed-delivery.when]
review = "review_triggered"
governance = "governance_required"

[workflow.governed-delivery.output]
next_command = true
routing_summary = true
execution_condition = true
```

### Future research and extension specs

The next forward-looking roadmap drafts now live in the repo-local `roadmap/`
folder. The S1 foundation is already being tracked in active
[`052-runtime-intelligence-substrate`](specs/052-runtime-intelligence-substrate/spec.md);
the copied drafts below extend that baseline rather than replacing the current
delivery line.

- [S2.1: Guidance And Guardian Capabilities](roadmap/S2-1%20-%20guidance-and-guardian-capabilities.md)
  extends the Expert Pack baseline from S2 into executable engineering
  principles (Guidance) and automated verification rules (Guardians) across
  architecture, design, and testing pillars.
- [S3: Authority-Zoned Delivery Roles, Personas, And Review Councils](roadmap/S3%20-%20authority-zoned-delivery-roles-and-review-councils-spec.md)
  extends the bounded review-council baseline in
  [`007-multi-agent-review`](specs/007-multi-agent-review/spec.md) and
  [`047-catalog-voting-inputs`](specs/047-catalog-voting-inputs/spec.md)
  into authority zones, personas, and admission-control posture.
- [S4: Control Graduation And Adaptive Governance](roadmap/S4%20-%20control-graduation-and-adaptive-governance-spec.md)
  extends adaptive execution and recovery depth from
  [`008-adaptive-execution-engine`](specs/008-adaptive-execution-engine/spec.md),
  [`021-adaptive-repair-depth`](specs/021-adaptive-repair-depth/spec.md), and
  [`023-broaden-bounded-adaptive-repair`](specs/023-broaden-bounded-adaptive-repair/spec.md)
  into calibrated governance progression, degradation, and control
  graduation.
- [S5: Advanced Context Intelligence](roadmap/S5%20-%20advanced-context-intelligence.md)
  and
  [S5.addendum: Advanced Context Intelligence Technology Evaluation](roadmap/S5.addendum%20-%20advanced-context-intelligence-technology-evaluation.md)
  are not already covered by an active delivery slice; they build on
  [`033-context-assembly-foundation`](specs/033-context-assembly-foundation/spec.md),
  [`040-context-selection-hardening`](specs/040-context-selection-hardening/spec.md),
  and [`052-runtime-intelligence-substrate`](specs/052-runtime-intelligence-substrate/spec.md)
  with optional semantic retrieval, graph projection, and local-first index
  technology evaluation.
- [S6: Advanced Multi-Agent Reasoning Profiles](roadmap/S6%20-%20advanced-multi-agent-reasoning-profiles-spec.md)
  is intentionally future-facing rather than already covered by the active
  roadmap; it builds on the review-council baseline in
  [`007-multi-agent-review`](specs/007-multi-agent-review/spec.md) after S3
  and S4 make councils operationally credible.
- [S7: Assistant Delight And Cognitive Affordance Layer](roadmap/S7%20-%20assistant-delight-and-cognitive-affordance-layer.md)
  complements assistant/package and operator-surface work in
  [`048-assistant-plugin-packages`](specs/048-assistant-plugin-packages/spec.md),
  [`049-project-scale-delivery-ux`](specs/049-project-scale-delivery-ux/spec.md),
  and [`051-delivery-control-consumer`](specs/051-delivery-control-consumer/spec.md),
  but it is not duplicate scope.

Coverage check: the delivered content already recorded in this roadmap is
covered by the active spec set, not by these future drafts. The copied S2.1-S7
documents mostly extend the current line; the strongest direct overlap is at
the dependency and foundation level rather than as duplicate roadmap scope.

### Sequencing rule

These three features are ordered and intentionally absorb all remaining major
scope:

1. Boundline must deliver real code changes before more platform abstraction work.
2. Canon must prove value inside that real delivery loop, not beside it.
3. Backend abstraction, assistant decoupling, and UX closure happen only after
  the real execution and governed-delivery story are stable.

## Architecture: User Through Execution

```text
User / Copilot / Claude
        ↓
      Boundline
  ┌───────────────┐
  │ Orchestrator  │
  │ Flows         │
  │ Agents        │
  │ Execution     │
  │ Review        │
  │ Adaptive      │
  └───────────────┘
        ↓
     Canon
   (governed stage docs + artifact persistence)
```

## In One Sentence

Boundline is a system that takes a problem and transforms it into working code, orchestrating bounded execution itself while using Canon to govern stage outputs and provide reusable documentation.
