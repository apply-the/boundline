# Data Model: Distribution & Bundling

## DistributionChannel

- **Purpose**: Names one supported install or update path exposed by Boundline.
- **Values**:
  - `homebrew`
  - `winget`
  - `source`
- **Validation rules**:
  - `homebrew` is only valid for supported macOS targets.
  - `winget` is only valid for supported Windows targets.
  - `source` remains available as the fallback channel and must not claim a
    bundled Canon install.

## DistributionTarget

- **Purpose**: Names one concrete release-bundle target that Boundline can ship.
- **Fields**:
  - `target_triple`: Rust target triple.
  - `platform`: operator-facing platform name.
  - `archive_name`: predictable release asset name.
  - `package_channel`: official install channel associated with the target.
- **Validation rules**:
  - Each official bundled target maps to exactly one asset name.
  - Asset names remain stable across release workflow, Homebrew formula, and
    winget manifests.

## CanonBundlePolicy

- **Purpose**: Declares the Canon companion version window and bundle assembly
  inputs for one Boundline release.
- **Fields**:
  - `canon_version`: pinned Canon version expected by the current Boundline release.
  - `asset_map`: per-target source URL or source identifier for the Canon
    binary used in release bundles.
  - `version_requirement`: operator-facing version requirement string used by
    install diagnostics.
  - `repair_hint`: bounded repair guidance when the Canon companion is missing
    or mismatched.
- **Validation rules**:
  - `canon_version` must be explicit; “latest” is invalid as stored metadata.
  - Every official bundled target must have a Canon asset entry.
  - Repair guidance must stay channel-aware and must not imply unsupported
    package managers.

## CompanionState

- **Purpose**: Represents the user-visible Canon pairing state reported by
  install diagnostics.
- **Values**:
  - `ready`
  - `already_satisfied`
  - `blocked`
  - `repair_needed`
- **Validation rules**:
  - `ready` means the bundled or discovered Canon runtime matches the supported
    version requirement.
  - `already_satisfied` means an acceptable Canon runtime was already present
    before bundle repair was needed.
  - `blocked` means Boundline cannot evaluate or satisfy the pairing because a
    required prerequisite is missing.
  - `repair_needed` means Boundline can name a bounded repair path.

## InstallDiagnosticsCheck

- **Purpose**: Captures one explicit verification statement for the installed
  distribution surface.
- **Fields**:
  - `name`: stable check identifier.
  - `status`: passed or failed.
  - `message`: concise operator-facing explanation.
  - `repairable`: whether the failure has a direct repair action.
- **Validation rules**:
  - `name` must stay stable across tests and docs.
  - Failed checks that are repairable must contribute a suggested action.

## InstallDiagnosticsReport

- **Purpose**: Summarizes whether the current Boundline installation is usable and
  paired with a compatible Canon companion.
- **Fields**:
  - `subject`: `install`.
  - `boundline_version`: current Boundline version string.
  - `supported_canon_version`: pinned Canon version requirement.
  - `channel_candidates`: channels that fit the current platform.
  - `checks`: ordered install diagnostics checks.
  - `companion_state`: paired Canon state.
  - `ready`: overall readiness flag.
  - `suggested_actions`: ordered repair or next-step guidance.
- **Validation rules**:
  - The report must always include the current Boundline version and supported
    Canon version requirement.
  - `ready` may be true only when no blocking failed checks remain.
  - `companion_state` must match the failed-check set and suggested actions.

## DistributionManifestSet

- **Purpose**: Groups the checked-in package metadata derived from a release.
- **Fields**:
  - `boundline_version`: release version.
  - `homebrew_formula_path`: formula path for macOS.
  - `winget_manifest_paths`: manifest set paths for Windows.
  - `bundle_assets`: release bundle asset names and checksums.
  - `source_fallback_ref`: docs reference for source installation.
- **Validation rules**:
  - All manifest versions must match the Cargo package version.
  - Bundle asset names referenced by package metadata must exist in the release
    workflow configuration.
  - Source fallback documentation must remain present even when bundled
    channels are supported.

## DocumentationLayer

- **Purpose**: Separates first-run guidance from architectural explanation.
- **Values**:
  - `quick_path`
  - `advanced_architecture`
- **Validation rules**:
  - The quick path must cover install, verification, initialization, and first
    bounded run without requiring architecture reading.
  - The advanced layer must keep Boundline and Canon responsibilities explicit and
    must link back to the quick path for operators who only need onboarding.