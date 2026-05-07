# Research: Stack-Neutral Workspace Entry

**Feature**: 043-stack-neutral-init  
**Date**: 2026-05-06

## R1: Generic Workspace Readiness Must Be Stack-Neutral

**Decision**: Treat any existing writable local directory as a valid native-workflow workspace and remove the Rust-specific manifest check from generic readiness and native direct-run bootstrap.

**Rationale**: The current readiness path in `src/cli/diagnostics.rs` blocks both `doctor --workspace` and the direct native `run --goal` flow unless `Cargo.toml` is present. That conflicts with spec 038 and with the primary Boundline story for empty or non-Rust repositories. Workspace readiness should validate local operability, such as existence, writability, trace persistence, and optional execution-profile state, while stack or domain credibility should be resolved later by capture and planning.

**Alternatives considered**:
- Keep `Cargo.toml` as a generic prerequisite: rejected because it hard-codes Rust into a multi-language operator surface.
- Infer a single required manifest by stack: rejected because empty repositories need to enter the same native path before a stack is chosen.

## R2: Reuse The Existing Built-In Routing Catalog For Assistant Defaults

**Decision**: Use the built-in routing defaults already defined in `src/domain/configuration.rs` as the single source of truth for assistant-model defaults, then derive a per-runtime default model catalog from those routes for `init`.

**Rationale**: The repository already documents and tests a built-in route catalog: Codex uses `gpt-5-codex`, Copilot uses `gpt-5.4`, Claude uses `sonnet-4`, and Gemini uses `gemini-2.5-pro`. Reusing that existing catalog avoids introducing a second hard-coded table in `init`, keeps effective routing and initialization aligned, and gives docs one authoritative set of defaults.

**Alternatives considered**:
- Add a second init-only model map: rejected because it would drift from effective routing and docs.
- Discover models from external provider APIs at init time: rejected because the feature needs deterministic offline behavior and the user explicitly delegated model sourcing to Boundline.

## R3: Seed Route Slots Deterministically From Assistant Selection

**Decision**: When `init` receives assistant targets and no explicit `--route` values, seed `planning`, `implementation`, `verification`, and `review` deterministically from the selected assistants.

**Rationale**: Operators who choose one assistant target expect Boundline to choose the matching default model automatically. For a single selected assistant, every required route slot should use that assistant's default model. For multiple selected assistants, Boundline should preserve the existing built-in slot preferences when the preferred runtime is among the selected assistants, and otherwise fall back to the first selected assistant's default model. Explicit `--route` values still win.

**Alternatives considered**:
- Require manual routes whenever more than one assistant is selected: rejected because it would keep `init` half-configured for common mixed-runtime cases.
- Seed only one slot automatically: rejected because the initial workspace would still be incomplete for the primary flow.

## R4: Technology Hygiene Defaults Need A Merge-Only Policy Surface

**Decision**: Add a reusable workspace-hygiene policy module that maps selected domain families and repository tool cues to merge-only ignore defaults for `.gitignore` and optional tool-specific ignore files.

**Rationale**: The new slice needs to carry domain selection into real repository hygiene without overwriting local rules. A dedicated policy module can keep the default patterns inspectable and testable, while `init` remains responsible only for deciding when to apply them and for writing files safely. The initial implementation will cover universal patterns, domain-family patterns for the first-party catalog, and tool-specific packs for Docker, ESLint, Prettier, Terraform, Helm, and Kubernetes cues.

**Alternatives considered**:
- Keep hygiene rules only in external assistant skills: rejected because they would stay outside the product surface and remain unavailable to CLI-driven initialization.
- Replace ignore files wholesale on every init: rejected because the feature must preserve operator-authored overrides and existing repository rules.

## R5: Release Surfaces Must Move Together

**Decision**: Ship the slice as `0.43.0` and update CLI-facing docs, assistant guidance, changelog, roadmap, and release references together.

**Rationale**: This feature changes how operators enter workspaces and configure assistants during `init`, so stale docs would create the same confusion that triggered the feature. The README, getting-started guide, configuration docs, assistant README, roadmap, and changelog all need synchronized updates.

**Alternatives considered**:
- Update only the README: rejected because `init` and configuration behavior are also described in deeper docs and assistant guidance.
