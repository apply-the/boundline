# Research: Expand Multi-Workspace Delivery

**Feature**: 025-multi-workspace-delivery  
**Date**: 2026-05-01

## R1: Keep the session-native path as the clustered delivery entry point

**Decision**: Extend the existing session-native delivery flow with cluster-aware
entry and follow-up instead of introducing a separate multi-workspace runtime
surface.

**Rationale**: The roadmap explicitly says the primary operator path should stay
session-native. Reusing the existing `goal -> plan -> run -> status
-> next -> inspect` story keeps orchestration authority legible and avoids
teaching users a second delivery model just to cross repository boundaries.

**Alternatives Considered**:
- Add a new independent clustered execution runtime: rejected because it would
  split orchestration ownership and duplicate follow-up semantics.
- Keep cluster support limited to status/inspection only: rejected because it
  leaves multi-repository delivery as a manual convention rather than a bounded
  Boundline capability.

## R2: Persist clustered delivery state inside existing local state surfaces

**Decision**: Record clustered delivery authority and workspace participation by
extending the existing cluster config, session, task-context, and trace
surfaces instead of creating a new cluster-session persistence file.

**Rationale**: The repository already has local cluster membership, session
state, and trace storage. A new persistence surface would add synchronization
risk without making the delivery story more bounded or inspectable.

**Alternatives Considered**:
- Add a dedicated cluster-session file: rejected because it creates another
  state authority to reconcile.
- Infer all cluster participation on demand from traces only: rejected because
  active follow-up needs stable authority even before a run reaches terminal
  trace inspection.

## R3: Keep clustered execution sequential with one active workspace at a time

**Decision**: Allow clustered delivery stories to traverse multiple member
workspaces, but keep one authoritative owner and one active workspace step live
at a time.

**Rationale**: Sequential execution stays aligned with the constitution and is
the smallest credible step toward multi-repository delivery. It also preserves
bounded reasoning about which workspace is authoritative right now.

**Alternatives Considered**:
- Parallel fan-out across cluster members: rejected because it violates the
  current sequential-first rule and would hide authority transitions.
- Precompute one full multi-workspace plan and execute it blindly: rejected
  because bounded replanning and credibility checks need to stay explicit.

## R4: Project clustered follow-up through existing summary surfaces

**Decision**: Surface clustered authority, active workspace context, and
workspace participation through existing `run`, `status`, `next`, `inspect`,
`cluster status`, and `cluster inspect` outputs rather than creating a separate
dashboard-style summary surface.

**Rationale**: The 0.24.0 slice already aligned route-summary wording across the
current delivery surfaces. Reusing those same surfaces is the smallest way to
make clustered work feel like one bounded system instead of another partially
aligned subsystem.

**Alternatives Considered**:
- Create a new cluster dashboard output model: rejected because it would drift
  from the operator path users already follow.
- Only expose clustered details in raw traces: rejected because authority and
  next-action cues must be visible before deep inspection.

## R5: Close the slice as 0.25.0 with release-aligned validation

**Decision**: Reserve the version bump to `0.25.0` and include impacted docs,
assistant guidance, changelog, coverage refresh for modified Rust files,
clippy cleanup, and formatting as explicit implementation tasks.

**Rationale**: This slice changes how operators run and interpret delivery work
across multiple repositories. The release must ship one coherent runtime and
documentation story.

**Alternatives Considered**:
- Defer documentation and version closeout until after runtime changes land:
  rejected because the clustered operator story would ship ambiguously.
- Stop at tests only and skip touched-file coverage refresh: rejected because
  the user explicitly requires release validation and the repository already
  treats coverage as part of closeout.