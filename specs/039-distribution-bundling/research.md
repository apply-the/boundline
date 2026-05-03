# Research: Distribution & Bundling

## Decision 1: Ship repository-owned release bundles that pair Boundline with one explicit Canon companion version per target

- **Decision**: Produce predictable release bundle assets for supported macOS and
  Windows targets, and keep a repository-owned Canon bundle map that pins the
  compatible Canon version and asset location used to assemble those bundles.
- **Rationale**: The feature needs an install surface that feels first-party
  rather than “install Boundline somehow, then separately figure out Canon”. A
  bounded bundle map keeps the pairing explicit, testable, and release-aligned
  without making Boundline depend on Canon-owned control flow at runtime.
- **Alternatives considered**:
  - Depend on a separately managed Homebrew or winget Canon package. Rejected
    because the slice needs one Boundline-owned pairing story and cannot assume
    external package publication parity.
  - Leave Canon installation entirely to documentation. Rejected because the
    feature explicitly promises paired install, update, and repair guidance.

## Decision 2: Extend `doctor` with an install-focused verification path instead of inventing a second verification command family

- **Decision**: Add an installation verification mode to the existing
  diagnostics surface so operators can check Boundline version, Canon companion
  presence, Canon version alignment, and repair guidance through one existing
  product command.
- **Rationale**: `doctor` already owns the “tell me whether I am ready and what
  to do next” story. Extending it keeps readiness and repair semantics explicit
  while avoiding a parallel install-debugging surface.
- **Alternatives considered**:
  - Add a new `distribution verify` command family. Rejected because it would
    fragment the operator-facing diagnostics model.
  - Rely on `boundline --version` and `canon --version` only. Rejected because the
    feature needs paired state and repair guidance, not raw version strings.

## Decision 3: Keep official package-channel assets as repository-managed metadata plus release automation

- **Decision**: Store Homebrew and winget artifacts in a dedicated
  `distribution/` tree, validate them in CI, and add a release workflow plus a
  metadata sync script that keeps package metadata aligned with the Cargo
  version and bundle asset naming.
- **Rationale**: The package-channel surface must be durable, reviewable, and
  version-controlled. Repo-managed metadata makes drift obvious and keeps the
  maintainer story bounded to one release path.
- **Alternatives considered**:
  - Publish package metadata manually outside the repository. Rejected because
    the feature needs one coherent release surface with explicit source of
    truth.
  - Generate everything ad hoc inside GitHub Actions with no checked-in
    artifacts. Rejected because maintainers and reviewers would lose normal
    diff-based visibility.

## Decision 4: Keep Homebrew and winget as the only official bundled channels, with source install remaining the fallback path

- **Decision**: Treat Homebrew on macOS and winget on Windows as the supported
  bundled install channels for this release, while keeping the existing source
  installation path documented for Linux and unsupported environments.
- **Rationale**: The roadmap item is about real distribution coverage, but the
  user explicitly asked for a bounded, feature-complete slice rather than a new
  matrix of partially maintained package managers.
- **Alternatives considered**:
  - Add apt, scoop, npm, or standalone GUI installers in the same slice.
    Rejected because each additional channel would widen validation and support
    costs without improving the primary operator path proportionally.
  - Drop source-install guidance entirely. Rejected because Linux and advanced
    operators still need a supported fallback.

## Decision 5: Split product documentation into a brutal quick path and a separate advanced architecture layer

- **Decision**: Rewrite the top-level onboarding path so README and
  getting-started lead with install, verify, initialize, and run, while moving
  detailed architecture, Canon boundaries, compatibility nuance, and deeper
  routing explanation into clearly labeled advanced docs.
- **Rationale**: Current docs mix first-run instructions with architectural
  narrative. The user feedback for this slice explicitly asks for a fast path
  plus a more deliberate advanced layer, while keeping Boundline and Canon
  responsibilities explicit.
- **Alternatives considered**:
  - Keep one long README with minor edits. Rejected because the current problem
    is structural rather than local wording.
  - Move all architecture detail out of the repository root. Rejected because
    advanced users still need a discoverable explanation of product boundaries.

## Decision 6: Treat 0.39.0 release closure as part of the feature, not a follow-up cleanup

- **Decision**: Ship the slice as `0.39.0`, including version bump,
  distribution metadata, docs and assistant guidance updates, roadmap cleanup,
  changelog, formatting, lint, test validation, and >95% line coverage for all
  modified Rust files.
- **Rationale**: Distribution and bundling change the first interaction users
  have with the product, so the release surface has to close coherently in the
  same slice.
- **Alternatives considered**:
  - Defer docs and release metadata to a later cleanup pass. Rejected because
    the user asked for one feature-complete macrofeature.