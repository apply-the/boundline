# Contract: Reasoning Version Alignment

## Purpose

Define the current supported release pair for the bilateral reasoning-profile
feature and the checks that must fail closed on drift.

## Supported Release Pair

- **Boundline**: `0.72.x`
- **Canon**: `0.67.x`
- **Shared Contract Line**: `governed_reasoning_posture_v1`

## Required Validation

- Boundline contract tests MUST assert that the active Canon provider contract
  advertises a supported contract line.
- Boundline contract tests MUST assert that the provider's compatibility window
  admits the active Boundline version.
- Canon contract tests or docs checks MUST assert that the published posture
  contract names Boundline `0.72.x` as a supported consumer window.
- Release-facing docs, changelogs, and compatibility guidance in both repos
  MUST agree on the supported pair.

## Failure Conditions

Validation fails closed when any of the following occur:

- unsupported major contract line
- missing compatibility window
- Canon posture doc not found
- Boundline version outside the supported window
- Canon version outside the supported window
- shared vocabulary mismatch for profile or independence terms

## Operator Guidance

When version alignment fails, Boundline should surface one of these operator
stories:

- `upgrade boundline`
- `upgrade canon`
- `refresh reasoning posture contract`
- `reasoning profile unavailable for this version pair`
