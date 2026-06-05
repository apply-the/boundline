# External Capability Provider Protocol

## Integration Update

This roadmap item remains the generic capability boundary for external systems.

**Adapter-Specialized Execution Profiles** should be added as a follow-up profile layer on top of this protocol, not as a replacement for it.

Canon must not become an adapter. Canon remains a first-class governed producer and semantic governance authority consumed by Boundline through stable contracts.

## Relationship To Other Roadmap Files

| Related file | Relationship |
|---|---|
| `specs/070-large-codebase-context-substrate/spec.md` | Owns local context substrate; provider-supplied context uses this protocol |
| `08-evals-and-runtime-observability.md` | Owns event schema and provider-call observability |
| `13-sandboxed-execution-and-secret-inheritance.md` | Enforces sandboxing, path, network, and secret policy |
| `14-ai-gateway-and-inference-economics.md` | Owns route economics and provider/model cost policy |
| `15-browser-and-visual-testing-provider.md` | Should be implemented as a concrete provider using this protocol |
| `17-experimental-recursivemas-provider-adapter.md` | Should remain an experimental external provider using this protocol |

## Canon Boundary

Canon is not an external provider for Boundline stage execution.

Canon owns:

- governed packets
- evidence and approval semantics
- lineage and provenance
- posture contracts
- project memory
- policy semantics

Boundline consumes Canon outputs through stable contracts and uses them for validation, gating, and traceability.

Adapters and providers are for external capabilities and execution support.

Examples:

- Speckit provider
- browser provider
- sandbox provider
- static-analysis provider
- RecursiveMAS experimental provider
- company harness provider

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
- session reference
- step ID
- capability
- goal
- lifecycle phase
- authority zone
- Context Pack references
- permissions
- limits
- expected outputs

Must include in response:

- status
- observations
- findings
- artifacts
- evidence references
- state patch proposals
- limitations
- next actions

### collect_evidence

Provider normalizes evidence after execution.

Must include:

- claims
- evidence references
- artifacts
- findings
- limitations
- reproducibility information

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

## Adapter-Specialized Execution Profiles

A later profile layer may define known framework semantics on top of the provider protocol.

Example: Speckit specialized profile.

```text
Boundline goal
  -> native Boundline

Boundline plan
  -> speckit.specify
  -> speckit.clarify when required
  -> speckit.plan
  -> speckit.tasks
  -> speckit.analyze
  -> remediation for blocking findings

Boundline run
  -> speckit.implement
```

Rules:

- profile is versioned
- profile maps framework operations to Boundline stage ownership
- provider still obeys protocol permissions and trace boundaries
- Boundline still owns stage state, stop semantics, and final acceptance
- provider failure after stage claim fails the stage
- profile output is structured and inspectable
- Canon remains outside provider profile semantics

## Operator Setup And Activation

Provider onboarding is a Boundline runtime concern, not Canon setup logic.

The protocol should support:

- explicit operator registration and activation
- setup requirement projection before first use
- non-secret configuration capture through interactive or config-driven flows
- secret-handle references instead of prompt-visible secret values
- connectivity or health dry-run before activation is marked ready
- atomic setup so an interrupted flow leaves previous active config intact

Hard boundaries:

- a locally discoverable executable must not auto-enable itself as a provider
- setup must not persist raw secrets in traces or tracked files
- provider activation must remain visible in status and inspect

## Acceptance Criteria Additions

- Boundline can discover provider capabilities.
- Boundline can reject unavailable providers before run.
- Boundline requires explicit operator registration before activation.
- Provider execution is permission-scoped.
- Provider output cannot directly mutate Boundline state without validation.
- Evidence packets are trace-linked.
- Provider limitations are visible in inspect.
- Specialized profiles can be layered over this protocol without making Canon an adapter.
- Generic provider protocol remains intact when profiles are absent.

## Risks

- External providers become trusted implicitly.
- Local executables become active accidentally.
- Setup leaks secrets into prompts or traces.
- Hidden provider state makes runs non-reproducible.
- Permissions are too broad.
- Protocol is too generic to validate.
- Specialized profiles bypass generic safety rules.

## Hard Rules

- Boundline owns admission control. Providers never approve themselves.
- Discoverability is not activation.
- Canon is not an adapter.
