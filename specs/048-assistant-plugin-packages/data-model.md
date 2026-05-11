# Data Model: Assistant Plugin Packages

## HostPluginPackage

- **Purpose**: Represents one host-facing package or prompt-pack surface.
- **Fields**:
  - `host_id`: stable host id such as `claude-code`, `codex`, `cursor`, or `copilot-prompts`.
  - `package_path`: repository-relative package folder.
  - `manifest_path`: repository-relative JSON file when the host package has JSON metadata.
  - `commands_path`: repository-relative command binding file when applicable.
  - `capabilities`: command or capability ids exposed by the package.
  - `paths`: repository-relative paths to shared command, skill, prompt, hook, or asset sources.
- **Validation Rules**:
  - Package path must exist.
  - JSON manifests must parse.
  - Required metadata fields must be present.
  - Version must match the workspace package version.
  - Path references must exist.
  - Capability claims must not exceed what the package can honestly represent.

## SharedPluginMetadata

- **Purpose**: Shared Boundline identity and package configuration used to keep host packages aligned.
- **Source Asset**: `assistant/plugin-metadata.json`.
- **Fields**:
  - `name`, `displayName`, `version`, `description`, `author`, `homepage`, `repository`, `license`, `keywords`.
  - `positioning`: approved Boundline positioning phrases.
  - `capabilities`: supported Boundline command/capability records.
  - `requiredPaths`: repository-relative files or folders every package depends on.
  - `supportedHosts`: host package registry.
  - `copilot`: prompt-pack boundary metadata.
  - `prohibitedPositioning`: disallowed product claims.
- **Validation Rules**:
  - Version matches workspace version.
  - Required paths exist.
  - Required command ids are present in capabilities.
  - Prohibited positioning terms do not appear in metadata.

## CommandBinding

- **Purpose**: Declares one namespaced Boundline chat command and maps it to real CLI/runtime behavior.
- **Source Asset**: `assistant/commands/session-workflow.json`.
- **Fields**:
  - `id`: required namespaced command such as `/boundline:start`.
  - `label`: human-readable command name.
  - `purpose`: short delivery purpose.
  - `boundlineCommand`: concrete CLI command or command pattern.
  - `skillRef`: host command-pack file or prompt file reference.
  - `stateHandling`: states the binding must surface explicitly.
  - `conditional`: true only for commands such as `/boundline:govern` that depend on Canon governance configuration.
- **Validation Rules**:
  - Every required command id exists.
  - Referenced files exist.
  - Conditional commands must be labeled conditional and cannot imply Canon is the default entrypoint.
  - Command text must preserve `.boundline/session.json` as authoritative state.

## StarterPrompt

- **Purpose**: Short host-discoverable prompts that help a user start or resume bounded delivery.
- **Source Asset**: `assistant/prompts/starter-prompts.md`.
- **Required Prompts**:
  - "I want to turn this idea into a bounded implementation plan."
  - "Help me fix a failing test with Boundline."
  - "Continue the active Boundline session."
  - "Inspect the latest Boundline trace and tell me the next safe action."
- **Validation Rules**:
  - All required starter prompts appear in the prompt asset.
  - Prompts point users into the runtime-backed session loop.

## PackageValidationReport

- **Purpose**: Closeout evidence for package coherence and implementation validation.
- **Source Asset**: `specs/048-assistant-plugin-packages/validation-report.md`.
- **Fields**:
  - version evidence
  - package validation command output
  - fmt/clippy/test output
  - touched-Rust-file coverage evidence
  - acceptance criteria readback
- **Validation Rules**:
  - Must be updated after fresh verification commands.
  - Must not claim success without command evidence.
