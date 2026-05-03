# Research: Inspectable Routing And Assistant Decoupling

**Feature**: 027-routing-assistant-decoupling  
**Date**: 2026-05-01

## R1: Project routing decisions through existing session and trace surfaces

**Decision**: Represent active slot-routing decisions as part of the existing
session and trace story instead of creating a new routing-status file or a new
top-level command.

**Rationale**: Operators already use `run`, `status`, `next`, and `inspect` to
understand what Boundline is doing. Reusing those surfaces delivers immediate value
without teaching a second workflow or splitting authority away from the current
session and compatibility follow-up model.

**Alternatives Considered**:
- Add a dedicated routing-inspection command: rejected because it would create a
  second read-side workflow for information that materially shapes existing
  execution and follow-up decisions.
- Persist routing decisions in a new file under `.boundline/`: rejected because it
  would duplicate information already derivable from config, session, and trace
  state.

## R2: Bind assistant/backend selection from resolved slot routing instead of hard-wired defaults

**Decision**: Use the already-resolved slot routing as the authority for which
assistant/backend family should be bound to a bounded delivery slot.

**Rationale**: The current config surface already expresses runtime and model by
slot. Reusing that same resolved route is the smallest coherent way to reduce
hard-wired assistant coupling while keeping one authoritative explanation for
why a backend was chosen.

**Alternatives Considered**:
- Add a separate assistant-binding config tree: rejected because it would create
  two competing authorities for one routing decision.
- Keep assistant selection hard-coded and only improve display output: rejected
  because visibility without actual binding would preserve the core coupling
  problem.

## R3: Keep command names and orchestration ownership stable

**Decision**: Preserve the existing command names, primary session-native flow,
and explicit compatibility path while changing only how backend choice is
explained and bound.

**Rationale**: The roadmap asks for inspectable backend routing, not for a new
product surface. Preserving existing command vocabulary keeps the slice small
and prevents assistant/backend choice from being mistaken for a second
orchestration runtime.

**Alternatives Considered**:
- Introduce assistant-specific execution commands: rejected because it would
  dilute Boundline's ownership of orchestration.
- Introduce a provider-gateway surface first: rejected because it expands scope
  before the simpler inspectable-binding slice proves its value.

## R4: Keep compatibility and clustered authority explicit

**Decision**: When routing or assistant binding is projected on explicit
compatibility follow-up or clustered delivery, the output must preserve the same
route-ownership and cluster-authority cues already used elsewhere.

**Rationale**: The current product story depends on explicit authority. Routing
visibility is only useful if it does not suggest that compatibility became
session-native or that member workspaces now own clustered control flow.

**Alternatives Considered**:
- Synthesize a session-owned route story for compatibility traces: rejected
  because it hides the real owning route.
- Duplicate routing authority into cluster members: rejected because primary
  workspace ownership is already the bounded cluster model.

## R5: Close the slice as 0.27.0 with release-aligned validation

**Decision**: Treat version bump, impacted docs, assistant guidance, changelog,
touched-Rust coverage refresh, clippy cleanup, and formatting as first-class
tasks for the feature rather than post-hoc cleanup.

**Rationale**: This slice changes how operators and assistants explain backend
choice. The release must ship as one coherent story across runtime output,
assistant packs, documentation, and validation evidence.

**Alternatives Considered**:
- Defer release closeout until after implementation: rejected because it risks a
  mismatch between runtime behavior and assistant or operator guidance.
- Skip touched-file coverage refresh: rejected because the requested delivery
  discipline explicitly includes coverage for modified or created Rust files.