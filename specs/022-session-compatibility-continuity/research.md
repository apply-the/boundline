# Research: Session And Compatibility Continuity

**Feature**: 022-session-compatibility-continuity  
**Date**: 2026-05-01

## R1: Reuse existing session and trace state before adding new persistence

**Decision**: Derive compatibility follow-up continuity from the existing active session record plus the latest workspace trace instead of introducing a new persistence file in the first slice.

**Rationale**: The roadmap explicitly prefers reuse of existing session and trace surfaces before adding a new runtime surface. The current system already persists session-native state in `.synod/session.json` and compatibility traces in `.synod/traces/`.

**Alternatives Considered**:
- Add a new compatibility continuity file under `.synod/`: rejected because it widens the persistence model before proving that existing state is insufficient.
- Copy compatibility results into the active native session as if they were native: rejected because it blurs route ownership and risks hiding which path actually ran.

## R2: Teach `status` and `next` to reason about compatibility follow-up explicitly

**Decision**: Extend `status` and `next` so they can explain compatibility follow-up state explicitly instead of only succeeding when an active native session fully explains the next action.

**Rationale**: `inspect` already falls back to the latest workspace trace, but the operator handoff still breaks because `status` and `next` remain session-only concepts. This slice delivers more value by clarifying those commands than by adding another top-level command.

**Alternatives Considered**:
- Add a new compatibility-only follow-up command: rejected because it adds another operator surface instead of tightening continuity across the existing ones.
- Leave `status` and `next` unchanged and document the limitation: rejected because the roadmap now prioritizes continuity as runtime behavior, not just as documentation.

## R3: Keep route ownership explicit even when summary wording converges

**Decision**: Reuse the same summary vocabulary for adaptive, review, governance, and terminal concepts across native and compatibility traces, while keeping route attribution explicit in every follow-up surface.

**Rationale**: Shared vocabulary improves operator comprehension, but only if it does not imply that native and compatibility routes are actually the same execution mode.

**Alternatives Considered**:
- Keep route-specific wording everywhere: rejected because the operator has to relearn synonymous concepts between routes.
- Fully normalize outputs and hide route-specific distinctions: rejected because it violates the explicit-intelligence and route-ownership constraints in the roadmap and constitution.

## R4: Prefer latest persisted workspace trace over hidden resumability assumptions

**Decision**: When a compatibility run leaves a latest workspace trace but no resumable compatibility session, later commands should surface inspect-oriented continuity instead of implying that execution can be resumed implicitly.

**Rationale**: This keeps non-success and non-resumable states explicit and prevents hidden background progression.

**Alternatives Considered**:
- Infer resumability from terminal status alone: rejected because terminal state is not the same as session ownership.
- Pretend compatibility runs always produce a resumable session: rejected because that silently promotes compatibility execution into the primary session model.

## R5: Close the slice as 0.22.0 with full release hygiene and validation

**Decision**: Reserve a version bump to `0.22.0` and include docs, assistant guidance, changelog, coverage refresh for modified Rust files, clippy cleanup, and `cargo fmt` as explicit implementation tasks.

**Rationale**: The user requested full release closeout, and the slice materially changes the operator-facing route story.

**Alternatives Considered**:
- Defer release hygiene until after runtime changes land: rejected because the route story changes the documented product behavior.
- Limit validation to targeted tests only: rejected because this slice touches cross-cutting CLI follow-up behavior and should close with repository-standard gates.