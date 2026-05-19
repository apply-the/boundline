# Contract: Release Alignment

## Purpose

Define the release and compatibility alignment obligations for the reasoning
profile closure slice.

## Release Targets

```toml
boundline_target = "0.62.0"
boundline_supported_window = "0.62.x"
canon_target = "0.59.0"
canon_supported_window = "0.59.x"
contract_line = "governed_reasoning_posture_v1"
```

## Alignment Rules

- Boundline MUST update its release-facing docs, changelog, roadmap, validation
  report, and compatibility tests to match the final shipped profile
  classification.
- Canon MUST publish the matching `0.59.x` companion compatibility docs, version
  windows, changelog notes, and contract-test expectations for the released
  `0.62.x` Boundline pair.
- When the sibling Canon repository is unavailable, Boundline MUST still be able
  to validate the supported pair using repo-local compatibility artifacts.
- Canon's published compatibility material MUST admit the new Boundline window
  and its version anchors MUST match the released pair.

## Failure Conditions

Validation fails closed when any of the following occur:

- Boundline runtime claims and release-facing docs disagree on whether a
  capability is a shipped profile or a primitive
- compatibility tests require the sibling Canon repository with no local
  fallback artifact
- Boundline publishes a new release window that the Canon companion artifacts do
  not admit
- Canon is modified without an aligned version or changelog update