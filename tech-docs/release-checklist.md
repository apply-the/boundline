# Release Checklist: Version Bump

This document lists every file that must be updated when advancing the crate
version tag. Two contract tests enforce alignment automatically:
`distribution_metadata_keeps_versions_and_bundle_names_aligned` and
`release_surface_tracks_current_workspace_version_without_stale_status_heading`.

## Files To Update

### Version sources (the single source of truth)

- **`Cargo.toml`** — `[workspace.package] version = "X.Y.Z"`. All crates in the
  workspace inherit this value; no crate-local version override is needed.
- **`Cargo.lock`** — root and member-package version entries for `boundline`,
  `boundline-core`, `boundline-adapters`, and `boundline-cli` must match the
  workspace version after the bump.

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

- **`tech-docs/getting-started.md`** and **`tech-docs/architecture.md`** — update any
  current Canon compatibility target named in the active product docs.

- **Derived-index lifecycle docs** — when the local semantic retrieval surface
  changes, update the README, getting started docs, configuration reference,
  troubleshooting guidance, and any wiki quick-start or architecture pages that
  describe `boundline index status|refresh|rebuild|clean|doctor`, semantic
  fallback guidance, or hook-managed stale detection.

- **CLI and assistant surface updates** — when `status`, `inspect`, or the
  generated assistant command packs change, document the active usage and any
  revised fallback guidance in the README and assistant command docs.

### Assistant plugin surface

- **`assistant/plugin-metadata.json`** — `"version": "X.Y.Z"` plus any
  active `supportModes` or `supportModeNotes` declarations.

- **`.claude-plugin/manifest.json`**, **`.codex-plugin/plugin.json`**,
  **`.cursor-plugin/manifest.json`**, and **`.copilot-prompts/pack.json`** —
  update the packaged host manifest version to `X.Y.Z`.

- **`assistant/global/manifest.json`** — update `"version": "X.Y.Z"` and any
  active host-support declarations when the release changes bootstrap or host
  parity wording.

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

## Governed Contract-Package Publication

T019 prepares the `0.90.0` contract candidates; it does not authorize
publication. T020 may begin only from the source commits and artifact digests
recorded in
`specs/083-governed-1-0-release/evidence/m1e-package-readiness.md`.

The later T020 operator must:

1. Rebuild both candidates from the recorded clean source commits and require
   the reviewed SHA-256 and normalized payload digests to match.
2. Re-run `cargo publish --dry-run --locked -p boundline-protocol` and the
   equivalent `canon-contracts` command.
3. Publish `boundline-protocol` and `canon-contracts` to crates.io. They are
   independent roots; the recorded deterministic operator order is
   Boundline first, Canon second.
4. Retrieve each exact version from crates.io and repeat the clean-consumer,
   metadata-source, build, test, and offline-repeat checks.
5. Stop without tagging if upload, checksum, retrieval, resolution, build, or
   test evidence differs from the reviewed candidate.
6. Create the repository-local `0.90.0` tag only after registry verification.
   Each tag must be annotated and cryptographically signed, target the recorded
   source commit exactly, and include package/version identity plus its
   provenance digest.
7. Verify each tag signature locally, prove the tag is new, and push the tag
   without force. There is no unsigned fallback and no tag reuse.

Publication is irreversible. Package upload, final tag creation, commit push,
and tag push always require a separate explicit approval.

### 0.90.0 contract-package publication record

T020 completed on 2026-07-28 under separate explicit approval. The published
packages are `boundline-protocol 0.90.0` and `canon-contracts 0.90.0`; the
Speckit adapter was qualified against the crates.io protocol package but was
not published.

The Boundline tag `0.90.0` is signed and targets
`860b7247d8343c63dca2940a923b58f3172632f9`. Its tag object is
`27ed5a17c34016554186eab84288021903d300c3`. The Canon tag with the same name
targets `bd361d7e2ad112e5e0d599267e8024e748293605`, with tag object
`cf68539f9e326b8542576319eff7bb8120c77153`. Both remote tags were fetched into
isolated verification namespaces and passed target, object-identity, and GPG
signature checks.

The complete registry checksums, consumer lockfile digests, timestamps, and
historical preflight failures are retained in
`specs/083-governed-1-0-release/evidence/m1e-package-readiness.md`.
