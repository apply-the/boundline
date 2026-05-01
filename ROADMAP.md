# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.22.0

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
- adaptive compatibility execution can now use validation evidence to shift the next bounded repair slice without leaving manifest-declared `read_targets`
- `status` and `next` now surface `continuity_authority`, compatibility follow-up mode, and inspect-only guidance when the latest authoritative follow-up state comes from an explicit compatibility trace instead of an active session
- `status`, `next`, and `inspect` now reuse one route plus `execution_condition` vocabulary across native and compatibility follow-up while keeping route ownership explicit

## Next Priority: Broaden Bounded Adaptive Repair

Now that `0.22.0` makes explicit compatibility follow-up usable through
`status`, `next`, and `inspect`, the next slice should strengthen bounded
adaptive repair itself without reopening route ownership or continuity
semantics.

Suggested release target: `0.23.x`

The next spec direction should explicitly deliver three things:

- broader bounded mutation families beyond the current local heuristics
- stronger adaptive exhaustion and credibility handling when the current bounded slice stops looking plausible
- continued reuse of the new continuity-aware `status`, `next`, and `inspect` summary model so stronger adaptive behavior still ends in explicit follow-up

### What 0.23.x should concretely deliver

- expand adaptive change kinds beyond the current arithmetic, comparison, and boolean flips while keeping generation deterministic and bounded
- improve how adaptive runs explain exhaustion, replacement-slice credibility, and why a bounded candidate was or was not chosen next
- keep adaptive selection bounded to manifest-declared scope and explicit validation evidence instead of drifting into open-ended mutation
- preserve explicit compatibility ownership even when adaptive traces get richer or more recoverable

### What 0.23.x should not do

- no open-ended autonomous code generation outside the manifest-bounded compatibility surface
- no hidden promotion of compatibility repair into the primary session-native route
- no generic workflow engine or background daemon for adaptive retries
- no Canon-owned orchestration or Canon-driven adaptive planning

### Priority rationale

- continuity and follow-up ownership are now explicit enough that stronger adaptive behavior will not leave operators guessing which surface is authoritative
- adaptive repair is already useful but still intentionally narrow; expanding it is a better next return than opening another top-level product surface
- bounded orchestration ownership stays inside Synod while reuse of traces, validation evidence, and continuity-aware summaries gets deeper

### Why this comes before the other roadmap items

- `0.22.0` removed the immediate continuity ambiguity, so the next leverage point is making the bounded compatibility path more capable without changing its ownership model
- stronger adaptive repair benefits directly from the clearer `status` and `next` follow-up semantics now in place
- multi-workspace expansion and broader summary unification will be easier once adaptive compatibility behavior is less narrow

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

The remaining roadmap after `0.22.0` is best read as an ordered sequence rather
than an unordered backlog.

### Proposed sequence after 0.22.0

#### 0.23.x - Broaden Bounded Adaptive Repair

- expand adaptive mutation families beyond the current bounded local heuristics
- keep adaptive selection bounded to manifest-declared scope and explicit validation evidence
- improve credibility and exhaustion behavior without turning Synod into open-ended autonomous code generation

This should come after continuity work because stronger adaptive behavior is more useful once follow-up state and trace ownership are unambiguous.

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