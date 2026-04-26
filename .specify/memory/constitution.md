<!--
Sync Impact Report
Version change: 1.0.0 -> 1.1.0
Modified principles:
- XII. Strict Non-Goals: expanded to allow explicitly reprioritized, bounded
	review councils and voting as delivery quality-control slices
Added sections:
- None
Removed sections:
- None
Templates requiring updates:
- updated: .specify/templates/plan-template.md
- updated: .specify/templates/spec-template.md
Related docs updated:
- None
Follow-up TODOs:
- None
-->

# Synod Spec Kit Constitution

## Core Principles

### I. Delivery Identity

- Synod MUST be specified and implemented as a delivery orchestrator.
- Features MUST directly improve multi-step engineering execution, task coordination,
	state handling, or working-code production.
- Features MUST NOT redefine Synod as a generic agent platform, chat framework, or
	prompt experimentation surface.

Rationale: a tight identity prevents drift into work that makes Synod broader but less
useful for delivery.

### II. Delivery-First Scope

- Every feature MUST answer this question clearly: does it help Synod deliver working
	code or complete engineering tasks more reliably?
- If delivery value is unclear, indirect, or speculative, the feature MUST be rejected.
- Priority order for all planning and review MUST remain: execution, orchestration,
	decomposition, validation, optimization, polish.

Rationale: Synod only earns complexity when that complexity improves delivery outcomes.

### III. No Abstract Agent Systems

- Features MUST NOT be framed as multi-agent ecosystems, collaborative AI networks,
	or general reasoning frameworks.
- An agent is valid only when it performs a concrete delivery step, consumes actionable
	input, and produces actionable output that advances execution.
- Agents that do not move the task forward MUST be removed from scope.

Rationale: agents are delivery components, not an end in themselves.

### IV. Bounded Execution

- Every execution-oriented feature MUST define explicit task start conditions, explicit
	terminal conditions, and explicit execution limits.
- Limits MUST cover maximum steps and maximum retries; budget limits SHOULD be defined
	when they materially affect task control.
- Features MUST NOT introduce infinite loops, unbounded recursion, or hidden background
	processes.

Rationale: bounded execution is required for safety, credibility, and debugging.

### V. Stateful Execution

- Synod features MUST read from shared task context and MUST write meaningful updates
	back to that context.
- Stateless execution patterns MUST be treated as invalid unless the spec explicitly
	justifies why state is unnecessary.
- Context updates MUST preserve enough history for later steps to reason about prior
	execution.

Rationale: delivery work is iterative, and iteration without state is not reliable.

### VI. Mutable Planning

- Orchestration features MUST support initial plan creation and subsequent plan mutation
	through replanning, step insertion, or step replacement.
- Plan changes MUST remain understandable and traceable to explicit evidence.
- Opaque self-modifying behavior that cannot be explained in traces MUST be rejected.

Rationale: Synod must adapt without becoming inscrutable.

### VII. Execution Over Perfect Planning

- Features MUST prefer simple plans plus iteration over complex upfront reasoning.
- Planning logic MUST optimize for starting, correcting, and converging, not for
	theoretical optimality.
- Designs that delay execution in pursuit of perfect planning MUST be rejected.

Rationale: delivery improves through controlled iteration, not exhaustive pre-analysis.

### VIII. Sequential-First Design

- All initial specs MUST assume sequential execution with one step active at a time.
- Parallelism, DAG execution, and concurrency MUST remain out of scope until a later,
	explicit prioritization changes this rule.
- Specs MUST NOT smuggle concurrency into initial designs through background workers,
	hidden branches, or implicit fan-out.

Rationale: sequential behavior is the easiest execution model to inspect and trust.

### IX. Tool-Agent Symmetry

- Steps MUST be expressible through agents, tools, or explicit evaluation logic.
- Features MUST NOT privilege reasoning paths over action paths or vice versa.
- Execution models MUST make "think", "act", and "evaluate" transitions visible.

Rationale: Synod delivers by combining reasoning and action, not by hiding one behind
the other.

### X. Required Observability

- Every feature MUST produce inspectable execution output.
- At minimum, specs and implementations MUST account for step-by-step traces, per-step
	inputs and outputs, retries, errors, replanning events, and terminal outcome.
- Systems that cannot be debugged through explicit traces MUST be rejected.

Rationale: inspectability is a requirement, not a later enhancement.

### XI. No Hidden Intelligence

- Decisions MUST be explicit, traceable, and reproducible within reasonable runtime
	limits.
- Hidden heuristics, silent failure handling, or invisible fallback behavior MUST NOT
	be introduced without surfaced evidence and rationale.
- Specs MUST describe how important decisions become visible to developers.

Rationale: trust in Synod depends on visible control flow rather than implied magic.

### XII. Strict Non-Goals

- Specs MUST NOT introduce councils or voting systems unless those capabilities are
	explicitly reprioritized in the roadmap and bounded by clear reviewer counts,
	decision rules, triggers, and terminal outcomes.
- When councils or voting are in scope, specs MUST frame them as delivery quality-
	control surfaces for reviewing bounded execution output, not as generic agent
	collaboration platforms.
- Specs MUST NOT introduce provider abstraction complexity beyond what is required to
	execute the explicitly prioritized review slice, and MUST NOT introduce distributed
	agent systems, memory systems beyond task scope, UI or UX work, or deployment
	pipelines in the default roadmap path.
- Out-of-scope work MUST be named explicitly rather than left ambiguous.

Rationale: clear exclusions protect the delivery core from premature expansion.

### XIII. Minimal Capability Slices

- Each spec MUST introduce one core capability that can be implemented in isolation.
- The capability MUST deliver immediate, tangible value once shipped.
- Bundled systems, speculative extensions, and future-proof abstractions MUST be
	rejected unless a smaller slice cannot deliver the same value.

Rationale: minimal slices reduce risk and keep progress measurable.

### XIV. Real Acceptance Criteria

- Every spec MUST include acceptance scenarios grounded in real engineering tasks.
- Acceptance scenarios MUST involve actual execution, not reasoning-only narratives.
- Each spec MUST cover both successful execution and at least one non-success path such
	as retry, replanning, failure, or exhaustion.

Rationale: real acceptance criteria keep the work tied to delivery behavior.

### XV. Failure as a First-Class Path

- Specs MUST define what happens when a step fails, when retries occur, when replanning
	occurs, and when execution stops.
- Failure handling MUST be treated as core behavior, not as optional polish.
- Features that ignore failure paths MUST be considered incomplete.

Rationale: delivery systems are only credible when they handle failure explicitly.

### XVI. Separation From External Systems

- Specs MUST NOT depend on Canon behavior, external persistence models, or governance
	runtimes in order to function.
- Synod features MUST remain independently testable and executable.
- External systems MAY receive outputs later, but they MUST NOT define Synod's core
	control flow for the feature under review.

Rationale: Synod must remain a usable delivery engine even when adjacent systems evolve.

### XVII. Evolution Without Premature Lock-In

- Every feature MUST be the simplest version that works while leaving room for later
	extension.
- Extensibility MUST come from clear primitives, not rigid frameworks or speculative
	architecture.
- Designs that lock future architecture too early MUST be rejected.

Rationale: Synod needs growth paths without paying for future complexity before it is
needed.

### XVIII. Done Means Executable Delivery

- A feature is complete only when it executes a real multi-step task, handles at least
	one failure scenario correctly, produces a usable execution trace, and can be reasoned
	about by a developer without guesswork.
- Features that are impressive in concept but not useful for delivery MUST be rejected.

Rationale: completion is defined by delivery behavior, not by conceptual ambition.

## Specification Standards

- Every feature spec MUST state the user value in terms of bounded engineering tasks and
	MUST define explicit scope boundaries and non-goals.
- Every feature spec MUST describe task state, execution steps, recovery behavior,
	terminal conditions, and trace expectations whenever the feature touches orchestration
	or execution.
- Every feature spec MUST use concrete engineering scenarios that include both a success
	path and at least one failure, retry, replanning, or exhaustion path.
- Every feature spec MUST avoid language that implies hidden heuristics, autonomous
	background work, or capabilities delegated to external governance systems.
- Every feature spec SHOULD choose the smallest viable capability slice; when a broader
	scope is necessary, the author MUST explain why a smaller slice would fail.

## Planning & Review Workflow

- Every plan MUST perform a Constitution Check against these principles and record pass
	or fail decisions with justification.
- Every task breakdown MUST include work for validation, failure handling, and
	observability whenever those concerns appear in the spec; for Synod delivery features,
	reviewers SHOULD expect those concerns by default.
- Reviews MUST reject features that invert the delivery-first priority order or that
	reintroduce deferred non-goals without an approved amendment.
- Non-compliant specs, plans, and tasks MUST be revised before implementation begins.
- Implementation is not done until the execution behavior, failure handling, and trace
	expectations required by Principle XVIII are demonstrably satisfied.

## Governance

- This constitution supersedes conflicting local habits, template defaults, and feature-
	level preferences.
- Amendments MUST include a written summary of the change, impacted principles or
	sections, a Sync Impact Report, and updates to affected templates or guidance files.
- Amendments MUST be approved by Synod maintainers before they are treated as active
	governance.
- Versioning policy MUST use semantic versioning: MAJOR for removed or redefined
	principles, MINOR for new principles or materially expanded guidance, and PATCH for
	clarifications that preserve existing meaning.
- Compliance review expectations are mandatory for every change to specs, plans, tasks,
	templates, or workflow guidance. Reviews MUST either confirm compliance or propose a
	constitution amendment.
- Ratification and amendment dates MUST use ISO format. An amendment is incomplete until
	this file, the Sync Impact Report, and dependent artifacts are in sync.

**Version**: 1.1.0 | **Ratified**: 2026-04-23 | **Last Amended**: 2026-04-26
