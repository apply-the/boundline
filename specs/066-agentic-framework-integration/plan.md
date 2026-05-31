# Implementation Plan: Agentic Framework Integration

**Branch**: `066-agentic-framework-integration` | **Date**: 2026-05-31 | **Spec**: [specs/066-agentic-framework-integration/spec.md](specs/066-agentic-framework-integration/spec.md)

**Input**: Feature specification from `specs/066-agentic-framework-integration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Keep Boundline's Canon-aware behavior as the built-in default lifecycle path,
then add one explicit external framework-adapter slot per lifecycle run so an
operator can register `speckit` or a custom trusted local subprocess without
breaking the no-adapter experience. Boundline owns the host runtime, workspace
config, known-profile registry, lifecycle routing, audit output, and operator
surfaces; the sibling `../boundline-framework-template/` repository is the
reusable Rust scaffold for third-party adapters; and the sibling
`../boundline-adapter-speckit/` repository implements the Speckit profile and
ships the `boundline-adapter-speckit` binary.

The initial slice stays intentionally bounded: one active adapter or none,
sequential stage execution, one-shot JSON stdin/stdout subprocess calls, a
standard host-visible success/error response envelope, and explicit fallback
boundaries. Boundline performs adapter discovery and config preflight before
adapter-owned stage execution begins, requires `describe` to declare supported
transports so V1 explicitly advertises JSON over stdin/stdout while leaving
room for future transports, falls back to built-in behavior when no adapter is
selected or preflight fails before ownership is claimed, and marks the current
stage failed with required operator intervention when an adapter fails after
claiming a stage. Optional structured stderr may be ingested into traces when
adapters emit it, but graceful shutdown and other long-running transport
lifecycle concerns remain out of scope because V1 stays on one-shot
subprocesses. Known-profile activation remains explicit through
`boundline adapter add speckit`; PATH discovery may assist setup, but it must
never auto-enable the adapter. When a declared `plan` or `run` stage passes
preflight, the adapter becomes the authoritative execution path for that stage:
Boundline may assemble host-owned context first, but it must not complete the
built-in stage result before the adapter returns. Successful adapter responses
become the persisted stage outcomes; blocked responses leave the stage blocked
and incomplete; post-claim failures stop the lifecycle. The template repository may
remain a generic scaffold, but the Speckit repository must bridge real Speckit
workflow execution and return real produced artifacts or actionable blocked and
failure outcomes.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024 across the Boundline workspace, the sibling template repo, and the sibling Speckit adapter repo for the initial compatibility line

**Primary Dependencies**: existing workspace crates and dependencies (`clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, `boundline-core`, `boundline-adapters`, `boundline-cli`) plus a shared framework-adapter protocol surface owned by `boundline-adapters` and consumed by sibling repos through versioned git-tag dependencies rather than committed path-based copies

**Storage**: workspace-local `.boundline/config.toml`, `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, and `.boundline/workflows.toml`, extended with an optional adapter selection block and adapter audit fields, while the sibling template and Speckit repos persist only their own Cargo manifests, README docs, and protocol fixtures

**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
focused unit tests for config parsing, known-profile lookup, preflight, and
stage-routing decisions, contract tests for CLI JSON and adapter wire payloads
including standard success or error envelope classification, `describe`
transport-declaration enforcement, and best-effort structured stderr handling,
temp-workspace integration tests covering default no-adapter execution,
`boundline adapter add speckit`, custom adapter registration, interactive
required-field collection, non-interactive missing-config failure,
unsupported-transport blocking before stage claim, and post-claim stage-failure
stop semantics, plus authoritative-routing tests that prove built-in `plan` and
`run` completion is not treated as the completed stage outcome once an adapter
claims those stages, and cross-repo compile plus end-to-end tests for the
template and Speckit repos against the released protocol line

**Target Platform**: macOS and Linux developer workstations, plus Linux CI,
with trusted local adapter binaries discovered on `PATH` or configured through
an explicit command/path override

**Project Type**: multi-repo Rust CLI and library workspace in this repository plus one reusable sibling adapter-template repo and one sibling Speckit adapter-binary repo

**Execution Model**: sequential one-run/one-adapter local subprocess model.
Boundline loads adapter capabilities once per lifecycle run, validates required
config before adapter-owned stages begin, requires `describe` to declare
supported transports, invokes every one-shot subprocess command over JSON
stdin/stdout using the same standard host-visible success/error response
envelope with command-specific outcomes kept inside `data`, records ownership
per stage and hook, may ingest optional structured stderr diagnostics into
traces without letting stderr change result classification, routes declared
`plan` and `run` stages to the adapter as the authoritative stage path after
successful preflight, persists the adapter response as the stage outcome, and
never starts background daemons, hidden retries, or concurrent adapters

**Observability Surface**: `boundline adapter add|show|remove`, `boundline init`,
`boundline config show`, `boundline goal|plan|run|status|inspect`,
`.boundline/config.toml`, `.boundline/session.json`, `.boundline/traces/`, and
adapter audit records that expose execution source, capability compatibility,
hook delivery, optional structured adapter diagnostics when emitted, and
intervention-required failure reasons

**Performance Goals**: preserve the current no-adapter path with effectively no
behavioral regression, keep capability discovery and preflight to one bounded
exchange per lifecycle run, and keep adapter-owned stage overhead proportional
to the number of declared stages and hooks rather than a long-lived daemon cost

**Constraints**: built-in Canon-aware behavior remains the default when no
adapter is selected; exactly one active adapter is allowed per lifecycle run;
explicit operator selection is the only activation path; non-interactive runs
with missing adapter config must fail before adapter execution begins; stage
ownership cannot silently revert mid-stage; external adapters remain trusted
local subprocesses only; no MCP core dependency is introduced; committed
cross-repo path dependencies are forbidden; V1 accepts only a declared JSON
over stdin/stdout transport; graceful shutdown for persistent transports is
deferred; the known Speckit profile cannot satisfy this slice with placeholder
claimed-stage markers alone; and the sibling template repo must be bootstrapped
from its currently empty Git state

**Scale/Scope**: one Boundline host repo, one empty adapter-template repo to
bootstrap, one known Speckit adapter repo with an initial commit, one shipped
known profile (`speckit`) plus one custom-adapter path, one authoritative
workspace-local adapter selection, and a bounded lifecycle stage/hook catalog
for the existing Boundline delivery flow

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: Boundline remains the delivery orchestrator and
  keeps Canon-aware behavior as the built-in default when no adapter is
  selected. External adapters extend bounded delivery stages instead of
  redefining the product entrypoint. See Summary, `research.md` Decisions 1-3,
  and `contracts/adapter-management-cli-contract.md`.
- **PASS** Delivery-first scope: The slice exists to improve delivery execution
  for repos that want framework-specific behavior while preserving the default
  lifecycle for everyone else. It does not introduce a generic plugin or chat
  framework. See Summary, Technical Context, and `research.md` Decisions 1, 2,
  and 5.
- **PASS** No abstract agent systems: The design is limited to concrete
  lifecycle stages, declared hooks, explicit operator setup, and bounded
  subprocess calls. No multi-agent ecosystem or autonomous background network is
  introduced. See `research.md` Decisions 2 and 3.
- **PASS** Bounded execution: The slice supports one adapter or none, one-shot
  subprocess commands, explicit preflight, explicit stop semantics after a
  claimed-stage failure, and no unbounded retries or daemons. See Technical
  Context, `data-model.md`, and
  `contracts/framework-adapter-stdio-contract.md`.
- **PASS** Stateful execution: Adapter selection, resolved config values,
  capability snapshots, stage ownership, and hook outcomes are persisted through
  workspace config, session state, and traces rather than transient prompt-only
  state. See `data-model.md` and `quickstart.md`.
- **PASS** Mutable planning: Boundline still owns plan mutation and execution
  state. Adapters receive explicit context and return explicit outcomes instead
  of introducing opaque self-modifying control flow. See Summary,
  `data-model.md`, and `contracts/framework-adapter-stdio-contract.md`.
- **PASS** Sequential-first design: The initial slice forbids concurrent
  adapters and background workers. One lifecycle run has one active adapter and
  one active stage at a time. See Technical Context and `research.md` Decision 3.
- **PASS** Tool-agent symmetry: The source of truth is a host-owned CLI and
  typed protocol contract that humans, assistant packages, and external adapter
  repos all consume. See `contracts/adapter-management-cli-contract.md` and
  `contracts/framework-adapter-stdio-contract.md`.
- **PASS** Required observability and no hidden intelligence: Operator-visible
  surfaces expose when built-in behavior or adapter behavior ran, which stages
  were claimed, which hooks were delivered, and why fallback or intervention was
  required. See Technical Context, `data-model.md`, and `quickstart.md`.
- **PASS** Failure as a first-class path: The design distinguishes no adapter
  selected, invalid capability manifest, missing config, unavailable binary,
  preflight block, undeclared-stage requests, and post-claim stage failures.
  See `research.md` Decision 6, `data-model.md`, and
  `contracts/framework-adapter-stdio-contract.md`.
- **PASS** Separation from external systems: Boundline remains independently
  usable without the template repo, the Speckit adapter repo, or any custom
  adapter binary. External repos consume a host-owned contract rather than
  becoming the default control plane. See Summary, Constraints, and
  `contracts/known-profile-versioning-contract.md`.
- **PASS** Minimal capability slice: The smallest useful slice is one explicit
  adapter selection path, one trusted subprocess protocol, one known Speckit
  profile, and one reusable template bootstrap. Multi-adapter coordination and
  broader plugin ecosystems stay out of scope. See `research.md` Decisions 1-5.
- **PASS** Catalog currency: Public provider docs were rechecked on 2026-05-31.
  The bundled catalog already matches the currently documented OpenAI, Anthropic,
  and Gemini model families relevant to this repo, so no catalog delta is
  required for this adapter-planning slice. See `research.md` Provider Catalog
  Refresh.

## Project Structure

### Documentation

```text
specs/066-agentic-framework-integration/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── adapter-management-cli-contract.md
│   ├── framework-adapter-stdio-contract.md
│   └── known-profile-versioning-contract.md
└── tasks.md
```

### Source Code

```text
Cargo.toml
assistant/
└── catalog/
    └── model-catalog.toml

src/
├── adapters/
│   ├── config_store.rs
│   └── governance_runtime.rs
├── cli/
│   ├── init/
│   ├── config.rs
│   ├── init.rs
│   ├── orchestrate.rs
│   ├── output_runtime.rs
│   └── output_session_status.rs
├── domain/
│   ├── configuration.rs
│   ├── execution.rs
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   ├── engine.rs
│   ├── session_runtime.rs
│   ├── session_runtime_execution_core.rs
│   ├── session_runtime_native_goal_plan.rs
│   └── session_runtime_surface.rs
└── registry/

crates/
├── boundline-core/src/
├── boundline-adapters/src/
└── boundline-cli/src/

tests/
├── contract/
├── integration/
└── unit/

../boundline-framework-template/
├── Cargo.toml            # bootstrap target for the reusable adapter scaffold
├── src/
├── tests/
└── README.md

../boundline-adapter-speckit/
├── Cargo.toml
├── src/
├── tests/
└── README.md
```

**Structure Decision**: Keep the Boundline host, workspace config, lifecycle
router, audit output, and known-profile registry in this repository; place the
shared protocol models and fixture helpers in `crates/boundline-adapters`; use
`../boundline-framework-template/` as the reusable bootstrap target for new
external adapters; and keep all Speckit-specific behavior, tests, and release
docs in `../boundline-adapter-speckit/`. Do not copy the template or Speckit
implementation into the Boundline tree.

## Complexity Tracking

No constitution violations are expected for this slice.
