# Authority-Zoned Delivery Roles, Personas, and Review Councils

## Status

Proposed Cross-Repo Specification

## Scope

Canon + Boundline

---

# 1. Outcome

This specification defines the next major governance and runtime alignment layer between Canon and Boundline.

The goal is to transform review councils from a lightweight review feature into a complete admission-control system for AI-assisted software delivery.

The system must support:

- authority-zoned governance
- runtime role orchestration
- bounded review councils
- independent challenge
- structured findings
- operational stop semantics
- proportional control
- explainable governance posture

The system must preserve the separation between:

```text
Canon = governed semantics and authority
Boundline = runtime orchestration and execution control
```

---

# 2. Product Thesis

AI-assisted delivery is no longer constrained by generation capacity.

It is constrained by judgment quality, admission control, and coherence preservation.

Cheap generation without structured challenge creates systemic architectural drift.

The missing layer is not more autonomy.

The missing layer is explicit control over what is allowed to cross the acceptance boundary.

This specification introduces:

- authority zones
- change classes
- runtime roles
- review councils
- adjudication
- stop semantics
- control graduation

The system is designed around the following principles:

```text
structure before autonomy
independent challenge before acceptance
proportional control over generated work
cheap generation, scarce judgment
```

---

# 3. Architectural Boundary

## 3.1 Canon Owns

Canon owns:

- authority-zone vocabulary
- change-class vocabulary
- intended personas
- persona anti-behaviors
- governed packet metadata
- approval state
- readiness state
- lineage and promotion metadata
- project memory
- evidence publication
- governance semantics

Canon defines:

```text
what the work means
what risk posture it belongs to
what governance semantics apply
```

Canon does NOT:

- orchestrate runtime councils
- dispatch subagents
- choose providers
- choose models
- define retry policy
- define stop behavior
- execute runtime reviews

---

## 3.2 Boundline Owns

Boundline owns:

- runtime orchestration
- runtime role taxonomy
- domain-expert selection
- council composition
- review algorithms
- adjudication
- routing
- stop semantics
- human gates
- runtime traces
- council lifecycle structure
- structural matrices

Boundline defines:

```text
how work moves
who reviews it
when execution must stop
how findings become operational
```

S2 may emit candidate reviewer capabilities.
S3 determines whether governance policy requires them operationally.

---

# 4. Shared Contract Vocabulary

## 4.1 Canon Vocabulary

Canon publishes stable governed semantics for:

```text
authority_zone
change_class
intended_persona
persona_anti_behaviors
approval_state
packet_readiness
risk
primary_artifact
artifact_order
promotion_refs
stage_role_hints
```

---

## 4.2 Authority Zones

Canonical authority zones:

```text
green
yellow
red
restricted
```

### Green

Low-governance bounded work.

Examples:
- exploratory discovery
- bounded implementation
- local refactors
- isolated backlog work

### Yellow

Work requiring independent challenge or bounded structural validation.

Examples:
- architecture refinement
- bounded migrations
- contract-affecting implementation
- domain-model changes

### Red

High-risk structural or operational work.

Examples:
- public API changes
- authentication changes
- database ownership changes
- critical migrations
- production-impacting operations

### Restricted

Human-gated destructive or approval-required operations.

Examples:
- destructive migrations
- irreversible production actions
- regulated or approval-bound delivery

---

## 4.3 Change Classes

Canonical change classes:

```text
low-impact
bounded-impact
systemic-impact
critical-operations
```

---

# 5. Canon Personas

Every governed Canon mode MUST define:

- intended persona
- intended posture
- intended audience
- anti-behaviors

Example:

```text
Mode:
architecture

Persona:
System Architect

Anti-behaviors:
- implementation-level tunnel vision
- local optimization over system coherence
- introducing technology without rationale
- expanding scope beyond bounded architecture concerns
```

---

# 6. Runtime Roles

Boundline runtime roles are distinct from Canon personas.

Runtime roles include:

```text
planner
implementer
reviewer
verifier
arbiter
security-reviewer
architecture-reviewer
migration-reviewer
operations-reviewer
domain-expert
```

The system MUST distinguish between:

```text
authoring persona
runtime role
model route
provider capability
final decision authority
```

---

# 7. Domain Expert Selection

Boundline selects domain experts dynamically from:

- repository signals
- workspace manifests
- target surfaces
- active routes
- runtime configuration
- Canon hints
- pack metadata

Domain experts remain a Boundline concern.

Canon may provide hints but never executable assignments.

---

# 8. Effective Control Resolution

Boundline computes:

```text
effective_control_class = max(
  authority_zone_floor,
  change_class_floor,
  stage_floor,
  assurance_floor,
  evidence_floor
)
```

---

## 8.1 Stage Floors

### Green Floor

```text
discovery
requirements
```

### Yellow Floor

```text
domain-language
domain-model
system-shaping
architecture
backlog
change
implementation
verification
review
refactor
pr-review
```

### Red Floor

```text
incident
migration
security-assessment
system-assessment
supply-chain-analysis
```

### Restricted Floor

```text
destructive operations
approval-required actions
```

---

# 9. Council Trigger Policy

Councils are NOT always-on.

Councils activate proportionally based on:

- authority zone
- change class
- stage
- evidence quality
- assurance profile
- governance requirements
- public contract exposure
- operational criticality

The safe path must remain faster than the unsafe path.

---

# 10. Adaptive Governance Boundary

This specification defines the structural governance options only.

`catch`, `rule`, and `hook` are valid governance control levels, but their
runtime progression is owned by S4.

S3 answers:

- which governance structures exist
- which zones and profiles are available
- which stop states are structurally valid

S4 answers:

- when a control level activates
- when it degrades
- when it escalates
- when it is promoted or rolled back

---

# 11. Control-Level Compatibility

Cold-start, advisory rollout, and progressive governance adoption are owned by
S4.

This specification only requires that any active council profile can be run in
more than one control level.

The timing and evidence thresholds for initial rollout are not defined here.

---

# 12. Structural Reviewer Requirements

Reviewer credibility is a required runtime input, but its dynamic evaluation is
owned by S4 and extended by S6.

S3 only requires that council structures can express:

- mandatory reviewers
- mandatory capabilities
- role-gated acceptance
- reviewer independence requirements

How credibility changes confidence, degradation, or escalation is not owned by
this specification.

---

# 13. Structured Findings

Every council finding MUST be persisted.

Example:

```json
{
  "finding_id": "security:001",
  "reviewer_id": "security-reviewer",
  "runtime_role": "security-reviewer",
  "severity": "high",
  "disposition": "block",
  "summary": "Authorization boundary is underspecified.",
  "details": "Ownership validation is implicit.",
  "required_action": "Add explicit invariant and verification coverage.",
  "confidence": "high",
  "evidence_refs": [
    "file:src/auth.rs",
    "trace:abc123"
  ]
}
```

---

# 14. Producer Response Protocol

Findings must become operational.

For every:

```text
concern
block
```

the producer MUST respond with:

```text
accepted
rejected
deferred
```

plus rationale.

Accepted findings MUST generate:
- follow-up tasks
- remediation work
- plan updates

The producer cannot silently ignore blocking findings.

---

# 15. Stop Semantics Continuum

This section defines the structural stop-semantics vocabulary.

The runtime supports:

```text
proceed
proceed_with_advisory
proceed_with_warning
degraded_proceed
council_required
adjudication_required
human_gate_required
hard_stop
```

S3 owns the vocabulary.

S3 defines legal states.
S4 defines runtime transitions between those states.

S4 owns the transitions between these states.

---

# 16. Hard-Stop Escalators

The following conditions are structurally defined as hard-stop triggers:

- unsupported contract line
- missing required approval
- blocked Canon governance
- missing required artifacts
- missing mandatory reviewer role
- reviewer independence failure
- restricted action without human gate
- required council cannot be assembled
- no credible domain expert exists
- unresolved blocking finding

This section defines policy shape.

S4 owns when and how the runtime transitions into `hard_stop`.

---

# 17. V1 Algorithms

## Required V1 Algorithms

### independence_guard

Prevents fake councils.

```text
reviewers counted in quorum
must not collapse onto the same effective route
```

---

### majority_vote

Simple majority acceptance.

---

### weighted_majority

Reviewer authority carries different weight.

---

### reject_on_blocking

Mandatory blocking reviewers can veto continuation.

---

### quorum

Minimum reviewer participation required.

---

### role_gated_acceptance

Mandatory reviewer roles cannot object unresolved.

---

### adjudicated_review

Mixed councils escalate to independent arbiter.

---

### producer_response_protocol

All concern/block findings require producer response.

---

# 18. Reasoning Extension Boundary

Advanced reasoning profiles are not defined here.

Profiles such as:

- self_consistency
- blind_double_check
- heterogeneous_consensus
- reflexion_loop
- multi_agent_debate

are owned by S6.

S3 imposes one strict constraint on them:

Reasoning profiles extend governance councils.
They do not replace governance councils.

No reasoning profile may introduce:

- a second council system
- a second adjudication ladder
- a second stop-semantics vocabulary
- a second governance-orchestration stack

---

# 19. Operational Cost Boundary

Operational cost estimation is not owned by this specification.

S4 and S6 may expose runtime cost for active governance or reasoning profiles,
but S3 only requires that selected council profiles remain inspectable.

---

# 20. Council Profiles

## V1 Profiles

### none

No council.

---

### light_single

Single reviewer or self-critique.

Non-blocking.

---

### yellow_pair

Two reviewers.

Algorithms:
- majority_vote
- reject_on_blocking

---

### red_five

Five reviewers.

Algorithms:
- weighted_majority
- quorum
- role_gated_acceptance
- adjudication

---

### restricted_manual

Red council plus mandatory human gate.

---

# 21. V1 Matrix

| Zone | Risk | Stage | Council Profile | Stop Semantics |
|---|---|---|---|---|
| green | low-impact | discovery, requirements | none | proceed |
| green | low-impact | implementation, refactor | light_single | proceed |
| green | bounded-impact | architecture, domain-model | yellow_pair | council_required |
| yellow | bounded-impact | implementation, verification | yellow_pair | council_required |
| yellow | systemic-impact | structural work | red_five | adjudication_required |
| red | any structural risk | architecture, migration, security | red_five | human_gate_required |
| restricted | any | destructive operations | restricted_manual | hard_stop on unresolved state |

---

# 22. CLI And Trace Projection

Boundline MUST surface:

- runtime roles
- selected domain experts
- council profile
- findings
- producer responses
- adjudication results
- stop semantics
- effective control class

in:

```text
plan
run
status
next
inspect
```

---

# 23. Non-Goals

This specification does NOT:

- make Canon a runtime orchestrator
- make councils always-on
- replace deterministic validation
- replace tests
- replace security scanning
- replace human approval
- introduce provider-specific governance logic
- make debate default
- introduce a new standalone governance registry
- turn Boundline into uncontrolled swarm orchestration

---

# 24. Documentation Requirements

## Boundline

Required documents:

```text
docs/review-council-algorithms.md
docs/authority-zones-and-stop-semantics.md
docs/council-adoption-guide.md
```

---

## Canon

Required documents:

```text
docs/governed-personas-and-authority-zones.md
```

---

# 25. Success Criteria

The system succeeds when:

1. Low-risk work remains cheap.
2. High-risk work receives proportional challenge.
3. Findings become operational.
4. Governance and execution remain separated.
5. Runtime behavior is explainable.
6. Review councils are inspectable and bounded.
7. Human gates remain authoritative.
8. Council structures remain compatible with progressive adoption in S4.
9. Dynamic degradation and escalation remain outside this specification.
10. Canon remains the source of governed semantics.

---

# 26. Final Thesis

Boundline should evolve into an authority-zoned admission-control runtime for
AI-assisted software delivery.

Canon defines governed posture.

S3 defines the static governance posture that Boundline can execute:

- zones
- council profiles
- structural algorithms
- mandatory reviewers
- stop-semantics vocabulary

The system exists to preserve coherence while scaling AI-assisted delivery.

The objective is not more generation.

The objective is governed acceptance.
