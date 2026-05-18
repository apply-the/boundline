# Release Checklist: Version Bump

This document lists every file that must be updated when advancing the crate
version tag. Two contract tests enforce alignment automatically:
`distribution_metadata_keeps_versions_and_bundle_names_aligned` and
`release_surface_tracks_current_workspace_version_without_stale_status_heading`.

## Files To Update

### Version sources (the single source of truth)

- **`Cargo.toml`** — `[workspace.package] version = "X.Y.Z"`. All crates in the
  workspace inherit this value; no crate-local version override is needed.

### Distribution surface

- **`distribution/channel-metadata.toml`** — three fields:
  - `boundline_version = "X.Y.Z"`
  - `manifest_root = "distribution/winget/manifests/a/ApplyThe/Boundline/X.Y.Z"`
  - `bundle_name = "boundline-bundle-X.Y.Z-windows-x86_64.zip"`

- **`distribution/homebrew/Formula/boundline.rb`** — release tag, version, and
  active pairing strings:
  - `url "...", using: :git, tag: "X.Y.Z"`
  - `version "X.Y.Z"`
  - any current-release caveat text that names the Boundline and Canon pairing

- **`distribution/winget/manifests/a/ApplyThe/Boundline/X.Y.Z/`** — a new
  version-named directory with the three manifest files copied and updated from
  the previous release:
  - `ApplyThe.Boundline.yaml` — `PackageVersion: X.Y.Z`
  - `ApplyThe.Boundline.installer.yaml` — `PackageVersion: X.Y.Z`, bundle URL,
    and release download path
  - `ApplyThe.Boundline.locale.en-US.yaml` — `PackageVersion: X.Y.Z`

### Repository docs

- **`CHANGELOG.md`** — add `## [X.Y.Z] - YYYY-MM-DD` as the first entry, with
  a concise summary of the deliverables.

- **`ROADMAP.md`** — two edits:
  - Replace the single `## Current Status: vPREV` heading with
    `## Current Status: vX.Y.Z` and update the paragraph below it.
  - Add `### Delivered in X.Y.Z` immediately before the previous
    `### Delivered in PREV` section and list the key deliverables.
  - Verify there is exactly one `## Current Status:` heading in the file.

- **`README.md`** — update the active feature-line paragraph and any
  current-release Canon compatibility statement.

- **`docs/getting-started.md`** and **`docs/architecture.md`** — update any
  current Canon compatibility target named in the active product docs.

### Assistant plugin surface

- **`assistant/plugin-metadata.json`** — `"version": "X.Y.Z"`.

- **`assistant/README.md`** — update or remove any release-specific wording if
  it is describing active assistant surfaces rather than historical release
  notes.

## Canon Compatibility

When the Canon compatibility target also changes (i.e. `SUPPORTED_CANON_VERSION`
in `src/domain/distribution.rs`), update these additional locations:

- `src/domain/distribution.rs` — `SUPPORTED_CANON_VERSION`
- `distribution/channel-metadata.toml` — `canon_version = "C.C.C"`
- `distribution/homebrew/Formula/boundline.rb` — the `canon-source` resource
  tag `"C.C.C"`, the caveats string, and the `canon --version` test assertion
- `distribution/channel-metadata.toml` — `canon_asset` URL
- `tests/fixtures/canon_capabilities_*.json` — fixture `canon_version` fields
- `tests/unit/distribution_metadata.rs` — the expected supported Canon version

## Validation

Run the two distribution contract tests after every version bump before opening
a PR:

```bash
cargo test --test contract distribution_metadata_contract::distribution_metadata_keeps_versions_and_bundle_names_aligned -- --exact
cargo test --test contract distribution_release_surface_contract::release_surface_tracks_current_workspace_version_without_stale_status_heading -- --exact
```

Or run the full contract suite:

```bash
cargo nextest run --workspace --all-features
```
