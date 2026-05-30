# Quickstart: Distribution & Bundling

## Scenario 1: Install Boundline on macOS through Homebrew

1. Run `brew tap applythe/boundline <tap-url>` if the tap is not already
   configured.
2. Run `brew install boundline`.
3. Run `boundline doctor --install`.

Expected result: Boundline is installed from the official macOS channel, install
diagnostics report the current Boundline version, the supported Canon companion
version, and the pairing state is either `ready` or `already_satisfied`.

## Scenario 2: Install Boundline on Windows through winget

1. Run `winget install ApplyThe.Boundline`.
2. Open a new terminal session so the portable executable shims are visible.
3. Run `boundline doctor --install`.

Expected result: the Windows install path is usable from a normal shell, and
install diagnostics report the bounded Canon pairing state plus any explicit
repair action.

## Scenario 3: Repair a missing or mismatched Canon companion

1. Start from a machine where `boundline` is installed but the paired Canon
   companion is absent or on the wrong version.
2. Run `boundline doctor --install`.
3. Follow the printed repair action for the active channel.
4. Run `boundline doctor --install` again.

Expected result: the first diagnostics report is `repair_needed` or `blocked`
with a clear next step, and the second report becomes `ready` or
`already_satisfied` after repair.

## Scenario 4: Use the quick path without reading architecture internals

1. Open README.
2. Follow only the quick path section to install Boundline, verify it, initialize a
   workspace, and run the first bounded task.

Expected result: a new user reaches `goal -> plan -> run -> status
-> next -> inspect` without needing to read compatibility internals or Canon
governance details.

## Scenario 5: Read the advanced architecture layer when deeper control is needed

1. Open the advanced docs linked from README or `docs/getting-started.md`.
2. Read the sections that explain session-native routing, source-install
   fallback, and the Boundline versus Canon boundary.

Expected result: advanced readers can understand how bounded package bundles,
source installs, and Canon governance fit together without confusing Canon for
the product that owns orchestration.

## Scenario 6: Prepare a release-aligned package metadata set

1. Update the crate version for the release.
2. Run `scripts/sync-distribution-metadata.sh`.
3. Review the changed Homebrew formula, winget manifests, and release workflow
   references.

Expected result: all package-channel metadata references the same Boundline version,
the same bounded Canon companion version, and the same predictable bundle asset
names.

## Scenario 7: Validate the 0.39.0 release surface

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run targeted unit, integration, and contract tests for distribution
   diagnostics and metadata.
4. Run `cargo test --no-run --all-targets`.
5. Run `cargo nextest run --workspace --all-features`.
6. Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
7. Verify modified and new Rust files remain above 95% coverage.

Expected result: `0.39.0` ships with official bundled install metadata,
install verification, docs split, roadmap and changelog updates, and validated
coverage for the touched Rust surfaces.