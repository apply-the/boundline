# Feature Specification: External Capability Provider Protocol

**Feature Branch**: `071-capability-provider-protocol`

**Created**: 2026-06-05

**Status**: Draft

**Input**: User description: "Follow [roadmap specs](./spec-external-capability-provider-protocol.md)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Register And Activate A Provider Safely (Priority: P1)

An operator needs to register an external capability provider, review what it can do, satisfy its setup requirements, and activate it deliberately before Boundline is allowed to rely on it during a session.

**Why this priority**: Without safe registration and activation, provider-backed execution cannot be trusted and Boundline cannot replace ad hoc external integrations with a governed runtime surface.

**Independent Test**: Can be fully tested by registering a provider with declared capabilities, projecting missing setup requirements, completing activation, and confirming that discovery alone does not enable execution.

**Acceptance Scenarios**:

1. **Given** a discoverable provider that is not yet registered, **When** the operator inspects available providers, **Then** Boundline shows the provider as discoverable but inactive and unavailable for execution.
2. **Given** a provider that requires setup before use, **When** the operator starts activation, **Then** Boundline shows the setup requirements, blocks activation until they are satisfied, and preserves the previous active configuration if setup is interrupted.
3. **Given** a registered provider that later becomes unavailable, **When** the operator requests execution through that provider, **Then** Boundline blocks the run before execution begins and reports the readiness failure clearly.

---

### User Story 2 - Execute Through A Permission-Scoped Provider (Priority: P2)

A runtime owner needs Boundline to send a bounded request to an activated provider, receive claims and evidence back, and validate the outcome before any provider-suggested state changes are accepted.

**Why this priority**: The provider protocol only delivers value if execution remains bounded, inspectable, and subordinate to Boundline admission control rather than becoming an implicit authority.

**Independent Test**: Can be fully tested by sending one provider-backed request with declared permissions, confirming the provider returns claims and evidence, and verifying that Boundline can reject unsupported or unsafe state patch proposals.

**Acceptance Scenarios**:

1. **Given** an activated provider and a provider-backed stage request, **When** Boundline executes the request, **Then** the request includes capability, lifecycle, authority, context references, permissions, limits, and expected outputs.
2. **Given** a provider response that includes findings, artifacts, evidence references, limitations, and patch proposals, **When** Boundline evaluates the response, **Then** provider output is treated as claims rather than truth and no proposed mutation is accepted without validation.
3. **Given** a provider response that omits required evidence or exceeds granted permissions, **When** Boundline processes the response, **Then** the run is blocked or degraded with a traceable reason rather than silently accepted.

---

### User Story 3 - Inspect Provider State, Limits, And Evidence (Priority: P3)

An operator needs to understand which provider was used, what capability it claimed, what permissions it received, what limitations it reported, and why Boundline accepted or rejected the result.

**Why this priority**: Provider-backed execution becomes operationally unsafe if routing, evidence, limitations, or readiness failures are hidden from status and inspection surfaces.

**Independent Test**: Can be fully tested by running a provider-backed request and confirming that status or inspect surfaces expose provider identity, readiness, capability, evidence references, limitations, and validation disposition without opening raw trace files.

**Acceptance Scenarios**:

1. **Given** a completed provider-backed run, **When** the operator checks status or inspect, **Then** Boundline shows the provider identity, capability used, evidence references, and reported limitations.
2. **Given** a provider-backed run that was blocked before execution or rejected after execution, **When** the operator checks status or inspect, **Then** Boundline shows whether the failure happened during readiness, permission admission, execution, or validation.

### Edge Cases

- What happens when a provider is discoverable on the local machine but has never been explicitly registered by the operator?
- How does the system handle a provider that passes setup but later reports degraded or unavailable health before a run starts?
- What happens when a provider returns artifacts and findings but no reproducible evidence references?
- How does the system handle a provider that requests broader permissions than the selected Boundline stage should allow?
- What happens when a specialized execution profile exists for a provider capability but the generic protocol metadata is missing or inconsistent?
- How does the system behave when Canon-governed evidence is present for the session but Canon itself is not a provider participant in the execution path?

## Out of Scope

- Concrete providers such as browser, sandbox, RecursiveMAS, or static-analysis providers.
- Route economics, model or provider cost policy, and provider benchmarking.
- Provider-specific UI polish beyond the operator-visible setup, status, and inspect surfaces required for safe use.
- Canon acting as an external provider or adapter for stage execution.
- Automatic activation of discovered providers.
- Provider output directly mutating Boundline-owned state without validation.
- Full sandbox enforcement, network isolation, and secret inheritance policy beyond the protocol permission envelope.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST define a provider capability contract that separates provider discovery, registration, activation, health, preparation, execution, and evidence collection.
- **FR-002**: The system MUST require each provider to declare a stable provider identity, protocol version, capability identifiers, supported lifecycle phases, supported inputs and outputs, mutation support, required permissions, and evidence formats before activation can complete.
- **FR-003**: The system MUST treat discovery as informational only; discovering a local executable or remote endpoint MUST NOT activate it automatically.
- **FR-004**: The system MUST require explicit operator registration before a provider can be activated for any session-backed execution path.
- **FR-005**: The system MUST project required and optional setup inputs before first activation and MUST preserve the previous active provider configuration if setup is interrupted or fails.
- **FR-006**: The system MUST capture provider health as one of ready, degraded, or unavailable and MUST block provider-backed execution when the provider is unavailable.
- **FR-007**: The system MUST support a prepare step that reports required context, optional context, missing evidence, expected artifacts, risk observations, and any available cost or runtime estimate before execution begins.
- **FR-008**: The system MUST send bounded execution requests that include request identity, session reference, step or stage reference, capability, goal, lifecycle phase, authority zone, context references, granted permissions, execution limits, and expected outputs.
- **FR-009**: The system MUST require every execution request to use an explicit least-privilege permission envelope that can cover file access, command execution, network access, secret access, artifact writes, allowed paths, runtime bounds, and output bounds.
- **FR-010**: The system MUST require providers to return execution status, observations, findings, artifacts, evidence references, state patch proposals, limitations, and next actions in a structured response.
- **FR-011**: The system MUST treat provider responses as claims rather than truth and MUST validate or reject provider-supplied findings, artifacts, and patch proposals before they affect Boundline-owned state.
- **FR-012**: The system MUST support a collect-evidence step that normalizes provider claims, evidence references, artifacts, findings, limitations, and reproducibility metadata after execution.
- **FR-013**: The system MUST ensure raw secrets are never persisted in traces or tracked files during provider setup, activation, or execution.
- **FR-014**: The system MUST keep provider readiness, activation status, selected capability, evidence references, and reported limitations visible in operator-facing status or inspect surfaces.
- **FR-015**: The system MUST distinguish between readiness failures, permission admission failures, execution failures, and post-execution validation failures in operator-facing output.
- **FR-016**: The system MUST allow specialized execution profiles to layer stage-specific behavior over the generic provider protocol without bypassing protocol permissions, trace boundaries, stop semantics, or final acceptance rules.
- **FR-017**: The system MUST preserve the rule that Canon is not an external provider for stage execution; Canon remains a governed producer consumed through stable contracts outside provider activation semantics.
- **FR-018**: The system MUST allow Boundline to reject provider-backed execution before run start when the provider is unregistered, inactive, unhealthy, or missing required capability metadata.
- **FR-019**: The system MUST retain provider limitations and reproducibility information alongside accepted or rejected provider output so later inspection can reconstruct why the result was or was not trusted.

### Key Entities *(include if feature involves data)*

- **Provider Registration**: The operator-approved record that identifies a provider, its setup state, activation status, and the capabilities that Boundline may route to it.
- **Provider Capability Declaration**: The provider-published description of what a capability can do, which lifecycle phases it supports, what permissions it may need, and what forms of output and evidence it can return.
- **Provider Health Snapshot**: The current readiness report for a provider, including readiness state, missing dependencies, warnings, and runtime environment constraints.
- **Provider Preparation Report**: The pre-execution projection of required context, optional context, missing evidence, expected artifacts, risk observations, and any cost or runtime estimate.
- **Provider Execution Request**: The bounded request that ties provider execution to a specific session, step or stage, authority zone, permission envelope, context references, and expected outputs.
- **Provider Execution Result**: The provider's structured response containing status, observations, findings, artifacts, evidence references, limitations, next actions, and patch proposals.
- **Evidence Collection Record**: The normalized post-execution record that preserves provider claims, artifacts, limitations, evidence references, and reproducibility metadata for later validation and inspection.
- **Specialized Execution Profile**: An optional overlay that maps a provider's capabilities to Boundline stage semantics while remaining subordinate to the generic provider protocol.

## Minimal Observability

Provider-backed execution must emit or project enough structured runtime state for operators to distinguish:

- readiness failure
- permission admission failure
- execution failure
- post-execution validation failure
- accepted provider evidence
- rejected provider evidence
- provider limitations

## Conflict Rule

When provider metadata, specialized profile metadata, and Boundline runtime policy disagree, the stricter Boundline runtime policy wins. If a conflict affects permissions, capability identity, lifecycle phase support, or evidence requirements, provider-backed execution must fail closed before execution starts.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In acceptance testing, 100% of provider-backed runs are blocked before execution if the selected provider is unregistered, inactive, unavailable, or missing the requested capability declaration.
- **SC-002**: In acceptance testing, 100% of provider-backed execution requests show an explicit permission envelope and execution bound before the provider is allowed to run.
- **SC-003**: In operator validation, at least 95% of provider-backed success and failure scenarios allow an operator to identify provider readiness, capability used, evidence references, and limitations from status or inspect within 30 seconds.
- **SC-004**: In acceptance testing, 100% of provider responses that propose state changes require a visible validation disposition before any Boundline-owned state is changed.
- **SC-005**: In acceptance testing, interrupted provider setup leaves the previously active provider configuration intact in 100% of tested scenarios.

## Assumptions

- The first slice defines the generic provider contract and operator activation surface; concrete providers such as browser, sandbox, or experimental recursive providers are follow-on work.
- Provider-backed execution may be read-only or mutation-capable, but Boundline-owned validation and stop rules remain authoritative in both cases.
- Operators need a local-first onboarding flow that can support both local executables and remote provider endpoints without trusting discovery alone.
- Canon compatibility remains a consumption boundary for governed evidence and packet semantics rather than part of provider activation or execution.
- This slice focuses on protocol definition, onboarding, bounded execution, and inspectability rather than route economics, provider benchmarking, or provider-specific UI polish.
