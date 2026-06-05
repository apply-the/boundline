# Implementation Plan: External Capability Provider Protocol

**Branch**: `071-capability-provider-protocol` | **Date**: 2026-06-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/071-capability-provider-protocol/spec.md`

## Summary

Add a Boundline-owned external capability provider protocol that separates
discoverability, operator registration, activation, health, preparation,
bounded execution, and evidence collection from both Canon governance and the
existing framework-adapter and model-provider surfaces. The first slice should
introduce typed provider registrations, explicit least-privilege permission
envelopes, explicit setup-requirement projection, fail-closed admission
checks, structured read-only execution and evidence contracts, additive runtime
projections, and thin operator-facing CLI surfaces for registration and health.
Close the slice as Boundline `0.72.0`, keep Canon compatibility guidance at
`0.67.0`, record the 2026-06-05 provider-catalog no-change audit against
official OpenAI, Anthropic, and Google model docs, and require at least 95%
changed-file coverage for touched Rust implementation files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only;
`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`,
`reqwest`, `rusqlite`, `dialoguer`, `boundline-core`, `boundline-adapters`,
and `boundline-cli`; reuse the existing request and auth-storage primitives
already present in the workspace rather than adding a second transport stack

**Storage**: Existing workspace-local `.boundline/config.toml`,
`.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`,
and auth-profile storage, extended additively with provider registration,
activation, health, permission, and evidence projection state only; raw secrets
must remain outside tracked config and traces

**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets
--all-features -- -D warnings`, focused `cargo test --test unit`,
`cargo test --test contract`, provider-focused integration tests,
`cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`,
and `scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime and CI on supported developer
workstations; operator-facing host surfaces for Copilot, Claude, Codex, and
Antigravity remain thin projections over the same runtime-owned provider
contract

**Project Type**: Rust workspace with CLI, runtime, transport adapters,
assistant assets, distribution metadata, and repository-managed documentation

**Performance Goals**: Block 100% of provider-backed execution attempts when
the selected provider is unregistered, inactive, unhealthy, or incompatible;
keep health and prepare admission checks bounded to one deterministic request
per execution attempt; let operators identify provider readiness, capability,
evidence refs, and failure class from `status` or `inspect` within 30 seconds
for at least 95% of maintained fixture scenarios

**Constraints**: Provider output is never truth; no auto-activation from
discovery; no raw secrets in traces or tracked files; Canon is not a provider
or adapter; specialized execution profiles cannot bypass generic protocol
permissions or final acceptance; preserve fail-closed behavior on metadata
conflicts; avoid running `boundline` CLI commands against the repository root;
all changed Rust implementation files must remain above 95% changed-file
coverage

**Scale/Scope**: One workspace may hold multiple registered providers, but any
single provider-backed execution attempt resolves one activated provider and
one capability at admission time; the slice includes generic protocol records,
registration and activation surfaces, bounded execution and evidence
collection, additive runtime and assistant projections, docs, and release
closure for Boundline `0.72.0`

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | The slice improves real engineering delivery by making external capabilities usable through bounded, inspectable execution instead of ad hoc hidden integrations. |
| Bounded execution | PASS | Every provider lifecycle step is explicit and bounded: registration, health, prepare, execute, and collect-evidence each run once per request with fail-closed admission. |
| Stateful execution | PASS | Provider registration, activation, health, evidence, and validation disposition are persisted in Boundline-owned config, session, and trace state. |
| Mutable planning and execution over perfect planning | PASS | The protocol supports pre-execution repair through prepare or health failures without inventing automatic provider fixes or hidden retries. |
| Sequential-first design | PASS | One provider-backed request is admitted and validated at a time; no concurrent provider fan-out or background lifecycle manager is introduced. |
| Tool-agent symmetry and required observability | PASS | The same runtime contract feeds CLI, host JSON, status, inspect, trace, and assistant surfaces with explicit provider identity, failure class, permissions, and evidence. |
| No hidden intelligence | PASS | Providers return claims, artifacts, evidence, limitations, and patch proposals, but Boundline keeps validation and acceptance explicit and traceable. |
| Strict non-goals and minimal capability slice | PASS | Concrete providers, route economics, provider-specific UI polish, and full sandbox enforcement stay out of scope for this slice. |
| Real acceptance criteria and failure-first behavior | PASS | The quickstart and tasks will cover registration without activation, interrupted setup, unavailable providers, invalid permission requests, metadata conflicts, and post-execution validation failure. |
| Separation from external systems | PASS | Canon remains a governed producer outside provider activation semantics; provider-backed execution remains independently testable from local fixtures. |
| Catalog currency | PASS | `research.md` records a 2026-06-05 official-provider audit and explicit no-change rationale for the bundled model catalog. |
| Rust language rules | PASS | The design stays additive, typed, zero-panic outside `main.rs`, and modular, with explicit helper extraction rather than a new monolithic runtime function. |

## Project Structure

### Documentation (this feature)

```text
specs/071-capability-provider-protocol/
├── spec.md
├── feat-external-capability-provider-protocol.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── external-capability-provider-runtime-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── capability_provider.rs
│   ├── configuration.rs
│   ├── framework_adapter.rs
│   ├── session.rs
│   └── trace.rs
├── adapters/
│   ├── auth_profile_store.rs
│   ├── config_store.rs
│   ├── provider_runtime.rs
│   └── capability_provider_runtime/
│       ├── command.rs
│       └── http.rs
├── orchestrator/
│   ├── capability_provider_runtime.rs
│   ├── session_runtime.rs
│   ├── session_runtime_planning_runtime.rs
│   └── session_runtime_surface.rs
└── cli/
    ├── inspect/projections.rs
    ├── output_host.rs
    ├── output_runtime.rs
    ├── output_session_status.rs
    ├── provider.rs
    └── session.rs

tests/
├── unit/
│   ├── capability_provider_model.rs
│   ├── capability_provider_runtime.rs
│   └── cli_output.rs
├── contract/
│   ├── assistant_command_definition_contract.rs
│   ├── host_command_output_contract.rs
│   ├── capability_provider_protocol_contract.rs
│   └── distribution_release_surface_contract.rs
└── integration/
    ├── capability_provider_activation_flow.rs
    ├── capability_provider_execution_flow.rs
    └── host_session_runtime_flow.rs

assistant/
├── antigravity/commands/
├── claude/commands/
├── codex/commands/
├── copilot/prompts/
└── catalog/model-catalog.toml

docs/
├── providers/
│   ├── overview.md
│   ├── protocol.md
│   ├── registration.md
│   └── troubleshooting.md
├── guide/
│   └── common-workflows.md
├── runtime/
│   ├── inspect.md
│   ├── run.md
│   ├── status.md
│   └── trace.md
└── reference/
    ├── cli.md
    └── configuration.md

tech-docs/
├── architecture.md
├── configuration.md
└── getting-started.md

distribution/
├── channel-metadata.toml
├── homebrew/Formula/boundline.rb
└── winget/manifests/a/ApplyThe/Boundline/0.72.0/
```

**Structure Decision**: Keep provider protocol work inside the existing Rust
workspace and reuse the current transport, auth, runtime-surface, and
framework-adapter patterns where they are structurally similar. The provider
protocol must remain distinct from the model-provider runtime in
`src/adapters/capability_provider_runtime.rs` and from the stage-owning framework-adapter
contract in `src/domain/framework_adapter.rs`; implementation should share
supporting primitives, not collapse the three concepts into one generalized
abstraction.

## Phase 0 Research Conclusions

- Keep external capability providers separate from both framework adapters and
  model-provider routes. Framework adapters can own an entire Boundline stage;
  capability providers offer bounded capabilities that Boundline admits and
  validates.
- Use one typed logical contract across all provider lifecycle calls:
  `capabilities`, `health`, `prepare`, `execute`, and `collect_evidence`.
- Support the same protocol over two transport kinds in the first slice:
  local command/stdio and HTTP endpoint. Discovery may find either, but
  registration and activation remain explicit.
- Persist non-secret provider registration, activation, and capability metadata
  in Boundline-owned config while keeping secret handles or auth references out
  of tracked config and traces.
- Represent setup requirements explicitly as typed runtime state so the CLI,
  status, and inspect surfaces can show what remains missing before activation
  is allowed to complete.
- Treat provider setup and activation as an atomic state transition: if setup,
  auth resolution, or dry-run health validation fails, the previously active
  provider configuration stays authoritative.
- Keep the initial execute path read-only in trust semantics even when a
  provider returns patch proposals. Provider output remains a proposal until
  Boundline validates and accepts it.
- Use explicit validation disposition and failure classes as first-class trace
  state so operators can distinguish readiness, permission, execution, and
  post-execution validation failures.
- Allow specialized execution profiles only as overlays on top of the generic
  provider contract. If overlay metadata conflicts with generic provider
  metadata or Boundline runtime policy, the stricter Boundline policy wins and
  admission fails closed.
- The required provider-catalog audit produced a no-change result on
  2026-06-05 for the bundled families already represented in
  `assistant/catalog/model-catalog.toml`.

## Phase 1 Design Outputs

- [research.md](research.md) records the provider-catalog audit, transport and
  storage decisions, and rejected alternatives.
- [data-model.md](data-model.md) defines provider registrations, capability
  declarations, health and preparation reports, permission envelopes, bounded
  execution records, evidence collection records, and validation dispositions.
- [external-capability-provider-runtime-contract.md](contracts/external-capability-provider-runtime-contract.md)
  defines protocol calls, additive runtime projections, fail-closed conflict
  rules, and assistant-surface obligations.
- [quickstart.md](quickstart.md) defines isolated validation scenarios for
  registration without activation, interrupted setup, unavailable providers,
  metadata conflicts, evidence normalization, and release-quality closure.

## Post-Design Constitution Recheck

The design remains compliant after Phase 1. It introduces no hidden provider
trust path, no automatic activation, no Canon-owned control flow, no
background lifecycle daemon, and no speculative concrete provider bundle. The
first slice is deliberately bounded: it defines the generic provider protocol,
registration and activation runtime, explicit permission and evidence rules,
and the minimum operator-facing projections needed to use external capability
providers safely.
