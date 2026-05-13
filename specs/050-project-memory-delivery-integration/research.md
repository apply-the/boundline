# Research: Project Memory Delivery Integration

## Decision 1: Consumer-Side Type Ownership

**Decision**: Define Boundline-owned consumer types (`PromotionStateView`,
`LineageRef`, `ProjectMemoryContext`) that mirror Canon's vocabulary without
importing Canon crate dependencies.

**Rationale**: Boundline and Canon are separate repositories with independent
release cycles. Direct crate dependencies would couple Boundline's build to
Canon's release schedule. Mirroring the vocabulary through string-based or
enum-based mapping keeps the consumer independently compilable while the shared
contract document governs semantic alignment.

**Alternatives considered**:
- Shared crate dependency: rejected because it couples release schedules and
  violates the constitutional principle of separation from external systems.
- Raw string-based consumption with no types: rejected because it would
  lose compile-time safety for promotion-state handling.

## Decision 2: Contract-Version Compatibility Strategy

**Decision**: Pin Boundline's initial consumer support to the Canon
`0.1.x` contract line. Treat `0.1.x` as compatible and reject malformed or
future contract lines until the integration slice is explicitly updated.

**Rationale**: The contract remains pre-1.0, so no compatibility grace period
is guaranteed across minor versions. A pinned `0.1.x` window matches the actual
consumer posture, keeps behavior explicit, and avoids implying support that the
consumer has not verified.

**Alternatives considered**:
- Per-field capability negotiation: rejected as over-engineered; the contract
  already specifies required vs. additive fields.
- Major-version-only compatibility: rejected because it would overstate support
  during the contract's pre-1.0 phase.
- No compatibility check: rejected because the spec requires explicit failure
  on incompatible versions.

## Decision 3: Canon Output Discovery Location

**Decision**: Read Canon-promoted output from well-known paths documented in
the shared contract (`docs/project/`, `docs/evidence/`) relative to the
workspace root, using adjacent `<surface>.packet-metadata.json` sidecars for
lineage and promotion metadata. Discovery is path-based, not registry-based.

**Rationale**: The shared contract already specifies the stable target paths and
the current Canon implementation emits file-adjacent packet-metadata sidecars.
A registry would add a moving part without adding value in the initial slice.

**Alternatives considered**:
- Central manifest file listing all Canon outputs: rejected because the
  contract already enumerates stable surfaces.
- Git-based discovery (tracking Canon commits): rejected as out of scope and
  coupling to Canon's VCS workflow.

## Decision 4: Session Context Storage

**Decision**: Store the `ProjectMemoryContext` snapshot inside the existing
session task context (`session.json`), not in a separate file.

**Rationale**: The session task context is the established mechanism for
carrying delivery-relevant state between stages. A separate file would add a
new persistence surface without justification.

**Alternatives considered**:
- Separate `.boundline/project-memory-context.json`: rejected because it
  duplicates what session context already handles.
- In-memory only: rejected because downstream stages (status, inspect) need
  to read it after the planning step completes.

## Decision 5: Handling Absent Canon Output

**Decision**: When Canon project-memory output is absent, set
`ProjectMemoryContext` to an explicit `absent` state and let Boundline
continue delivery using other available context. No synthetic Canon output.

**Rationale**: Constitution principle XVI requires Boundline to remain
independently testable. Inventing Canon output when none exists would violate
the invariant that Boundline must not redefine Canon semantics.

**Alternatives considered**:
- Block delivery when Canon output is missing: rejected because it would make
  Boundline unusable without Canon.
- Synthesize default Canon output: rejected because it would violate the
  contract boundary.
