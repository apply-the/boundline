# Research: Developer UX for Orchestrator Core

## Decision 1: Keep the developer command surface inside the existing crate

- **Decision**: Add one CLI binary target to the current `synod` package rather than creating a separate CLI crate or a second workspace member.
- **Rationale**: The new feature only needs a thin command surface over the existing orchestrator library, trace store, and deterministic test fixtures. A second crate would add manifest, dependency, and release complexity without creating additional delivery value for this slice.
- **Alternatives considered**:
  - Separate `synod-cli` crate: rejected because it duplicates package-level complexity before the CLI has proven value.
  - Shell scripts only: rejected because the spec requires an explicit, inspectable command interface with reliable exit semantics.

## Decision 2: Use `clap` as the only new runtime dependency

- **Decision**: Add `clap` 4.x for subcommand parsing, help text, and stable argument validation while keeping output formatting on the standard terminal surface.
- **Rationale**: The feature needs a small but reliable CLI with multiple commands, explicit usage errors, and bounded inputs. `clap` provides that with minimal surface area and avoids ad hoc parsing logic that would be harder to test and maintain.
- **Alternatives considered**:
  - Manual argument parsing with `std::env::args`: rejected because it increases validation and help-text work for no product gain.
  - Interactive or TUI frameworks: rejected because the constitution defers UI work and the spec only requires readable command-line output.

## Decision 3: Reuse the existing trace store and add inspection as a formatting layer

- **Decision**: Keep persisted execution traces under `<workspace>/.synod/traces/` as the canonical execution record and implement trace inspection as a summary layer that reads those traces without changing their storage model.
- **Rationale**: The current core already emits durable local traces. The missing capability is readable inspection, not a new persistence mechanism. Reusing the existing trace record keeps the feature small and preserves compatibility with the core orchestrator behavior.
- **Alternatives considered**:
  - Introduce a new trace database or index: rejected because it adds infrastructure outside the scope of a local developer UX slice.
  - Emit a separate command-specific trace format: rejected because it fragments observability and would duplicate recorded execution data.

## Decision 4: Make the documented run path deterministic with workspace fixtures

- **Decision**: Implement the documented run path with a repository-local workspace fixture manifest plus deterministic built-in agent and tool adapters.
- **Rationale**: The spec requires contributors to see meaningful orchestration behavior quickly and repeatedly without relying on synthetic onboarding surfaces or external providers. Workspace fixtures keep the workflow stable, fast to debug, and grounded in real red-to-green behavior.
- **Alternatives considered**:
  - Use live model or shell integrations in the first release: rejected because they would make onboarding and validation nondeterministic.
  - Hardcode a synthetic scripted run: rejected because it hides whether Synod can actually validate and change a real workspace.

## Decision 5: Provide a dedicated `doctor` command for local readiness

- **Decision**: Add a `doctor` command that checks the local repository context, workspace writability, trace-directory readiness, and the availability of the built-in developer command surface before a run starts.
- **Rationale**: The spec requires actionable setup diagnostics. A dedicated command is the smallest way to make readiness explicit without mixing environment checks into every run invocation or forcing developers back into README-driven troubleshooting.
- **Alternatives considered**:
  - Fail only at run start: rejected because it hides readiness issues until after the developer has already chosen a command path.
  - Document setup only in prose: rejected because FR-008 requires a runnable diagnostic surface.

## Decision 6: Document the CLI as markdown command contracts

- **Decision**: Define the developer command surface, workspace fixture, diagnostics report, and trace summary as markdown contracts under `specs/002-developer-ux-orchestrator/contracts/`.
- **Rationale**: The repository already documents interface boundaries as markdown contracts for the orchestrator core. The new feature exposes a local CLI, not a network API, so markdown contracts remain the smallest and clearest fit.
- **Alternatives considered**:
  - OpenAPI or JSON Schema: rejected because the interface is a local CLI and human-readable command output, not an HTTP service.
  - No explicit contracts: rejected because the feature introduces multiple developer-facing commands and output surfaces that should remain stable and testable.