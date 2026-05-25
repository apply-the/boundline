# Implementation Plan: Delivery Control Consumer

**Branch**: `051-delivery-control-consumer` | **Date**: 2026-05-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/051-delivery-control-consumer/spec.md`

## Summary

Consume Canon-owned project-memory and delivery-control contracts through
Boundline's existing workflow, config, and runtime surfaces. The plan keeps the
session-native loop primary, reuses `.boundline/workflows.toml` for V1 delivery
paths, introduces `project.boundline.toml` as the repo-visible project semantics
index, distinguishes V1 hard stops from warnings, and surfaces Canon refs plus
consumer compatibility state without redefining Canon promotion policy.

## Technical Context

**Language/Version**: Rust 1.95.0, Edition 2024  
**Primary Dependencies**: `clap`, `dialoguer`, `serde`, `serde_json`,
`thiserror`, `tracing`, `uuid`, `toml`  
**Storage**: workspace-local `.boundline/session.json`, `.boundline/traces/`,
`.boundline/workflows.toml`, `.boundline/cluster.toml`, repo-visible
`project.boundline.toml`, plus Canon-promoted docs under `docs/project/` and
`docs/evidence/`  
**Testing**: `cargo test`, `cargo nextest run`, `cargo test --no-run
--all-targets`, `cargo fmt --check`, `cargo clippy --workspace --all-targets
--all-features -- -D warnings`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: multi-crate Rust workspace with CLI, adapters, and core
runtime modules  
**Execution Model**: sequential session-native loop with explicit bounded stops,
warnings, and replanning  
**Observability Surface**: persisted session state, traces, session-native
status/next/inspect output, Canon refs, consumer compatibility state, and
producer-attributed evidence blocks  
**Performance Goals**: no material CLI responsiveness regression while adding
project-memory and project-index reads to planning and inspection paths  
**Constraints**: do not add a second workflow registry; do not collapse
project semantics into cluster topology; do not redefine Canon promotion rules;
do not require provider-readiness or voting work for V1  
**Scale/Scope**: one new consumer slice that touches config and planning
surfaces, one new repo-visible project index, tiered stop behavior, and shared
managed-block consumption across Canon and Boundline evidence docs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Delivery identity**: PASS. The slice improves bounded engineering-task
  delivery by letting planning and inspection consume credible Canon knowledge
  without hidden fallbacks. (Spec: User Story 1, Requirements)
- **Delivery-first scope**: PASS. Execution and planning control come first;
  provider-runtime, voting, UI, and deployment work stay out of scope. (Spec:
  Scope Boundaries)
- **Primary workflow**: PASS. The main operator path remains session-native
  (`goal -> plan -> run -> status -> next -> inspect`). No new
  compatibility-first path is introduced. (Spec: User Story 1, User Story 3)
- **Bounded execution**: PASS. V1 hard stops and warnings are explicit, and the
  slice does not add background work or hidden loops. (Spec: FR-006, FR-007,
  FR-008)
- **Stateful execution**: PASS. Canon consumption flows through session state,
  project memory context, project index, and inspection surfaces rather than
  stateless one-off reads. (Spec: Key Entities, FR-001, FR-009)
- **Mutable planning**: PASS. Stable Canon knowledge can inform replanning,
  while warnings and hard stops keep non-credible inputs visible. (Spec: User
  Story 1)
- **Sequential-first design**: PASS. The slice reads and evaluates Canon state
  one step at a time inside the existing session-native loop. (Technical
  Context)
- **Tool-agent symmetry**: PASS. Canon refs, workflow entries, and stop states
  are explicit surfaces that guide visible execution decisions. (Spec: User
  Story 3, FR-009)
- **Observability and explicit intelligence**: PASS. The slice requires traces,
  inspection surfaces, compatibility state, and producer attribution to stay
  visible. (Technical Context, FR-005, FR-009)
- **Catalog currency**: PASS. Public provider docs were checked on 2026-05-13;
  no bundled catalog change is required for this feature. (Spec: Catalog
  Research & Currency)
- **Non-goals and external separation**: PASS. Canon remains an external
  producer boundary; Boundline consumes Canon facts but does not depend on Canon
  to own its control flow. Councils, voting, provider abstraction expansion,
  long-term memory, UI, and pipelines stay out of scope. (Spec: Scope
  Boundaries, FR-011)
- **Minimal slice**: PASS. The smallest independently valuable capability is
  reading Canon repo-visible knowledge and producing explicit continue, warn, or
  hard-stop outcomes without adding competing registries. (Spec: User Story 1,
  User Story 2)

## Project Structure

### Documentation (this feature)

```text
specs/051-delivery-control-consumer/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── canon-project-memory-consumer-contract.md
│   └── project-index-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
assistant/
└── catalog/
    └── model-catalog.toml
.boundline/
├── cluster.toml
└── workflows.toml
src/
├── domain/
│   └── project_memory.rs
├── orchestrator/
│   ├── session_runtime.rs
│   ├── goal_planner.rs
│   └── governance.rs
tests/
├── unit/
├── integration/
└── contract/
```

**Structure Decision**: Reuse the existing Boundline workspace layout and
session-native runtime surfaces. The new repo-visible project semantics live in
`project.boundline.toml`, while workflow and cluster state stay in their
existing files. No new crate or top-level runtime registry is introduced.

## Complexity Tracking

No constitution violations identified.
