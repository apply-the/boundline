# S13B - Local Sandbox Execution Runtime

## Owner

Boundline

## Status

B-level, after B13A and provider permission vocabulary (B07)

## Speckit Seed Notes

- Seed role: local Docker sandbox for high-risk or provider-backed commands.
- First slice: Docker sandbox with workspace mount policy, network policy,
  filesystem isolation, and secret handle inheritance.
- Depends on: B13A (execution safety foundation), B07 (provider permission
  vocabulary).
- De-duplication: this seed enforces sandbox policy; it does not redefine
  command intent classification or evidence capture from B13A.

## Strategic Role

This feature adds container-level isolation for the highest-risk commands.

B13A handles the vast majority of execution safety cases. B13B is reserved
for commands where the developer machine itself needs protection — untrusted
generated code, destructive scripts, provider-backed mutations, and red-zone
operations.

## Problem

B13A provides policy-level safety but not OS-level isolation.

This is insufficient for:

- untrusted generated code
- destructive scripts (rm -rf, database drops)
- risky migrations
- dependency install scripts
- tests with side effects
- external provider execution
- red-zone mutation commands

## Core Scope

- Local Docker sandbox
- Workspace mount policy (read-only vs read-write paths)
- Allowed path policy (which directories are visible)
- Network policy (allow, deny, restricted)
- Filesystem overlay isolation
- Command execution inside sandbox
- Secret handle inheritance (secrets passed as handles, not values)
- Artifact capture from sandbox
- Sandbox patch export (changes as a diff)
- Sandbox commit or discard flow

## Dependencies

- **B13A**: execution safety foundation (command classification, evidence
  capture, redaction, mutation boundaries)
- **B07**: provider permission vocabulary (what secrets and capabilities
  a provider can request)

## Enables

- **C07 Integration Onboarding**: enables sandboxed provider setup for
  Canon CLI integration
