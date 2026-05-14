# Research: Delivery Control Consumer

## Decision 1: Reuse `.boundline/workflows.toml` for V1 delivery paths

- **Decision**: Represent V1 delivery paths as higher-level entries inside the
  existing `.boundline/workflows.toml` registry.
- **Rationale**: A second registry file for delivery paths would overlap with
  the existing workflow surface before the consumer slice is even usable.
- **Alternatives considered**:
  - Introduce `.boundline/delivery-paths.toml`: rejected because it duplicates
    workflow responsibilities.
  - Keep delivery paths implicit: rejected because the consumer slice needs an
    inspectable stage map.

## Decision 2: Separate project semantics from cluster topology

- **Decision**: Use `project.boundline.toml` for repo-visible project
  semantics and keep `.boundline/cluster.toml` focused on workspace topology.
- **Rationale**: The control layer needs one surface for systems, domains,
  owners, and docs locations, and a separate surface for membership and primary
  workspace relationships.
- **Alternatives considered**:
  - Extend `cluster.toml` to carry product semantics: rejected because it blurs
    runtime topology and product meaning.
  - Avoid a project index entirely: rejected because delivery control needs a
    readable project map.

## Decision 3: Use tiered stop conditions

- **Decision**: Split consumer conditions into V1 hard stops, V1 warnings, and
  post-V1 hard-stop candidates.
- **Rationale**: Treating every missing or stale input as a hard stop would make
  the first consumer slice too brittle.
- **Alternatives considered**:
  - Make every issue a hard stop: rejected because it blocks incremental value.
  - Keep all conditions as warnings: rejected because blocked governance or
    missing required artifacts must stop execution explicitly.

## Decision 4: Keep Canon as the only source of truth for producer semantics

- **Decision**: Boundline carries a consumer note and version pin, not a second
  canonical contract.
- **Rationale**: The producer owns promotion semantics and compatible field
  evolution; duplicating the canonical contract in Boundline would invite drift.
- **Alternatives considered**:
  - Vendor a full canonical copy in Boundline: rejected because it creates two
    sources of truth.
  - Depend on Canon docs without any consumer note: rejected because Boundline
    still needs to declare what it reads and how it reacts to version changes.

## Decision 5: No model-catalog changes for this slice

- **Decision**: Keep the bundled assistant model catalog unchanged.
- **Rationale**: Public provider docs still match the text-and-coding model
  families already listed in the catalog, and newly exposed media or research
  models do not alter this delivery-control feature.
- **Alternatives considered**:
  - Refresh the catalog proactively with every newly exposed provider model:
    rejected because it would expand this slice beyond its delivery-control scope.