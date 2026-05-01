# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.23.0

Synod now has its core session-native orchestration baseline, bounded workflow
follow-through, a deeper governed-stage slice, stronger bounded adaptive
repair depth, and explicit continuity between session-native and
compatibility follow-up in place:

- session-native orchestration remains the primary operator path
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` now project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- bounded `review` and `govern` phases are now executable from the workflow surface, stopping in explicit paused, blocked, failed, or completed states instead of remaining declaration-only blockers
- workflow definitions live in workspace-local `.synod/workflows.toml`, can ship optional discovery metadata, and remain bounded to existing Synod phases instead of becoming a generic workflow DSL
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- Canon is still integrated as a bounded stage-boundary governance runtime, now including governed `bug-fix:investigate` on the primary route plus later verify-stage `security-assessment` reuse
- adaptive compatibility execution can now use broader bounded mutation families, rank one `candidate_family` over another from validation evidence, surface rejection plus exhaustion reasons, and still remain inside manifest-declared `read_targets`
- `status` and `next` now surface `continuity_authority`, compatibility follow-up mode, and inspect-only guidance when the latest authoritative follow-up state comes from an explicit compatibility trace instead of an active session
- `status`, `next`, and `inspect` now reuse one route plus `execution_condition` vocabulary across native and compatibility follow-up while keeping route ownership explicit

## Next Priority: Unify Route Summaries And Config Projection

Now that `0.23.0` broadens bounded adaptive repair while keeping continuity and
route ownership explicit, the next slice should unify more route summaries and
config projection without erasing which route actually owns the work.

Suggested release target: `0.24.x`

The next spec direction should explicitly deliver three things:

- stronger convergence of native, workflow, review, governance, and compatibility summary vocabulary
- projection of more configuration and follow-up state onto the same operator-facing surfaces
- explicit preservation of route ownership even when the summaries align more closely

### What 0.24.x should concretely deliver

- migrate more review and compatibility state onto the session-native summary model
- converge overlapping `run`, `status`, `next`, `inspect`, and workflow wording without hiding route-specific ownership
- keep config and routing projections explicit when a workspace mixes native, workflow, review, governance, and compatibility surfaces
- preserve explicit compatibility ownership even when follow-up surfaces become more uniform

### What 0.24.x should not do

- no hidden promotion of compatibility behavior into the primary session-native route
- no provider-agnostic control plane or generic workflow engine
- no Canon-owned orchestration or config-owned execution control flow
- no background daemons, distributed orchestration, or summary surfaces that hide the real authority

### Priority rationale

- continuity and follow-up ownership are now explicit enough that stronger adaptive behavior will not leave operators guessing which surface is authoritative
- adaptive repair is already useful but still intentionally narrow; expanding it is a better next return than opening another top-level product surface
- bounded orchestration ownership stays inside Synod while reuse of traces, validation evidence, and continuity-aware summaries gets deeper

### Why this comes before the other roadmap items

- `0.23.0` removed the biggest adaptive narrowness without reopening route ownership, so the next leverage point is making the operator-facing surfaces feel like one bounded system
- broader summary and config projection work now benefits from richer adaptive evidence already being explicit
- multi-workspace expansion will be easier once route summaries are less fragmented within one workspace

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

The remaining roadmap after `0.23.0` is best read as an ordered sequence rather
than an unordered backlog.

### Proposed sequence after 0.23.0

#### 0.24.x - Unify Route Summaries And Config Projection

- migrate more review and compatibility configuration onto session-native summaries without losing bounded manifest support
- converge the operator-facing summary vocabulary used by direct session, workflow, and compatibility routes
- keep route-specific ownership visible even when summaries become more uniform

This is the point where Synod should feel more like one bounded system with multiple entry paths, rather than multiple partially aligned surfaces.

#### 0.25.x - Expand Multi-Workspace Delivery

- extend multi-workspace cluster support toward full cross-repository execution planning and mutation
- preserve one authoritative orchestration story even when work spans more than one repository
- keep cluster behavior bounded and inspectable instead of drifting into distributed autonomous execution

This should stay behind continuity and summary unification work because cross-repository execution will multiply any ambiguity already present in single-workspace routing.

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