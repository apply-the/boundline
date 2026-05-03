# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.35.0

Synod now has its core session-native orchestration baseline, bounded workflow
follow-through, deeper governed-stage plus adaptive slices, explicit
continuity between session-native and compatibility follow-up, stronger
route-summary plus config projection, bounded multi-workspace clustered
delivery, negotiated delivery modeling, inspectable routing plus assistant
decoupling, guided decision follow-through, evidence-aligned next-command
selection, credible governed delivery completion, final product-surface
closure, explicit bounded context assembly, decision-driven bounded action
selection, and evidence-driven dynamic planning in place:

- session-native orchestration remains the primary operator path
- `capture` now derives one negotiated delivery packet from direct goals, authored briefs, and governance context before planning begins
- `plan` now stops early when `negotiation_resolution` is not yet credible instead of silently inventing a bounded change
- `plan` now also assembles one explicit bounded context pack from relevant workspace files, authored input, recent traces, negotiated delivery state, and reusable Canon artifacts before proposing a goal plan
- planning now stops explicitly when that context pack is insufficient or stale instead of relying on ambient workspace state
- native planning now infers flow, targets, and verification strategy from scored workspace and session evidence instead of relying primarily on goal keywords
- default `plan` now persists one proposed goal plan with explicit `goal_plan_state`, `goal_plan_revision`, `planning_rationale`, and `verification_strategy`, and `plan --confirm` now makes that proposal executable on the native route
- repeated `plan` can now supersede the active proposal revision when evidence changes flow, task targets, or verification strategy materially, while `run`, `status`, `next`, and `inspect` keep the revision lineage and blocking reason explicit
- `run`, `status`, `next`, and `inspect` now project `context_summary`, `context_credibility`, primary inputs, provenance, and any staleness reason from the active goal plan or authoritative native trace
- the native runtime now selects explicit bounded next actions such as `read`, `search`, `modify`, `test`, `ask`, and `replan` from persisted decision state and evidence instead of treating decisions as trace-only audit records
- `status`, `next`, and `inspect` now surface selector-driven guidance, rationale, evidence basis, verification intent, and explicit no-credible-next-step wording from the same persisted decision story
- `run`, `status`, `next`, and `inspect` now project `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` across native goal-plan traces and explicit compatibility traces
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` still project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- Claude, Codex, and Copilot now ship first-class workflow assistant surfaces, while Gemini CLI guidance uses the same workflow-first vocabulary
- workflows and direct native runs now read as the two primary Synod product entry styles, while explicit compatibility follow-up remains visibly subordinate
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- direct `run --goal` now bootstraps the same evidence-driven proposal-plus-confirm planning path before execution instead of shortcutting through keyword-only flow selection
- Canon remains a bounded stage-boundary governance runtime with governed `bug-fix:investigate` plus later verify-stage `security-assessment` reuse on the same operator surface
- bounded `bug-fix` and `change` delivery no longer succeed merely because the plan ran out of steps; they now require a material diff and passed validation evidence or stop explicitly as failed
- successful governed delivery now keeps `latest_changed_files`, `latest_validation_status`, and governed packet lineage visible on the same follow-through surfaces
- adaptive compatibility execution still stays inside manifest-declared `read_targets` while surfacing candidate credibility, rejection, and exhaustion reasons explicitly
- `status` and `next` still surface `continuity_authority`, compatibility follow-up mode, and inspect-only guidance when the latest authoritative follow-up state comes from an explicit compatibility trace instead of an active session
- `run`, `status`, `next`, `inspect`, and compatibility follow-up still surface explicit `route_owner` plus material `route_config_projection` cues when workflow metadata, governance intent, or workspace-local routing defaults explain the current follow-up story
- `config show`, `run`, `status`, `next`, and `inspect` now surface effective slot routing, assistant bindings, and persisted route snapshots instead of forcing operators to reconstruct backend ownership from current config files
- native execution now rejects implementation or verification routes that are outside declared `assistant_runtimes` capabilities instead of silently accepting a hard-wired backend
- `status`, `next`, and `inspect` now surface guided next-action and stop-condition output derived from persisted session or authoritative trace evidence instead of generic lifecycle wording alone
- explicit compatibility follow-up now keeps continuity authority explicit while still projecting one evidence-backed next bounded action
- surfaced `next_command` now stays aligned with the same authoritative follow-through, workflow resume, or explicit stop condition instead of drifting back to a generic fallback
- session-native commands still accept `--cluster <primary-workspace>` so one authoritative primary-owned session can plan and deliver a bounded change across registered member repositories
- clustered `run`, `status`, `next`, and `inspect` still surface authoritative workspace, clustered execution condition, participating workspaces, and any blocking member without implying distributed orchestration ownership

## Roadmap Closure In 0.35.0

The roadmap is no longer an open-ended backlog. `0.35.0` keeps the product
surface closed and makes bounded planning explicit, so Synod now presents one
coherent execution model across CLI, assistants, workflows, routing,
governance, bounded context assembly, evidence-driven proposal selection, and
the runtime loop itself.

The governing rule remains simple: Synod is the product and execution owner.
Canon stays a bounded, useful governed runtime inside that same delivery path
rather than drifting back into a parallel tool story.

### Ongoing Compatibility Watch

Canon will continue to release versions after `0.35.0`, so Synod keeps one
explicit maintenance track for compatibility drift on the machine-facing
governance adapter rather than reopening broad product-scope work:

- revalidate the documented Canon compatibility target against the latest
  released `canon governance start|refresh|capabilities --json` `v1` surface
  whenever Canon ships a materially new stable release
- preserve additive-field tolerance and capability-aware checks so intermediate
  Canon releases do not force unnecessary Synod churn when the `v1` adapter
  contract remains stable
- schedule a new roadmap/spec slice only when Canon changes the required wire
  contract, introduces a new adapter schema version, or adds governed behavior
  that Synod needs for its explicitly modeled bounded stages

### Next Macrofeature Line (036+)

`0.35.0` delivered evidence-driven `infer -> propose -> confirm` planning plus
bounded proposal supersession. The remaining roadmap line should not decompose
back into microfeatures. Future numbered specs must stay at the macrofeature
level and each one must change Synod's operating model, not just a single CLI
surface.

Compatibility maintenance for newer Canon releases stays inside the watch above
unless it becomes contract-breaking. It does not consume the next spec number
by itself.

#### Spec 036: Canon-Grounded Reasoning And Structured Memory

- treat Canon packets, governed artifacts, and capability signals as live input
  to context assembly and decision-making, not just as end-of-stage output
- promote packet reuse, artifact invariants, and governed constraints into the
  same bounded reasoning path used for planning and verification
- add durable summarization and context compaction so long-running sessions can
  carry forward the important evidence without replaying the whole workspace

**Exit criteria**: Canon materially changes planning and decision selection when
relevant, and Synod can carry forward compact structured memory across loops.

No new numbered roadmap specs should be introduced before `036` is either
delivered, explicitly dropped, or replaced at the macrofeature level.

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
  `next`, and `inspect` on the primary Synod path
- stop planning explicitly when a credible bounded context cannot be built,
  keeping the recovery action and blocked state visible on the same session and
  trace surfaces
- update README, getting-started, configuration, assistant guidance, roadmap,
  contributor docs, and changelog for the release

### Delivered in 0.32.0

- unify assistant surfaces, routing slots, workflow entry points, and model
  bindings so Copilot, Codex, Claude, and Gemini guidance map onto the same
  Synod-owned product story instead of provider-specific command drift
- keep workflow discovery and follow-through on the same primary Synod path as
  direct native execution while preserving `route_owner`, `route_config_projection`,
  and bounded next-command cues
- keep explicit compatibility usage visibly subordinate instead of letting it
  read like a second primary product path
- close README, getting-started, configuration, assistant guidance, roadmap,
  contributor guidance, and changelog around the final product identity: users
  use Synod; Canon is visible but secondary

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

- `synod workflow list` exposes named workflows, phase chains, summary text, and invocation guidance from `.synod/workflows.toml`
- `synod workflow run <name>` and `resume` can now carry bounded `review` and `govern` phases to explicit paused, blocked, failed, or completed outcomes on the existing session-native route
- workflow progress persists in `.synod/session.json` and reuses `.synod/traces/` while direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
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

### Sequencing rule

These three features are ordered and intentionally absorb all remaining major
scope:

1. Synod must deliver real code changes before more platform abstraction work.
2. Canon must prove value inside that real delivery loop, not beside it.
3. Backend abstraction, assistant decoupling, and UX closure happen only after
  the real execution and governed-delivery story are stable.

## Architecture: User Through Execution

```text
User / Copilot / Claude
        ↓
      Synod
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

Synod is a system that takes a problem and transforms it into working code, orchestrating bounded execution itself while using Canon to govern stage outputs and provide reusable documentation.