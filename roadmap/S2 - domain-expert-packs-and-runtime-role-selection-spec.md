# Domain Expert Packs And Runtime Role Composition

## Status

Draft

## Objective

Define how Boundline discovers, installs, validates, selects, and composes
reusable domain experts, language and framework specialists, reviewer
specializations, and runtime roles while consuming Canon-governed knowledge as
input.

This specification owns the pack ecosystem and role-composition layer only.

It does not own:

- council profile policy
- voting strategies
- stop-semantics policy
- adaptive degradation or escalation
- control graduation
- advanced reasoning profiles

## 1. Product Thesis

Reusable expertise should be a first-class runtime asset.

Boundline needs a pack ecosystem so that expertise can be:

- installable
- composable
- versioned
- inspectable
- repository-adaptable

This layer turns substrate inputs into candidate runtime roles and expert
compositions.

It does not decide the governance posture that applies to those roles.

## 2. Architectural Boundary

### 2.1 Canon Owns Knowledge

Canon owns:

- governed project memory
- standards
- policies
- architectural decisions
- domain language
- domain model
- safety and verification artifacts
- review and governance evidence

Canon does NOT own:

- pack manifests
- expert discovery
- role composition
- provider routing
- council orchestration
- advanced reasoning execution

Those artifacts become runtime inputs for Boundline.

### 2.2 Boundline Owns Pack Ecosystem And Role Composition

This specification makes Boundline authoritative for:

- expert-pack discovery
- expert-pack installation and validation
- built-in expert catalog
- role metadata
- workspace override resolution
- selection precedence
- role-composition inputs
- compatibility checks
- pack and role trace projection

### 2.3 Dependency Boundary

This layer depends on:

- S1 for substrate inputs and context assembly

This layer feeds:

- S3 for static council-profile structure
- S4 for adaptive governance behavior
- S6 for advanced reasoning profiles

If this document defines voting policy or stop semantics, it is wrong.

## 3. Built-In Runtime Experts

Boundline core must ship with a minimal built-in catalog.

Initial built-in experts SHOULD include:

- Rust
- TypeScript or Node
- React
- Python
- Java
- .NET
- SQL
- Security Reviewer
- Test Strategist
- Architecture Reviewer

The built-in catalog exists to guarantee:

```text
boundline init
boundline run
```

work without requiring additional downloads.

These are pack instances and default role seeds, not governance policies.

## 4. Expert Pack Sources

### 4.1 Supported Sources

Boundline MUST support pack installation from:

- git repositories
- local folders
- archive bundles
- future registries

### 4.2 Example Runtime Configuration

```toml
[expert_packs]

[[expert_packs.sources]]
type = "git"
url = "https://github.com/apply-the/boundline-pack-rust"
version = "1.2.0"

[[expert_packs.sources]]
type = "local"
path = ".boundline/packs/internal-platform"
```

## 5. Expert Pack Structure

### 5.1 Required Structure

```text
boundline-pack-rust/

pack.toml
instructions/
  planner.md
  implementer.md
  reviewer.md
rules/
  cargo.md
  async.md
checklists/
  testing.md
```

### 5.2 Machine-Readable Manifest

Every pack MUST expose a machine-readable manifest.

```toml
[pack]
id = "rust"
kind = "language-expert"
version = "1.0.0"

[support]
languages = ["rust"]
frameworks = ["axum", "tokio", "sqlx"]

[roles]
planner = "instructions/planner.md"
implementer = "instructions/implementer.md"
reviewer = "instructions/reviewer.md"

[review_capabilities]
candidate_specializations = ["maintainability-reviewer", "test-strategist"]
security_sensitive_specializations = ["security-reviewer", "architecture-reviewer"]

[canon]
preferred_artifacts = ["architecture", "domain-model", "safety-net"]

[compatibility]
boundline = ">=0.17.0"
canon_contract_major = 1
runtime_capabilities = ["local-index", "context-pack"]
```

### 5.3 Manifest Boundary

Pack manifests may describe candidate reviewer roles, preferred Canon artifacts,
and compatibility needs.

They MUST NOT define:

- council profiles
- voting strategies
- stop-semantics transitions
- adaptive governance rules

## 6. Role Metadata And Composition

This layer defines how packs contribute role metadata.

Examples include:

- implementation expertise
- reviewer specialization
- architecture specialization
- test specialization
- migration specialization
- domain specialization

Role metadata may express:

- supported languages and frameworks
- required or preferred Canon artifacts
- compatible repository cues
- compatible runtime capabilities
- candidate reviewer specializations

This layer composes candidate roles.

It does not decide which governance posture activates them.

S2 may emit candidate reviewer capabilities.
S3 determines whether governance policy requires them operationally.

## 7. Runtime Role Selection Inputs

Boundline MUST inspect:

- repository structure
- changed or target files
- task goals
- Canon project memory
- Canon evidence
- substrate-visible risk signals
- detected technologies
- active runtime capabilities

Example task:

```text
Add OAuth2 refresh token rotation to Rust auth service
```

Candidate composition may include:

- Rust Expert
- Security Reviewer
- Architecture Reviewer
- Test Strategist

Those are candidate roles and experts.

S3 and S4 decide whether they become part of an active council.

## 8. Workspace Overrides And Selection Precedence

Repositories MAY define local overrides under:

```text
.boundline/overrides/
```

Selection precedence MUST be:

```text
Boundline Built-In Experts
→ Shared Expert Packs
→ Canon Governed Standards
→ Workspace Overrides
→ Runtime Evidence
```

Workspace overrides MUST take precedence over shared pack guidance.

This precedence chain resolves pack and role composition only.

It does not resolve governance escalation or stop behavior.

## 9. Canon Integration

Canon artifacts are runtime inputs that packs and role composition may prefer.

Examples:

- domain-language
- domain-model
- architecture
- requirements
- safety-net
- security-assessment
- review
- verification

Canon MUST NOT become:

- a plugin marketplace
- an expert runtime selector
- a provider router
- a council orchestrator
- a model execution engine

## 10. Council Composition Inputs

Expert packs may contribute inputs that help later council composition, such as:

- candidate reviewer specializations
- preferred mandatory capabilities
- preferred Canon artifacts for review readiness
- repository cues that imply additional reviewer expertise

This specification stops at candidate composition inputs.

Council profiles, quorum, adjudication, and voting strategy are owned by S3.

Runtime activation, degradation, and escalation are owned by S4.

Advanced reasoning extensions are owned by S6.

## 11. Runtime Trace Projection

Boundline MUST project into traces:

- selected pack ids
- selected role metadata
- compatibility decisions
- workspace-override provenance
- Canon-artifact preferences consumed during selection
- rejected pack candidates and why they were rejected

These surfaces MUST appear in:

- status
- next
- inspect

This layer does not project vote outcomes or adjudication state.

## 12. Compatibility

Expert Packs MUST declare:

- compatible Boundline versions
- compatible Canon contract major versions
- supported languages and frameworks
- required runtime capabilities

Boundline MUST reject or warn on incompatible packs explicitly.

## 13. Advanced Retrieval Boundary

Vector retrieval, graph models, and advanced context intelligence are not owned
by this specification.

If those capabilities are introduced later, they belong under S5 and must feed
this layer as additional inputs rather than rewriting the pack model.

## 14. Non-Goals

This specification does NOT define:

- voting algorithms
- council policies by zone
- adjudication rules
- control graduation
- adaptive degradation or escalation
- self-consistency
- multi-agent debate
- fine-tuning pipelines
- hosted model serving
- provider authentication
- distributed execution
- enterprise org charts
- mandatory vector databases
- mandatory graph databases

## 15. Acceptance Criteria

The implementation is complete when:

- Boundline can install reusable Expert Packs
- Boundline can discover and validate experts automatically
- Boundline can compose candidate roles from pack metadata
- Boundline can consume Canon project memory as role-composition input
- Workspace overrides can supersede shared pack guidance
- Runtime traces expose selected packs, roles, and compatibility decisions
- Expert Packs can declare machine-readable compatibility
- This specification no longer defines council-policy or voting-policy behavior
- Canon remains separated from runtime orchestration
