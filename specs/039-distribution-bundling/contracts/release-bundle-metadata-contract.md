# Contract: Release Bundle Metadata

**Feature**: 039-distribution-bundling  
**Date**: 2026-05-03

## Purpose

Define the bounded release metadata that keeps Cargo versioning, bundled Canon
pairing, Homebrew formula references, winget manifests, and release asset names
aligned.

## Required Surface

- The repository must contain one authoritative Canon bundle policy that pins
  the Boundline-supported Canon version for the release.
- Release workflow asset names must be predictable and must match the names
  referenced by the Homebrew formula and winget manifests.
- Package metadata must reference the current Boundline release version and must
  remain diffable in normal repository review.
- Maintainers must have one bounded sync step that updates or validates package
  metadata after the version bump.

## Explicit Boundaries

- The metadata surface must not require manual version edits across unrelated
  files with hidden coupling.
- The feature must not depend on unpublished, undocumented asset naming.
- The repository must continue to document source install as the fallback path
  for unsupported environments.