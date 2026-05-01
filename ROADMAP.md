# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.21.0

Synod now has its core session-native orchestration baseline, bounded workflow
follow-through, a deeper governed-stage slice, and stronger bounded adaptive
repair depth in place:

- session-native orchestration remains the primary operator path
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` now project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- bounded `review` and `govern` phases are now executable from the workflow surface, stopping in explicit paused, blocked, failed, or completed states instead of remaining declaration-only blockers
- workflow definitions live in workspace-local `.synod/workflows.toml`, can ship optional discovery metadata, and remain bounded to existing Synod phases instead of becoming a generic workflow DSL
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- Canon is still integrated as a bounded stage-boundary governance runtime, now including governed `bug-fix:investigate` on the primary route plus later verify-stage `security-assessment` reuse
- adaptive compatibility execution can now use validation evidence to shift the next bounded repair slice without leaving manifest-declared `read_targets`

## Next Priority: Tighten Session And Compatibility Continuity

The next slice should build on `0.21.0` by tightening how explicit
compatibility runs reconnect to session and trace follow-up surfaces without
widening Synod into a general-purpose workflow engine or letting Canon take
orchestration ownership.

Suggested release target: `0.22.0`

The next spec direction should explicitly deliver three things:

- clearer continuity between explicit compatibility runs and later `status`, `next`, or `inspect` follow-up when a workspace expects one operator story
- more shared review, governance, and adaptive summaries across native and compatibility traces without hiding which route actually ran
- continued bounded orchestration ownership inside Synod without hidden background progression

### What 0.22.0 should concretely deliver

- define which persisted state is authoritative after an explicit compatibility `run`: active session, latest workspace trace, or an explicit no-session result
- make `status`, `next`, and `inspect` report that continuity model consistently instead of leaving operators to infer whether a compatibility run can be resumed or only inspected
- reuse the same adaptive, review, and governance summary vocabulary across native and compatibility traces where the concepts overlap
- keep route ownership explicit in all follow-up surfaces so compatibility execution never appears to have silently become session-native or workflow-owned
- preserve explicit terminal states when no resumable session exists instead of inventing background progression or implicit recovery

### What 0.22.0 should not do

- no hidden promotion of compatibility runs into the primary session-native route
- no generic workflow engine or background daemon for trace reconciliation
- no Canon-owned orchestration or Canon-driven follow-up selection
- no attempt to broaden adaptive mutation families in the same slice unless continuity work depends on a narrowly scoped projection fix

### Priority rationale

- this improves the operator handoff story after explicit compatibility runs instead of adding another top-level surface
- it builds on the now-stable governed-stage, workflow, and adaptive projections instead of reopening orchestration ownership questions
- it keeps Synod authoritative for orchestration while reducing ambiguity about which state or trace surface should drive the next action

### Why this comes before the other roadmap items

- the current platform value is already high enough that operator ambiguity between session state and trace state is now a more immediate problem than adding new execution power
- `0.19.0` through `0.21.0` added workflow, governance, and adaptive depth; the next useful step is to make those surfaces feel coherent when an operator moves between routes
- broader adaptive heuristics or cluster expansion will be easier to reason about once follow-up ownership and inspection semantics are explicit

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