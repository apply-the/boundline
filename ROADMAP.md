# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.25.0

Synod now has its core session-native orchestration baseline, bounded workflow
follow-through, a deeper governed-stage slice, stronger bounded adaptive
repair depth, explicit continuity between session-native and compatibility
follow-up, stronger route-summary plus config projection, and bounded
multi-workspace clustered delivery in place:

- session-native orchestration remains the primary operator path
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` now project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- bounded `review` and `govern` phases are now executable from the workflow surface, stopping in explicit paused, blocked, failed, or completed states instead of remaining declaration-only blockers
- workflow definitions live in workspace-local `.synod/workflows.toml`, can ship optional discovery metadata, and remain bounded to existing Synod phases instead of becoming a generic workflow DSL
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- Canon is still integrated as a bounded stage-boundary governance runtime, now including governed `bug-fix:investigate` on the primary route plus later verify-stage `security-assessment` reuse
- adaptive compatibility execution can now use broader bounded mutation families, rank one `candidate_family` over another from validation evidence, surface rejection plus exhaustion reasons, and still remain inside manifest-declared `read_targets`
- `status` and `next` now surface `continuity_authority`, compatibility follow-up mode, and inspect-only guidance when the latest authoritative follow-up state comes from an explicit compatibility trace instead of an active session
- `status`, `next`, and `inspect` now reuse one route plus `execution_condition` vocabulary across native and compatibility follow-up while keeping route ownership explicit
- `run`, `status`, `next`, `inspect`, and compatibility follow-up now also surface explicit `route_owner` plus material `route_config_projection` cues when workflow metadata, governance intent, or workspace-local routing defaults explain the current follow-up story
- session-native commands now accept `--cluster <primary-workspace>` so one authoritative primary-owned session can plan and deliver a bounded change across registered member repositories
- clustered `run`, `status`, `next`, and `inspect` now surface authoritative workspace, clustered execution condition, participating workspaces, and any blocking member without implying distributed orchestration ownership

## Next Priority: Goal Negotiation And Constraint Modeling

Now that `0.25.0` makes bounded multi-workspace delivery feel like one operator
story with one authoritative owner, the next slice should make acceptance
boundaries and constraint tradeoffs more explicit before execution begins.

Suggested release target: `0.26.x`

The next spec direction should explicitly deliver three things:

- richer goal negotiation before planning locks a bounded change
- explicit constraint modeling for scope, risk, and acceptance boundaries
- operator-visible tradeoffs instead of hiding those decisions inside planning heuristics

### What 0.26.x should concretely deliver

- add goal negotiation and constraint capture before execution begins
- make acceptance boundaries and tradeoffs explicit in the session-owned story
- keep negotiation outputs inspectable instead of burying them in planner internals

### What 0.26.x should not do

- no background daemons, distributed autonomous workers, or hidden fan-out control loops
- no provider-agnostic control plane or generic workflow engine
- no Canon-owned orchestration or config-owned execution control flow
- no negotiation layer that hides which constraints are binding or why a tradeoff was chosen

### Priority rationale

- clustered delivery authority is now explicit enough that more advanced planning can stay inspectable instead of becoming hidden negotiation logic
- goal and constraint capture become more valuable once session, continuity, and cluster semantics are already stable
- bounded orchestration ownership stays inside Synod while the reasoning that shapes a plan becomes easier for operators to challenge before execution begins

### Why this comes before the other roadmap items

- `0.25.0` made multi-workspace execution feel like one bounded system, so the next leverage point is improving what the system decides before it mutates code
- explicit route and cluster authority now make constraint negotiation easier to project without hiding ownership
- stronger upfront constraint modeling is more tractable now that read-side follow-up surfaces are already aligned across workspace, compatibility, workflow, governance, and cluster states

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

### Secondary follow-up directions

The remaining roadmap after `0.25.0` is best read as an ordered sequence rather
than an unordered backlog.

### Proposed sequence after 0.25.0

#### 0.26.x - Goal Negotiation And Constraint Modeling

- add more advanced goal negotiation and constraint modeling
- make acceptance boundaries, tradeoffs, and delivery limits more explicit before execution begins
- keep these constraints operator-visible rather than hiding them inside planning heuristics

This is strategically important, but it becomes much more valuable once the runtime surfaces, continuity, and cluster semantics are already stable.

### Decision rule for sequencing

When choosing between roadmap items, prefer the slice that most improves:

- explicit operator understanding of what Synod will do next
- bounded orchestration ownership inside Synod
- reuse of existing traces, summaries, and session state before introducing a new surface or broader search space

To be challenged:
- add a provider-agnostic model gateway inside Synod, with first-class provider auth flows and capability discovery
- decouple assistant command packs from model backends so Claude/Codex/Copilot surfaces map to routing slots instead of hard-wired providers

These are intentionally below the main roadmap because they widen platform scope,
increase integration cost, and risk distracting from the still-unfinished core of
bounded orchestration continuity and delivery execution.

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