# Research: Assistant Plugin Packages

## Decision 1: Keep host packages as metadata and command bindings

- **Decision**: Treat `.claude-plugin/`, `.codex-plugin/`, `.cursor-plugin/`, and `.copilot-prompts/` as installation, discovery, metadata, command-binding, prompt, and host-glue surfaces only.
- **Rationale**: Boundline already owns the local session-native runtime. Host packages should help an assistant discover and invoke it, not become a second runtime or make chat state authoritative.
- **Alternatives considered**:
  - Add host-specific runtime wrappers: rejected because it would create divergent behavior and duplicate the CLI/session loop.
  - Keep only the existing Markdown command packs: rejected because the user asked for host-native package metadata and validation.

## Decision 2: Use shared metadata and one shared command definition source

- **Decision**: Add `assistant/plugin-metadata.json` and `assistant/commands/session-workflow.json` as common sources for package identity, paths, capabilities, required commands, and CLI mappings.
- **Rationale**: Shared metadata reduces version and path drift across host manifests while still allowing host folders to contain their own manifest shape.
- **Alternatives considered**:
  - Duplicate command details into every manifest: rejected because drift would be likely and validation would be harder to reason about.
  - Generate host manifests at runtime: rejected because package folders should be inspectable and installable from the repository.

## Decision 3: Represent Copilot as prompt-pack metadata, not a fake plugin format

- **Decision**: Add `.copilot-prompts/` with prompt-pack metadata and documentation while keeping Copilot prompt content under `assistant/copilot/prompts/`.
- **Rationale**: Copilot environments do not share one stable repository-local plugin manifest. A prompt-pack package boundary is honest, discoverable, and still validates paths and commands.
- **Alternatives considered**:
  - Invent `.copilot-plugin/manifest.json`: rejected because it would imply unsupported install automation.
  - Omit Copilot from package validation: rejected because the user asked to support Copilot where it can be represented cleanly.

## Decision 4: Add a small Rust validation module plus a shell wrapper

- **Decision**: Implement package validation helpers in `src/assistant_plugin_validation.rs`, test them through `tests/assistant_plugin_packages.rs`, and expose `scripts/validate-assistant-plugins.sh` as the maintainer command.
- **Rationale**: Rust validation can reuse existing dependencies, is easy to cover with failure-case tests, and keeps validation close to the repository's normal cargo workflow.
- **Alternatives considered**:
  - Shell-only validation: rejected because robust JSON and TOML checks would become fragile.
  - Python validation: rejected because the repo's validation surface is Rust-first and no Python runtime dependency is needed.

## Decision 5: Catalog research produces no model-entry delta for this slice

- **Decision**: Record a no-entry-change catalog result after checking current public provider docs.
- **Rationale**: The previous feature `047-catalog-voting-inputs` refreshed the bundled route-capable catalog. This slice does not change route selection or add live provider discovery; package metadata can reference Boundline capabilities without expanding the model catalog.
- **Sources reviewed**:
  - OpenAI model catalog: https://platform.openai.com/docs/models
  - Anthropic model overview and release notes: https://docs.anthropic.com/en/docs/about-claude/models/overview and https://docs.anthropic.com/release-notes/claude-apps
  - Google Gemini model catalog: https://ai.google.dev/gemini-api/docs/models
  - GitHub Copilot supported model catalog: https://docs.github.com/en/copilot/reference/ai-models/supported-models
- **Applied delta**: No model entries added or removed. Version metadata is allowed to align with the `0.49.0` feature bump.
