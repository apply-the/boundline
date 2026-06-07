# Boundline Project Structure Proposal

## Purpose

This document compares a generic Generative AI project structure with the structure Boundline should follow.

The reference structure is useful as an inspiration for separation of concerns, but it should not be copied directly. Boundline is not a generic GenAI application, chatbot, RAG prototype, or inference pipeline. Boundline is a governed AI-assisted delivery runtime.

The right question is not:

```text
Where do prompts, embeddings, and inference live?
```

The right question is:

```text
Where do runtime state, governance rules, provider claims, contracts, traces, and operator decisions live?
```

## Core Position

Boundline should not be structured like a typical Python GenAI project.

A generic GenAI project usually centers on:

```text
config/
data/
embeddings/
vectordb/
prompts/
rag/
inference/
```

Boundline should center on:

```text
domain/
orchestrator/
config/
registry/
cli/
contracts/
trace/
tests/
docs/
```

The reference structure is useful for its discipline, not for its exact folders.

## What To Take From The Generic GenAI Structure

The generic structure provides one good lesson: separate concerns clearly.

Useful principles:

1. Keep configuration separate from runtime code.
2. Keep generated state separate from source code.
3. Keep reusable runtime logic separate from CLI or UI entry points.
4. Keep documentation close to the project.
5. Keep scripts explicit and repeatable.
6. Avoid mixing provider configuration, runtime state, and business logic.
7. Make generated or derived data clearly disposable and non-authoritative.

These principles are valuable for Boundline.

The actual folder names and responsibilities should be different.

## What Not To Copy

Boundline should not copy these folders as first-class architectural centers:

```text
data/cache/
data/embeddings/
data/vectordb/
src/prompts/
src/rag/
src/inference/
```

These make sense for a RAG application or LLM pipeline, but they would distort Boundline.

Why:

- Boundline should not become an LLM wrapper.
- Boundline should not make prompts the primary source of behavior.
- Boundline should not make vector indexes authoritative.
- Boundline should not treat provider output as truth.
- Boundline should not hide governance logic inside inference orchestration.
- Boundline should not introduce a generic inference engine as the core abstraction.

Provider execution belongs behind protocols and adapters. Governance decisions belong in Boundline-owned runtime logic.

## Proposed Boundline Structure

A healthy Boundline structure should look closer to this:

```text
boundline/
├── .boundline/
│   ├── config.toml
│   ├── guardian-rules.toml
│   ├── calibration-policy.toml
│   ├── refinement-profiles.toml
│   ├── help-links.toml
│   ├── session.json
│   └── traces/
│
├── src/
│   ├── domain/
│   │   ├── trace.rs
│   │   ├── observability.rs
│   │   ├── governance.rs
│   │   ├── providers.rs
│   │   ├── council.rs
│   │   ├── calibration.rs
│   │   └── refinement.rs
│   │
│   ├── orchestrator/
│   │   ├── plan.rs
│   │   ├── run.rs
│   │   ├── provider_execution.rs
│   │   ├── council.rs
│   │   ├── calibration.rs
│   │   └── refinement.rs
│   │
│   ├── registry/
│   │   ├── agent_registry.rs
│   │   ├── provider_registry.rs
│   │   └── adapter_registry.rs
│   │
│   ├── config/
│   │   ├── workspace_config.rs
│   │   ├── guardian_rules.rs
│   │   ├── calibration_policy.rs
│   │   ├── refinement_profiles.rs
│   │   └── help_links.rs
│   │
│   ├── cli/
│   │   ├── plan_cmd.rs
│   │   ├── run_cmd.rs
│   │   ├── status_cmd.rs
│   │   ├── next_cmd.rs
│   │   ├── inspect_cmd.rs
│   │   ├── provider_cmd.rs
│   │   ├── council_cmd.rs
│   │   └── refinement_cmd.rs
│   │
│   └── contracts/
│       └── mod.rs
│
├── crates/
│   ├── boundline-core/
│   ├── boundline-cli/
│   └── boundline-adapters/
│
├── specs/
├── docs/
├── tech-docs/
├── roadmap/
├── tests/
│   ├── unit/
│   ├── contract/
│   └── integration/
│
└── scripts/
```

This layout keeps the core product identity clear:

```text
Boundline is a governed runtime, not a prompt collection.
```

## Responsibility Boundaries

### `.boundline/`

Workspace-local configuration and runtime state.

This is Boundline's equivalent of a config and local state area.

It may contain:

```text
config.toml
guardian-rules.toml
calibration-policy.toml
refinement-profiles.toml
help-links.toml
session.json
traces/
```

Rules:

- Configuration files must be versioned when they are contract surfaces.
- Runtime state must be inspectable.
- Derived files must be disposable or clearly marked as derived.
- Nothing in `.boundline/` should silently become a source of hidden authority.

### `src/domain/`

Strong domain types.

This is where Boundline should define the language of the runtime.

Examples:

```text
RoundPacket
StopReason
RefinementProfile
CalibrationPolicy
GuardianRule
CouncilOutcome
ProviderExecutionResult
TraceEvent
EvidenceRef
```

Rules:

- Domain types should be explicit and serializable where needed.
- Enums should replace stringly typed runtime decisions.
- Validation belongs close to the domain model.
- No prompt-specific logic should live here.

### `src/orchestrator/`

Runtime movement.

This is where Boundline coordinates stages, providers, guardians, councils, and refinement loops.

Examples:

```text
planning execution
provider-backed stage execution
guardian activation
council adjudication
adaptive calibration
recursive refinement
trace emission
stop semantics
```

Rules:

- Orchestrators own runtime transitions.
- Providers may propose outputs, but Boundline validates and decides.
- Stop, degradation, escalation, and failure behavior must be explicit.
- No open-ended agent autonomy.

### `src/registry/`

Lookup and resolution.

This area owns provider, adapter, and agent registration or resolution.

Examples:

```text
provider id -> provider handle
adapter id -> adapter command
guardian id -> guardian definition
profile id -> adapter profile
```

Rules:

- Registration and activation must be explicit.
- Runtime execution must not depend on hidden provider discovery.
- Provider output is a claim, not truth.

### `src/config/`

Typed configuration loading.

This area turns `.boundline/*.toml` into validated runtime structures.

Examples:

```text
guardian-rules.toml -> GuardianRules
calibration-policy.toml -> CalibrationPolicy
refinement-profiles.toml -> RefinementProfile
help-links.toml -> HelpLinks
```

Rules:

- Invalid governance-relevant config should fail closed.
- Defaults must be explicit and trace-visible when they affect behavior.
- CLI overrides should be clearly recorded when they affect runtime behavior.

### `src/cli/`

Operator surfaces.

This area owns commands and rendering.

Examples:

```text
boundline plan
boundline run
boundline status
boundline next
boundline inspect
boundline provider
boundline council
boundline trace
```

Rules:

- CLI code should not own core decisions.
- CLI should call domain/orchestrator/config services.
- Human-readable output and JSON output should share the same underlying projection.

### `specs/`

Feature truth and delivery planning.

This is where feature specifications, plans, contracts, data models, research, quickstarts, and tasks live.

Rules:

- `spec.md` is the normative feature source.
- Contracts should be explicit when output shape matters.
- Roadmap seeds copied into specs are historical context, not the normative implementation source.
- Tasks should map to requirements and acceptance criteria.

### `tests/`

Verification by layer.

Recommended split:

```text
tests/unit/
tests/contract/
tests/integration/
```

Rules:

- Unit tests verify domain logic.
- Contract tests verify serialized shape, CLI output, JSONL export, config formats, and trace payloads.
- Integration tests verify complete runtime flows.
- Failure paths must be tested, not only happy paths.

### `scripts/`

Repeatable quality and release tasks.

Examples:

```text
clippy.sh
test.sh
coverage.sh
update-docs-versions.sh
sync-distribution-metadata.sh
validate-assistant-plugins.sh
check-no-local-paths.sh
check-rust-no-panic.sh
```

Rules:

- Scripts should be the same commands used locally and in CI.
- Quality gates should be explicit.
- Release metadata should be synchronized from the canonical version source.

## Mapping From Generic GenAI Structure To Boundline

| Generic GenAI Area | Boundline Equivalent | Keep? | Notes |
|---|---|---:|---|
| `config/` | `.boundline/*.toml` + `src/config/` | Yes | Use typed, versioned, validated config. |
| `data/cache/` | `.boundline/traces/`, session state, derived indexes | Partly | Only if state is inspectable and authority is clear. |
| `data/embeddings/` | Derived retrieval substrate | Rarely | Not central. Must remain disposable and non-authoritative. |
| `data/vectordb/` | Derived local retrieval index | Rarely | Never use as hidden-state authority or communication bus. |
| `src/core/` | `src/domain/` + `src/orchestrator/` | Yes | Split domain types from runtime movement. |
| `src/prompts/` | Provider/adapter contracts, not core behavior | Mostly no | Prompts should not be the product architecture. |
| `src/rag/` | Context substrate or provider capability | Not core | Keep behind provider/context features. |
| `src/inference/` | Provider protocol and execution adapters | Reframe | Do not create a generic inference engine as the core. |
| `docs/` | `docs/`, `tech-docs/`, `specs/` | Yes | Keep docs, specs, and contracts clear. |
| `scripts/` | `scripts/` | Yes | Strongly keep. |

## Architectural Rules For Boundline

### Rule 1: Domain Before Prompt

Every important runtime behavior should have a domain type.

Bad:

```text
prompt text decides whether the run is blocked
```

Good:

```text
GuardianFinding
CouncilOutcome
StopReason
RefinementOutcome
```

### Rule 2: Provider Output Is A Claim

Providers can generate candidates, critiques, summaries, and suggested deltas.

Boundline must decide whether those outputs are accepted.

```text
provider output -> claim
runtime validation -> accepted/rejected/degraded outcome
trace -> evidence
```

### Rule 3: Trace Is The Runtime Memory

Boundline should not rely on hidden state or long prompt transcripts as the primary memory.

Trace records should carry:

```text
event type
schema version
artifact refs
evidence refs
provider claim refs
decision reason
stop reason
operator-visible projection
```

### Rule 4: Config Must Be Inspectable

Configuration that affects governance must be visible and explainable.

Examples:

```text
why did this guardian activate?
why is this control advisory?
why did refinement start?
why did it stop?
which provider was used?
which CLI override changed behavior?
```

### Rule 5: Derived Retrieval Is Not Authority

A local retrieval index can help find context.

It must not become:

```text
a hidden-state store
an agent communication bus
an authoritative memory
a replacement for trace evidence
```

### Rule 6: Contracts Beat Comments

When an output is consumed by another command, dashboard, CI job, or agent, define a contract.

Examples:

```text
round-packet-schema.md
inspect-output.md
refinement-profile-config.md
provider-response-schema.md
jsonl-export-schema.md
```

### Rule 7: Separate Runtime Decisions From Presentation

The CLI should render decisions. It should not own decisions.

```text
domain/orchestrator -> decision
cli -> projection
```

### Rule 8: Keep Feature Scope Narrow

Each feature should own one runtime concern.

Examples:

```text
provider protocol owns provider activation and execution
review councils owns adjudication
adaptive calibration owns control levels
recursive refinement owns bounded stage movement
evals/observability owns event and metric substrate
```

Do not let one feature absorb adjacent systems.

## Recommended Direction

Boundline should continue evolving toward this structure:

```text
governed runtime
typed contracts
workspace-local policy config
trace-first observability
provider claims validation
operator-visible state
strong tests
repeatable scripts
```

It should avoid evolving toward:

```text
prompt script collection
generic LLM framework
RAG-first application
hidden agent memory
inference engine wrapper
embedding-centered architecture
```

## Practical Checklist For Future Features

Before adding a new folder or abstraction, ask:

1. Is this a runtime decision or only a provider capability?
2. Does this need a domain type?
3. Does this need a config contract?
4. Does this need a trace event?
5. Does this need a CLI projection?
6. Does this need a JSON or JSONL contract?
7. Is the state authoritative or derived?
8. Can the operator inspect why it happened?
9. Does this duplicate council, calibration, provider protocol, route economics, or trace ownership?
10. Can it be tested without a specific model or vector database?

If the answer depends on prompts or hidden model behavior, the abstraction is probably in the wrong place.

## Final Recommendation

Use the generic Generative AI structure as an organizational inspiration only.

Do not copy its folders.

For Boundline, the stronger structure is:

```text
.boundline/      workspace config and runtime state
src/domain/     strong runtime vocabulary
src/orchestrator/ runtime movement and decisions
src/registry/   provider and adapter resolution
src/config/     typed config loading
src/cli/        operator commands and projections
specs/          feature truth and contracts
tests/          unit, contract, integration proof
scripts/        repeatable quality and release gates
docs/           operator and technical documentation
roadmap/        future direction, not active feature truth
```

This keeps Boundline aligned with its real product identity: a governed AI-assisted delivery runtime.
