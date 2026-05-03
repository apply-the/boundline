# Research: Unify Route Summaries And Config Projection

**Feature**: 024-unify-route-summaries  
**Date**: 2026-05-01

## R1: Converge route summaries on one shared read-side model

**Decision**: Reuse the existing session-native summary model as the canonical projection shape and migrate more workflow, review/governance, and compatibility follow-up meaning onto that shape instead of maintaining separate route-specific summary vocabularies.

**Rationale**: The roadmap explicitly prioritizes making Boundline feel like one bounded system with multiple entry paths. Reusing the current summary model preserves existing state authority and reduces duplication at the rendering layer.

**Alternatives Considered**:
- Add a second cross-route summary object just for CLI rendering: rejected because it would duplicate state mapping and increase drift risk.
- Leave route-specific summaries in place and only tweak wording: rejected because that would preserve the underlying fragmentation.

## R2: Project only material routing and config inputs

**Decision**: Surface only the configuration and routing facts that materially explain the current follow-up story, including explicit overrides, route selection, workflow metadata, and relevant workspace/global defaults.

**Rationale**: Unified summaries become misleading if they dump stale or irrelevant config. Operators need the minimum set of routing facts that explain why the current route owns follow-up.

**Alternatives Considered**:
- Show every known config value: rejected because it obscures the follow-up story and makes stale config look authoritative.
- Hide config projection entirely and rely on separate config commands: rejected because the new slice is specifically about making follow-up interpretation easier.

## R3: Preserve route ownership even when vocabulary converges

**Decision**: Keep route owner, continuity authority, and compatibility inspection guidance explicit on every aligned summary surface.

**Rationale**: The value of this slice disappears if summary convergence makes compatibility, workflow, review, or governance output look like hidden native ownership.

**Alternatives Considered**:
- Collapse all routes into one generic owner label: rejected because it hides the control path the operator must actually use next.
- Treat compatibility follow-up as resumable session state when wording aligns: rejected because it breaks the continuity model introduced in `0.22.0`.

## R4: Reuse existing persistence surfaces instead of adding a new summary store

**Decision**: Derive the unified route summary and config projection from the current session, trace, workflow, and config persistence surfaces.

**Rationale**: The repo already persists the authoritative state required for follow-up. A new summary persistence file would add synchronization risk without adding delivery value.

**Alternatives Considered**:
- Add a cached unified-summary file: rejected because it creates another state authority to reconcile.
- Recompute everything from scratch with no shared model: rejected because it would keep projection logic fragmented across CLI paths.

## R5: Close the slice as 0.24.0 with release-aligned validation

**Decision**: Reserve a version bump to `0.24.0` and include impacted docs, assistant guidance, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting as explicit implementation tasks.

**Rationale**: This slice changes how operators read the runtime across multiple existing routes. Release metadata and docs must stay aligned with the shipped behavior.

**Alternatives Considered**:
- Defer docs and release work until after the runtime lands: rejected because the feature would ship an ambiguous operator story.
- Validate with unit tests only: rejected because this slice affects shared runtime summaries and requires integration and coverage confirmation.
