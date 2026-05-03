# Data Model: Domain Agent Templates

## DomainFamily

- **Purpose**: Names one supported first-party domain-template family that can
  be selected for a bounded task.
- **Values**:
  - `systems`
  - `jvm_service`
  - `dotnet_service`
  - `python_service`
  - `node_service`
  - `web_ui`
  - `react`
  - `vue`
  - `angular`
  - `ruby`
  - `php`
  - `data`
  - `mobile`
- **Validation rules**:
  - Every effective domain selection must reference a known family.
  - Built-in catalog coverage must exist for every enum value.

## DomainTemplateSettings

- **Purpose**: Captures the scoped configuration for one domain family.
- **Fields**:
  - `enabled`: optional per-scope switch that enables or disables the family.
  - `standards`: optional scoped standards overlay text.
  - `external_context_bindings`: bound external context inputs declared at the
    same scope.
- **Validation rules**:
  - Empty standards text is invalid when provided.
  - External bindings must validate individually.
  - A family can be active only when at least one effective enablement source
    resolves to true.

## ExternalContextBinding

- **Purpose**: Declares one supporting context input that a domain family can
  reuse for bounded planning or execution.
- **Fields**:
  - `kind`: design reference, design system, design tokens, platform guidance,
    API contract, or custom.
  - `reference`: repository-relative path, URL, MCP-style identifier, or other
    bounded locator string.
  - `required`: whether the current task class must have this input available.
  - `notes`: optional operator-facing explanation of when the binding matters.
- **Validation rules**:
  - `reference` must not be empty.
  - Empty notes are invalid when provided.
  - Duplicate bindings at the same scope must resolve deterministically by kind
    plus reference.

## ResolvedDomainTemplate

- **Purpose**: Represents the effective per-family result after combining the
  built-in template, shared standards, workspace overrides, and scoped external
  bindings.
- **Fields**:
  - `family`: resolved domain family.
  - `enabled`: whether the family is active for the bounded task.
  - `built_in_summary`: concise description of the first-party template.
  - `standards_layers`: ordered guidance layers with explicit source labels.
  - `external_context_bindings`: effective bound inputs with source labels.
- **Validation rules**:
  - The built-in summary is always present.
  - Standards layers preserve explicit precedence from broadest to narrowest.
  - Effective bindings must keep their optional or required flag.

## AppliedDomainContext

- **Purpose**: Records the domain guidance that shaped the current bounded task
  or planning step.
- **Fields**:
  - `families`: selected domain families in specificity order.
  - `summary`: operator-facing line naming the active families and winning
    standards source.
  - `credibility`: credible, insufficient, or stale.
  - `selected_target`: task or file target that triggered the selection.
  - `guidance_sources`: explicit source lines for built-in, global, cluster,
    and workspace standards.
  - `external_input_status`: per-binding used, unavailable, stale, or skipped
    projection.
  - `governed_artifact_refs`: optional Canon-governed artifacts reused as
    supporting input.
  - `blocking_reason`: explicit reason when the context is not credible.
- **Validation rules**:
  - Credible context must name at least one selected family.
  - Insufficient context must explain the blocking reason.
  - Required external inputs may not be marked skipped without an explicit
    fallback or downgrade explanation.

## RoutingConfig Updates

- **New responsibilities**:
  - Persist per-family domain-template settings across workspace, cluster, and
    global scopes.
  - Resolve effective enablement, standards layers, and external bindings by
    source.

## ContextPack Updates

- **New responsibilities**:
  - Carry the applied domain context alongside existing workspace, negotiation,
    trace, and Canon evidence.
  - Reflect domain-selection failures in context-pack credibility so planning
    can stop explicitly.

## TaskContext And Trace Projection Updates

- **TaskContext responsibilities**:
  - Persist the latest applied domain context when a task-owned path needs it.
- **Trace responsibilities**:
  - Record domain selection, guidance sources, external-input status, and any
    blocked-domain explanation as part of normal inspectable trace payloads.