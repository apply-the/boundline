# Research: Domain Agent Templates

## Decision 1: Represent the first-party domain catalog as built-in template definitions plus scoped overlays

- **Decision**: Add one built-in domain-template catalog inside Synod's Rust
  domain model and layer operator-managed standards on top of it through the
  existing scoped configuration surfaces.
- **Rationale**: The feature needs one authoritative product-owned catalog for
  the supported language and framework families, but it also needs reusable
  organization standards and local repository overrides. Extending the current
  scoped config model keeps the behavior inspectable and avoids introducing a
  separate prompt-marketplace or file-discovery runtime.
- **Alternatives considered**:
  - Store every template as independent external files discovered at runtime.
    Rejected because the initial slice needs predictable first-party coverage,
    simple precedence, and no extra storage or discovery layer.
  - Keep all specialization implicit in goal-planner heuristics. Rejected
    because the feature specifically requires explicit standards, overrides,
    and source attribution.

## Decision 2: Store per-family enablement, standards, and external bindings in the existing config hierarchy

- **Decision**: Extend the current workspace, cluster, and global config
  hierarchy with per-family domain-template settings that carry enablement,
  scoped standards text, and bound external context inputs.
- **Rationale**: Synod already resolves configuration across workspace,
  cluster, and global scopes, and `config show` already explains effective
  source attribution. Reusing that hierarchy makes shared standards and local
  overrides inspectable without inventing a second persistence model.
- **Alternatives considered**:
  - Create a dedicated `.synod/agents/` storage subsystem. Rejected for this
    slice because it would add a second configuration plane before the product
    has proven the core domain-selection behavior.
  - Limit domain settings to workspace scope only. Rejected because the feature
    needs one reusable cross-workspace baseline for organization standards.

## Decision 3: Build applied domain context during bounded context assembly and let planning fail when it is not credible

- **Decision**: Resolve the effective domain-template settings while building
  the existing context pack, select the matching domain family or families from
  workspace evidence and bounded task targets, and mark the context pack as
  insufficient when no credible selection or required supporting input exists.
- **Rationale**: The goal planner already gates planning on context-pack
  credibility, so domain-template failures should reuse that explicit stop path
  instead of creating a second planning gate. This keeps domain selection part
  of one bounded context story.
- **Alternatives considered**:
  - Apply domain selection only at execution time. Rejected because the feature
    must stop or explain domain mismatches before a run begins.
  - Store domain context only in read-side output. Rejected because planning,
    replanning, and execution traces all need the same authoritative context.

## Decision 4: Treat Canon-governed artifacts and external context bindings as supporting inputs, not owners of template selection

- **Decision**: Let Canon-governed artifacts and bound external inputs augment
  the applied domain context as optional or required evidence, but keep Synod
  responsible for domain-family selection, precedence, and terminal behavior.
- **Rationale**: Canon and external sources such as design references, design
  systems, token sources, or platform contracts improve bounded delivery only
  when they remain visible supporting inputs rather than hidden control-flow
  owners. This preserves Synod's product identity and constitution.
- **Alternatives considered**:
  - Make Canon or external context providers decide the active template.
    Rejected because that would violate external-system separation.
  - Ignore governed or external context entirely. Rejected because the feature
    explicitly needs those inputs to shape domain-sensitive tasks.

## Decision 5: Surface domain selection through the existing init, config, plan, run, status, next, and inspect surfaces

- **Decision**: Extend existing CLI command families to seed domain settings,
  mutate them after initialization, and project the applied domain context
  through the same read-side surfaces that already explain routing and bounded
  planning state.
- **Rationale**: The feature is only credible if operators can see the active
  families, the winning standards layer, and the status of supporting inputs
  without opening raw config files or traces by hand.
- **Alternatives considered**:
  - Add a separate domain-template CLI surface unrelated to current config and
    inspection commands. Rejected because it would fragment operator-facing
    control flow.
  - Hide domain details inside debug-only traces. Rejected because the feature
    must be inspectable from normal product surfaces.

## Decision 6: Ship the feature as a release-aligned 0.38.0 macrofeature

- **Decision**: Treat `0.38.0` closeout as part of the slice, including the
  version bump, roadmap activation and closure, docs plus assistant guidance
  updates, clean formatting and lint, and >95% line coverage for modified Rust
  files.
- **Rationale**: Domain agent templates alter the operator-visible planning and
  configuration model, so the product story must ship coherently rather than as
  a hidden internal foundation.
- **Alternatives considered**:
  - Defer docs, roadmap, and release closure until a later cleanup. Rejected
    because the user requested one feature-complete macrofeature with release
    validation built in.