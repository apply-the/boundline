# Research: Session-Native Workflow Layer

**Feature**: 018-workflow-layer  
**Date**: 2026-04-30

## R1: Use one local workflow-definition surface instead of a generic workflow DSL

**Decision**: Add one workspace-local named-workflow definition surface for the first slice and explicitly reject generic workflow-programming semantics such as loops, switches, fan-out, and fan-in.

**Rationale**: The delivery value comes from making Synod's existing phase sequence easier to start and resume, not from creating a general automation language. A bounded definition surface preserves the product's session-native identity and keeps the slice small enough to ship independently.

**Alternatives Considered**:
- Adopt a general-purpose workflow DSL immediately: rejected because it widens scope into generic orchestration rather than delivery ergonomics.
- Keep only manual session-native commands: rejected because it leaves repeatable workflows without a stable named entrypoint.

## R2: Make TOML the first definition format

**Decision**: Use one human-editable TOML workflow-definition file for the first slice and defer YAML to a later interoperability-oriented expansion.

**Rationale**: Synod already relies on TOML for configuration, the crate already ships the `toml` dependency, and TOML fits a bounded registry of named workflows without introducing a second primary configuration dialect.

**Alternatives Considered**:
- YAML first: rejected because it adds a second primary authoring surface before the feature needs interoperability.
- JSON first: rejected because it is less ergonomic for hand-authored local workflow definitions.

## R3: Compile workflows onto the existing session-native runtime instead of creating a second engine

**Decision**: Treat a named workflow as a thin layer that compiles onto existing session-native phases and runtime transitions.

**Rationale**: Synod already has the bounded control plane the workflow needs: planning, execution, governance, review, status, next, and inspect. Reusing that control plane keeps workflow behavior inspectable and avoids a second source of truth for execution.

**Alternatives Considered**:
- Create a dedicated workflow runtime: rejected because it duplicates session state, routing, and execution rules.
- Delegate workflow ownership to Canon: rejected because Canon is intentionally bounded to governance and evidence.

## R4: Persist workflow progress inside the existing session record

**Decision**: Store the active workflow name, current phase, and satisfied-phase progress inside `.synod/session.json`, while keeping workflow definitions themselves in a local workflow-definition file.

**Rationale**: Session persistence is already the authoritative runtime state for Synod. Keeping workflow progress there allows `status`, `next`, and `inspect` to reuse the existing session-projection model rather than building a parallel persistence surface.

**Alternatives Considered**:
- Persist workflow runs in a separate workflow-state file: rejected because it fragments runtime state.
- Recompute workflow progress purely from traces on every command: rejected because it makes progression less explicit and less resilient.

## R5: Keep the first command family minimal and operator-facing

**Decision**: The first CLI slice should support `workflow list`, `workflow run <name>`, `workflow status`, `workflow inspect`, and `workflow resume`.

**Rationale**: These commands are sufficient to define, start, continue, and inspect named workflows without introducing a large parallel CLI surface. They map cleanly to the existing session-native operator story.

**Alternatives Considered**:
- Add a broad authoring or editing command family immediately: rejected because authoring can begin with direct file edits and validation.
- Expose only `workflow run`: rejected because resumability and inspectability are part of the feature's core value.

## R6: Treat release hygiene as a first-class closeout requirement

**Decision**: The implementation plan will begin with the crate version bump to `0.18.0` and end with the full validation and release-alignment sequence: coverage-aware testing, clippy warning resolution, fmt, docs, roadmap, and changelog updates.

**Rationale**: The repository already treats each roadmap slice as a versioned delivery unit. Encoding release hygiene into the task plan reduces end-of-slice drift and matches the user's explicit requested ordering.

**Alternatives Considered**:
- Leave versioning and release surfaces until after implementation: rejected because it increases the chance of mismatched docs and release metadata.
- Treat coverage and lint validation as optional polish: rejected because Synod's workflow slices are expected to close with executable validation.