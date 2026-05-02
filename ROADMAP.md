# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.31.0

Synod now has its core session-native orchestration baseline, bounded workflow
follow-through, deeper governed-stage plus adaptive slices, explicit
continuity between session-native and compatibility follow-up, stronger
route-summary plus config projection, bounded multi-workspace clustered
delivery, negotiated delivery modeling, inspectable routing plus assistant
decoupling, guided decision follow-through, evidence-aligned next-command
selection, and credible governed delivery completion in place:

- session-native orchestration remains the primary operator path
- `capture` now derives one negotiated delivery packet from direct goals, authored briefs, and governance context before planning begins
- `plan` now stops early when `negotiation_resolution` is not yet credible instead of silently inventing a bounded change
- `run`, `status`, `next`, and `inspect` now project `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` across native goal-plan traces and explicit compatibility traces
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` still project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
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

## Final Sequence After 0.31.0

The roadmap is no longer an open-ended backlog. The remaining work collapses
into the final closing feature that finishes the product.

The governing rule is simple: Synod remains the product and execution owner.
Canon must stay a bounded, useful governed runtime inside that same delivery
path rather than drifting back into a parallel tool story.

### Feature 032 - Product Unification And Surface Closure

**Outcome**: the system closes as one coherent product instead of a collection
of partially overlapping runtimes and assistant surfaces.

**This feature closes**:

- unify assistant surfaces, routing slots, workflow entry points, and model
  bindings so Copilot, Codex, Claude, Gemini, and future backends map onto the
  same Synod-owned product story instead of provider-specific command packs
- finish the backend abstraction work needed to keep model/provider choice
  inspectable without reopening hidden provider control planes
- remove remaining product ambiguity between session-native execution,
  workflow-driven execution, and explicit compatibility usage by making one path
  primary and the others clearly subordinate
- close docs, operator guidance, release narrative, and inspection output around
  the final product identity: users use Synod; Canon is visible but secondary

**Done means**:

- Synod presents one coherent UX and one coherent execution model across CLI,
  assistants, workflows, routing, and governance
- the remaining platform work fits inside maintenance and refinement rather than
  another major architectural reset

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