# Data Model: Delivery Control Consumer

## ProjectIndex

- **Purpose**: Repo-visible project semantics document at
  `project.boundline.toml`.
- **Fields**:
  - `project.name`
  - `project.primary_domains[]`
  - `docs.project_memory`
  - `docs.evidence`
  - `systems.*.workspace`
  - `systems.*.paths[]`
  - `systems.*.owner`
  - `systems.*.domain`
  - `systems.*.criticality`
- **Constraints**:
  - may reference workspace IDs from `.boundline/cluster.toml`
  - must not become the topology source of truth

## DeliveryPathEntry

- **Purpose**: Higher-level stage sequence represented inside the existing
  `.boundline/workflows.toml` registry.
- **Fields**:
  - `name`
  - `description`
  - `stages[]`
  - `adaptive`
- **Constraints**:
  - one-step-at-a-time execution remains the default
  - path entries describe stage intent, not an unbounded script

## ProjectMemoryContext

- **Purpose**: Boundline-side view of Canon repo-visible project memory,
  evidence, refs, and compatibility status.
- **Fields**:
  - `project_memory_refs[]`
  - `evidence_refs[]`
  - `compatibility_state`
  - `effective_credibility`
  - `warning_conditions[]`
  - `hard_stop_condition`

## EvidenceContribution

- **Purpose**: Producer-attributed evidence summary inside `docs/evidence/`.
- **Fields**:
  - `producer`
  - `source_ref`
  - `target`
  - `status`
  - `summary`

## ConsumerCompatibilityState

- **Purpose**: Boundline-owned result of evaluating whether Canon output is safe
  to consume.
- **States**:
  - `compatible`
  - `warning`
  - `unsupported`

### Classification Matrix

- `compatible`: stable Canon inputs on the supported V1 contract line and a
  supported active delivery stage
- `warning`: stale or partial Canon knowledge, incomplete project index, or
  missing optional evidence when the current or replanned stage remains
  credible
- `unsupported`: unknown major contract line, unsupported stage or mode,
  blocked governance, missing required approval, exhausted validation, or
  missing required source artifact

## StopCondition

- **Purpose**: Explicit continuation gate used by Boundline when Canon facts or
  delivery-control inputs are insufficient.
- **V1 hard-stop examples**:
  - insufficient context
  - blocked Canon governance
  - missing required approval
  - exhausted validation
  - unsupported stage or mode
  - missing required source artifact
- **V1 warning examples**:
  - stale project memory
  - missing evidence source
  - incomplete project index
  - unknown assurance profile

## FixtureBundle

- **Purpose**: Deterministic test fixture set for delivery-control planning and
  inspection behavior.
- **Required contents**:
  - `project.boundline.toml`
  - workflow registry file with `delivery_paths`
  - Canon-managed `docs/project/` fixtures
  - mixed-producer `docs/evidence/` fixtures
  - sidecars or inline metadata needed to classify continue, warning, or
    hard-stop outcomes