# S17 - Sandboxed Execution And Secret Inheritance

## Owner

Boundline

## Status

B-level, after provider permissions are defined

## Strategic Role

This feature makes high-risk mutation safer.

Checkpoints are useful, but they do not isolate execution from the developer machine. Sandboxing is necessary for enterprise-risk tasks.

## Problem

Boundline currently relies on local workspace execution and checkpoints.

This is insufficient for:

- untrusted generated code
- destructive scripts
- risky migrations
- dependency install scripts
- tests with side effects
- external providers
- high-risk mutation in red zones

## Core Scope

- Local Docker sandbox
- Workspace mount policy
- Allowed path policy
- Network policy
- Command execution policy
- Secret inheritance
- Artifact capture
- Sandbox commit/rollback
- Trace-linked sandbox metadata
- Governance requirement hooks

## Sandbox Modes

### Read-Only

For analysis, review, indexing.

### Test

Can run tests and write temporary artifacts.

### Mutation

Can edit allowed paths and produce patch artifacts.

### Migration Dry Run

Can run database or schema dry-run against configured disposable resources.

## Secret Inheritance

Secrets must not be passed through prompt context.

Use:

- explicit secret handles
- scoped secret access
- redacted trace output
- provider permission checks
- no secret persistence in sandbox artifacts unless approved

## Algorithms And Techniques

### Filesystem Overlay

Use copy-on-write or mounted overlay to isolate mutation.

### Commit Model

Sandbox produces:

- patch
- artifact bundle
- command log
- test output
- evidence packet

Boundline decides what to apply.

### Network Controls

Support:

- disabled
- allowlist
- inherited from workspace
- provider-specific policy

### Policy Binding

Canon or Boundline governance can require sandboxing for:

- red zone work
- migration work
- dependency install
- unknown provider
- destructive command
- external network access

## Acceptance Criteria

- Boundline can execute a command in local sandbox.
- Sandbox mutation does not affect workspace until commit.
- Secrets are never written to prompt or plain trace.
- Artifacts are captured and trace-linked.
- Sandbox failures preserve evidence.
- Red-zone work can require sandbox mode.
- CLI fallback is clear if Docker is unavailable.

## Risks

- Docker availability varies.
- Sandbox setup becomes slow.
- Secret handling is incomplete.
- Users misunderstand sandbox commit semantics.

## Hard Rule

Sandbox output is evidence, not automatic trust.
