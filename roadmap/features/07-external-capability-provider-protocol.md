# S10 - External Capability Provider Protocol

## Owner

Boundline

## Status

High-priority architecture feature

## Speckit Seed Notes

- Seed role: native capability boundary for external systems.
- First slice: implement discovery, explicit operator registration,
  `health`, setup-requirement projection, and one read-only `execute` path for
  a provider that returns findings and evidence only.
- Depends on: event/trace schema from evals and observability, or a deliberately
  minimal trace projection if this seed lands first.
- De-duplication: permission envelope lives here; sandbox enforcement lives in
  seed 13; browser behavior lives in seed 15; route economics lives in seed 14.

## Strategic Role

This feature makes Boundline framework-agnostic without turning it into an uncontrolled plugin runner.

External systems may provide bounded capabilities. Boundline keeps session state, permissions, trace, evidence validation, admission control, and setup flow.

## Problem

Without a generic provider protocol, Boundline will accumulate one-off adapters:

- custom harness adapter
- custom browser adapter
- custom static-analysis adapter
- custom sandbox adapter
- custom MCP adapter
- custom research adapter
- unsafe one-off setup prompts for provider configuration
- accidental activation of locally discoverable executables

That creates adapter sprawl and inconsistent trust boundaries.

## Core Principle

Provider output is not truth.

```text
Provider produces claims, findings, artifacts, evidence, and state patch proposals.
Boundline validates, traces, accepts, rejects, or escalates.
Canon may govern accepted evidence later.
```

## Required Protocol Calls

### capabilities

Provider declares identity and supported capabilities.

Must include:

- provider ID
- protocol version
- capability IDs
- supported lifecycle phases
- supported inputs
- supported outputs
- mutation support
- required permissions
- evidence formats
- version

### health

Provider reports readiness.

Must include:

- ready/degraded/unavailable
- missing dependencies
- auth state where relevant
- warnings
- supported runtime environment

### prepare

Provider declares required and optional context before execution.

Must include:

- required context
- optional context
- missing evidence
- expected artifacts
- risk observations
- estimated cost or runtime if available

### execute

Provider executes a bounded request.

Must include in request:

- request ID
- session ref
- step ID
- capability
- goal
- lifecycle phase
- authority zone
- Context Pack refs
- permissions
- limits
- expected outputs

Must include in response:

- status
- observations
- findings
- artifacts
- evidence refs
- state patch proposals
- limitations
- next actions

### collect_evidence

Provider normalizes evidence after execution.

Must include:

- claims
- evidence refs
- artifacts
- findings
- limitations
- reproducibility info

## Permission Model

Every request should include explicit permissions:

```text
read_files
write_files
run_commands
network
read_secrets
write_artifacts
allowed_paths
max_runtime
max_output_bytes
```

Default should be least privilege.

## Transport Options

V1 should support:

- JSON over stdio
- CLI process adapter
- JSON-RPC compatible envelope where practical

Later adapters:

- MCP client bridge
- HTTP local provider
- sandbox provider
- browser provider

## Operator Setup And Activation

Provider onboarding is a Boundline runtime concern, not Canon setup logic.

V1 should support:

- explicit operator registration and activation of a provider
- setup requirement projection before first use
- non-secret configuration capture through interactive or config-driven flows
- secret-handle references rather than prompt-visible secret values
- connectivity or health dry-run before activation is marked ready
- atomic setup so an interrupted flow leaves the previous active config intact

Hard boundaries:

- a locally discoverable executable must not auto-enable itself as a provider
- setup must not persist raw secrets in traces or tracked files
- provider activation must remain visible in status and inspect

## Provider Types

- read-only context provider
- planning provider
- review provider
- guardian provider
- verification provider
- mutation provider
- browser provider
- sandbox provider
- research provider
- code analysis provider

## Evidence Packet

Evidence should be Boundline-owned and Canon-compatible, not Canon-specific.

Suggested shape:

```json
{
  "kind": "boundline-provider-evidence",
  "provider_id": "string",
  "capability": "string",
  "claims": [],
  "findings": [],
  "artifacts": [],
  "limitations": [],
  "reproducibility": {}
}
```

## Acceptance Criteria

- Boundline can discover a provider's capabilities.
- Boundline can reject an unavailable provider before run.
- Boundline requires explicit operator registration before a provider can be
  activated.
- Boundline can project required setup fields and block activation until they
  are satisfied.
- Boundline can run a health or connectivity check before marking the provider
  ready.
- Provider execution is permission-scoped.
- Provider output cannot directly mutate Boundline state without validation.
- Evidence packets are trace-linked.
- Provider limitations are visible in inspect.
- MCP can later be implemented as an adapter, not the core architecture.

## Risks

- External providers become trusted implicitly.
- Local executables become active accidentally.
- Setup leaks secrets into prompts or traces.
- Hidden provider state makes runs non-reproducible.
- Permissions are too broad.
- Protocol is too generic to validate.

## Hard Rule

Boundline owns admission control. Providers never approve themselves.
Discoverability is not activation.
