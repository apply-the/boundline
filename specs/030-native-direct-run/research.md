# Research: Native Direct Run

**Feature**: 030-native-direct-run  
**Date**: 2026-05-02

## R1: Make direct `run --goal` native-first by default

**Decision**: Treat direct `boundline run --goal <goal>` as
an entry to the existing session-native goal-plan path by default.

**Rationale**: The current product contradiction is that the one-command run
surface still defaults to the explicit compatibility path, even though the
session-native route is documented as the primary delivery surface. Changing the
default fixes the highest-value product gap without inventing a second native
runtime.

**Alternatives Considered**:
- Keep direct run compatibility-first and tell operators to use `start`,
  `goal`, and `plan` manually: rejected because it preserves the split
  product story.
- Remove direct `run --goal` entirely: rejected because the one-command entry
  remains valuable if it leads into the primary route.

## R2: Avoid pending flow-confirmation dead ends during direct native run bootstrap

**Decision**: When direct run bootstraps a native session, confirm an inferred
built-in flow when one is credible; otherwise fall back to no-flow native
planning instead of stopping on pending flow confirmation.

**Rationale**: One-command execution should become immediately runnable. Sending
the operator through a blocked flow-confirmation detour would preserve the same
friction that direct run is meant to remove.

**Alternatives Considered**:
- Preserve pending flow confirmation and stop: rejected because it breaks the
  one-command promise.
- Always force one fixed flow such as `bug-fix`: rejected because it hides the
  current flow model and would misroute non-bug-fix goals.

## R3: Keep compatibility execution available only as explicit opt-in

**Decision**: Preserve the current execution-profile path, but only when the
operator chooses compatibility deliberately instead of reaching it implicitly
through the default direct-run surface.

**Rationale**: The compatibility route remains useful for test-oriented and
manifest-driven workflows, but it should no longer masquerade as the default
product experience.

**Alternatives Considered**:
- Delete compatibility execution immediately: rejected because existing
  compatibility tests, flows, and profiles still provide value.
- Continue inferring compatibility from the presence of goal text or briefs:
  rejected because it keeps the most visible product entry on the wrong route.

## R4: Protect meaningful active session state from silent overwrite

**Decision**: If a workspace already has active recorded, planned, or in-flight
session state, direct `run --goal` must stop explicitly instead of silently
replacing it.

**Rationale**: One-command convenience cannot come at the cost of hidden data
loss or route confusion. Explicit reset or continuation is safer and easier to
reason about.

**Alternatives Considered**:
- Always replace the active session: rejected because it destroys inspectable
  continuity without operator consent.
- Merge new goal input into active state automatically: rejected because it
  hides a major control-flow decision.

## R5: Split workspace diagnostics into native-ready and compatibility-ready paths

**Decision**: Native direct run should require only workspace, trace, and local
execution readiness, while compatibility execution continues to require a valid
execution profile.

**Rationale**: If native direct run still depends on diagnostics that insist on
`.boundline/execution.json`, the route default would change on paper but not in
practice.

**Alternatives Considered**:
- Keep one diagnostics gate that always requires the execution profile:
  rejected because it blocks the native-first story.
- Remove compatibility diagnostics entirely: rejected because explicit
  compatibility execution still needs bounded validation.

## R6: Close the slice as 0.30.0 with touched-file coverage discipline

**Decision**: Treat version bump, impacted docs, assistant guidance, changelog,
coverage above 95% for modified or created Rust files, clippy cleanup, and
formatting as first-class work for the feature.

**Rationale**: The direct run entry story is product-defining. The release must
ship with runtime behavior, prompts, docs, and validation evidence aligned.

**Alternatives Considered**:
- Defer docs and release hygiene until after runtime work: rejected because it
  risks shipping a contradictory operator story.
- Skip touched-file coverage targets: rejected because the requested release
  discipline explicitly includes them.