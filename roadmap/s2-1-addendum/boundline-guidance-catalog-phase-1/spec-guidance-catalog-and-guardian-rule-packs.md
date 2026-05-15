# Feature Specification: Guidance Catalog And Guardian Rule Packs

## Status

Draft

## Relationship To S2.1

S2.1 defines how Boundline loads, resolves, executes, traces, and exposes Guidance and Guardian capabilities.

This specification defines the first concrete content catalog for those capabilities.

It is content and packaging focused, not runtime-behavior focused.

## Outcome

Boundline gains a curated baseline catalog of engineering guidance and guardian rule seeds covering:

- clean code
- architecture
- testing
- Rust
- TypeScript/Node.js
- initial guardian rule packs

The catalog must be usable as:
- Boundline built-in capabilities
- shared expert pack content
- workspace override templates
- Canon-governed standards after promotion

## Product Thesis

S2.1 makes engineering standards executable.

This feature provides the initial standards.

Without a concrete catalog, S2.1 can load capabilities but has little useful content to apply.

## Scope

In scope:
- guidance markdown files
- guardian rule seed markdown files
- source classification
- guidance strength classification
- trace-visible authority source metadata
- pack-ready directory shape

Out of scope:
- guardian execution runtime
- council voting
- stop semantics
- adaptive governance
- model routing
- provider authentication
- semantic retrieval

## User Stories

### US1 — Use Built-In Engineering Guidance

As a Boundline user, I want built-in engineering guidance to be available immediately, so that guidance and guardian capabilities work before I install organization-specific packs.

Acceptance:
- Clean Code guidance exists.
- Architecture guidance exists.
- Testing guidance exists.
- Rust guidance exists.
- TypeScript/Node guidance exists.
- Each guidance file declares lifecycle usage and suggested guardian hooks.

### US2 — Promote Guidance Into Shared Packs

As a platform maintainer, I want the catalog to be pack-ready, so that teams can extract or version specific guidance into shared expert packs.

Acceptance:
- Guidance files are organized by pillar.
- Guardian rule seeds are separate from guidance prose.
- Pack manifests can reference each guidance file.
- Workspace overrides can replace individual files.

### US3 — Use Guidance As Canon-Governed Standard

As a governance owner, I want these guidance files to be promotable into Canon, so that the same content can become governed project memory or organization standard.

Acceptance:
- Files do not hard-code Boundline runtime internals.
- Files can be referenced as Canon-governed standards.
- Authority source can be recorded at resolution time.

### US4 — Seed Guardian Rules

As a Boundline operator, I want initial guardian rule seeds, so that guardians produce structured findings based on concrete rules rather than generic review prose.

Acceptance:
- Rule seed file defines initial guardians.
- Rule seed file defines required finding contract.
- Rule seed file distinguishes deterministic, LLM, and hybrid suitability.

## Functional Requirements

FR-001: System MUST provide a guidance catalog organized by pillar.

FR-002: System MUST include guidance for clean code, architecture, testing, Rust, and TypeScript/Node.js in the initial catalog.

FR-003: Each guidance file MUST describe purpose, authority classification, core principles, guardian hooks, finding examples, and lifecycle usage.

FR-004: System MUST provide guardian rule seeds separate from guidance prose.

FR-005: Guardian rule seeds MUST identify candidate guardian kind: deterministic, llm, or hybrid.

FR-006: Guidance files MUST be usable from built-ins, shared packs, workspace overrides, or Canon-governed standards without rewriting content.

FR-007: Guidance entries MUST support strength classification: recommendation, concern, warning, blocker, target-excellence, legacy-warning, anti-pattern, deprecated.

FR-008: Guardian rule seeds MUST map rules to structured finding output.

FR-009: Catalog files MUST avoid treating time-sensitive framework preferences as universal hard rules unless authority source promotes them.

FR-010: Language and framework guidance MUST be split into focused files rather than stored only in one monolithic report.

## Success Criteria

SC-001: A user can run a session with no external packs and still get useful guidance and guardian rule candidates.

SC-002: A maintainer can copy one guidance file into a workspace override without copying the whole catalog.

SC-003: A Canon governance owner can promote one guidance file as a governed standard without depending on Boundline runtime internals.

SC-004: Guardian rule seeds can produce structured findings that cite the relevant guidance file.

SC-005: The initial catalog covers at least one systems language, one web language, architecture, testing, and clean-code principles.

## Initial File Set

Required files:

```text
guidance/clean-code.md
guidance/architecture.md
guidance/testing-core.md
guidance/language-rust.md
guidance/language-typescript-node.md
guardians/guardian-rule-seeds.md
```

Future files:

```text
guidance/language-go.md
guidance/language-python.md
guidance/language-jvm.md
guidance/language-dotnet.md
guidance/framework-react.md
guidance/framework-python-services.md
guidance/framework-jvm-services.md
guidance/security-boundaries.md
guidance/domain-modeling.md
```

## Non-Goals

This specification does not:
- implement guardian execution
- define review councils
- define governance escalation
- define vector retrieval
- define model/provider catalog updates
- define all languages and frameworks in one release

## Final Thesis

S2.1 defines the mechanism.

This catalog defines the first useful content.

The correct shape is:

```text
guidance prose
+
guardian rule seeds
+
authority metadata
+
pack-ready layout
```

not a single unstructured best-practices document.
