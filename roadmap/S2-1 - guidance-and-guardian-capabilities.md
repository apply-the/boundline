
# Specification S2.1 — Guidance And Guardian Capabilities

## Status

Partially Implemented in Boundline 0.54.0

## Scope

Boundline runtime capability layer

Extends:
- S1 Runtime Intelligence Foundation
- S2 Domain Expert Packs And Runtime Role Selection

Consumed by:
- S3 Authority-Zoned Delivery Roles, Personas, and Review Councils
- S4 Control Graduation And Adaptive Governance

## Current Delivery Snapshot

The initial `0.54.0` slice ships the core runtime loop from this specification:

- guidance and guardian assets are repository-managed under `assistant/guidance/`, `assistant/guardians/`, and `assistant/packs/`
- bundled packs now combine shared engineering guidance with technology-clustered language, framework, and testing packs instead of a Rust-only surface
- runtime resolution uses explicit precedence: workspace overrides, optional Canon guidance, shared packs, then built-ins
- `plan` persists loaded and skipped source provenance for the resolved phase
- `run`, `status`, and `inspect` persist and project guardian timelines, structured findings, degradations, and blocking outcomes
- semantic guardians reuse the existing planning, implementation, verification, and review routes instead of introducing a guardian-specific slot

The broader design space in this document still applies to future guardian
families and engineering pillars, but the first shipped slice is intentionally
bounded around explicit source resolution and post-action verification.

---

# 1. Purpose

This specification introduces two foundational runtime capability types:

- Guidance
- Guardians

The goal is to make engineering principles, architectural constraints, language practices, testing expectations, and domain invariants operationally usable during:

- planning
- architecture
- implementation
- testing
- review
- verification
- refactoring
- migration

This specification exists because static documentation alone is insufficient for AI-assisted engineering systems.

Engineering guidance must become executable runtime capability.

---

# 2. Core Thesis

AI-assisted delivery systems fail when engineering standards exist only as human-readable documents.

Boundline must support:

guidance that shapes work before action

and:

guardians that validate work after action

This creates a bounded feedback loop:

observe
→ guide
→ act
→ verify
→ emit findings
→ govern

without turning every engineering principle into a separate autonomous agent.

---

# 3. Terminology

## 3.1 Expert

A runtime role that helps produce work.

Examples:
- planner
- implementer
- reviewer
- tester
- architect

Experts may consume guidance.

Experts may invoke guardians.

Experts are not guardians.

---

## 3.2 Guidance

Structured engineering knowledge intended to shape work before or during execution.

Guidance influences behavior.

Guidance does not enforce.

---

## 3.3 Guardian

A verification capability that checks whether work respects principles, patterns, rules, boundaries, or invariants.

Guardians emit structured findings.

Guardians do not autonomously modify production code.

Guardians may:
- advise
- warn
- challenge
- escalate
- block

depending on governance posture.

---

# 4. Design Principles

## 4.1 Guidance Must Be Operational

Guidance cannot be passive documentation only.

Every guidance capability must declare:
- where it applies
- when it applies
- who consumes it
- how it influences execution
- how compliance may later be checked

---

## 4.2 Guardians Must Be Explainable

Every guardian finding must explain:
- what rule or principle triggered
- why it matters
- where evidence exists
- how to resolve the concern
- confidence level

---

## 4.3 Deterministic Before LLM

When possible:
- static analysis
- AST analysis
- dependency analysis
- linting
- contract checks
- architecture tests
- policy engines

should be preferred over purely semantic LLM judgment.

LLM reasoning should augment—not replace—deterministic verification.

---

## 4.4 No Agent Explosion

Boundline must avoid:
- one agent per principle
- one agent per pattern
- uncontrolled swarms
- review theater

Correct:
- SOLID Guardian

Incorrect:
- Single Responsibility Agent
- Open Closed Agent
- Liskov Agent

---

# 5. Guidance Capability Model

## 5.1 Guidance Manifest

Guidance is declared in `pack.toml` or discovered in workspace overrides.

Example (Code Design Pack):
```toml
[guidance.solid]
title = "SOLID Design Principles"
applies_to = ["architecture", "implementation", "review"]
roles = ["architect", "implementer"]
path = "guidance/solid.md"
priority = "high"
```

Example (Testing Pack):
```toml
[guidance.first_testing]
title = "F.I.R.S.T. Testing Principles"
applies_to = ["testing", "review"]
roles = ["implementer", "tester"]
path = "guidance/testing-principles.md"
```

---

# 6. Guardian Capability Model

## 6.1 Guardian Manifest

Example (Semantic Design Guardian):
```toml
[guardians.pragmatic_mantras]
title = "DRY/KISS/YAGNI Guardian"
kind = "llm"
applies_to = ["architecture", "implementation", "review"]
rules = ["dry", "kiss", "yagni"]
severity_floor = "concern"
instruction = "Identify over-engineering, unnecessary complexity, or blatant duplication."
```

Example (Deterministic Architecture Guardian):
```toml
[guardians.fitness_functions]
title = "Architectural Fitness Functions"
kind = "deterministic"
applies_to = ["architecture", "review"]
command = "scripts/check-fitness.sh"
severity_floor = "error"
```

---

## 6.2 Guardian Kinds

### deterministic

Uses static analysis, AST rules, dependency rules, policy engines, architecture tests, shell scripts / binaries.

### llm

Uses semantic reasoning via LLM prompts to evaluate abstract principles (e.g., "Is this class violating SRP?").

### hybrid

Combines deterministic evidence (e.g., LOC count, cyclomatic complexity) with semantic reasoning (LLM explains why the complexity violates KISS).

---

# 7. Custom Injection Mechanism

Boundline supports injecting custom rules without modifying shared packs.

## 7.1 Local Workspace Injection

Boundline discovers custom guidance and guardians in the repository:

- `.boundline/guidance/` — `.md` files containing instructions.
- `.boundline/guardians/` — `.toml` files defining guardian capabilities.

## 7.2 Injection Precedence

Selection precedence for capabilities:

1. **Workspace Overrides** (`.boundline/guidance/`, `.boundline/guardians/`)
2. **Canon Governed Standards** (Artifacts from Canon)
3. **Shared Expert Packs** (Installed packs)
4. **Boundline Built-In Capabilities**

---

# 8. Operationalizing Engineering Pillars

Boundline operationalizes core engineering principles as follows:

## 8.1 Software Design, Clean Code & Coding Standards
- **Guidance:** Documentation derived from Canon's `clean-code-guidelines.md` covering **SOLID**, **DRY**, **KISS**, **YAGNI**, **SoC**, **Encapsulation**, **Composition over Inheritance**, **Orthogonality**, and the **30 Internal Policy Rules**.
- **Guardians:** 
  - `solid-guardian` (LLM/Hybrid): Checks for SRP, OCP, LSP, ISP, DIP violations and "God Object" or "Feature Envy" anti-patterns.
  - `pragmatic-guardian` (LLM): Flags YAGNI (over-engineering), KISS (unnecessary complexity), and "Wrong Abstraction" violations.
  - `clean-naming-guardian` (LLM): Enforces intentional, domain-coherent, verb+object naming conventions, rejecting generic terms (`data`, `manager`, `process`).
  - `magic-value-guardian` (Deterministic): Detects magic numbers and strings, enforcing enums/unions for closed states.
  - `orthogonality-guardian` (Deterministic/Hybrid): Detects boundary leakage ("Shotgun Surgery") where local changes cause remote breakage.

## 8.2 Domain Modeling & State Safety
- **Guidance:** Guidelines on modeling constraints, avoiding invalid states, and separating DTOs from domain models.
- **Guardians:**
  - `primitive-obsession-guardian` (Semantic): Flags the use of raw strings/ints where Value Objects (`EmailAddress`, `Money`) should be used.
  - `state-safety-guardian` (Hybrid): Detects "Boolean blindness" or structures that allow impossible states, recommending discriminated unions.
  - `boundary-leak-guardian` (Semantic): Prevents framework/infrastructure annotations (e.g., ORM tags) from polluting core domain models.

## 8.3 Error Handling & Resource Management
- **Guidance:** Rules for distinguishing bugs from expected/infrastructure errors, adding context, and deterministic cleanup.
- **Guardians:**
  - `error-policy-guardian` (Hybrid): Flags generic errors, swallowed exceptions, missing context, and inappropriate use of `panic`/`exit`/`fatal` outside of main/tests.
  - `resource-cleanup-guardian` (Deterministic): Verifies that resource acquisition is adjacent to language-idiomatic cleanup (e.g., `Drop`, `defer`, `try-with-resources`).

## 8.4 Language & Framework Idioms
- **Guidance:** Dynamic loading of rules from:
  - `language-best-practices/` (e.g., `rust-guidelines.md`, `typescript-guidelines.md`, `go-guidelines.md`).
  - `framework-best-practices/` (e.g., `react-guidelines.md`, `spring-guidelines.md`, `nextjs-guidelines.md`, `frontend-state-guidelines.md`).
- **Guardians:**
  - `language-idiom-guardian` (LLM): Evaluates code against language-specific idioms (e.g., structured concurrency, memory ownership).
  - `framework-guardian` (LLM): Verifies conformance to framework-specific patterns (e.g., React hook rules, Spring dependency injection patterns, Next.js server/client component boundaries).

## 8.5 Testing, QA & Framework Best Practices
- **Guidance:** Documentation derived from:
  - **Core Principles:** **F.I.R.S.T.**, **AAA Pattern**, **DAMP**, **Boundary Value Analysis**, and **Equivalence Partitioning**.
  - **Strategy:** `testing-strategy-guidelines.md` and `test-data-guidelines.md`.
  - **Frameworks:** Dynamic loading from `testing-framework-best-practices/` (e.g., `jest-guidelines.md`, `pytest-guidelines.md`, `rust-testing-guidelines.md`, `playwright-guidelines.md`, `contract-testing-guidelines.md`).
- **Guardians:**
  - `test-intent-guardian` (LLM): Verifies that tests focus on **behavior** rather than implementation details.
  - `test-framework-guardian` (LLM): Ensures correct usage of specific testing tools (e.g., proper Jest mocking, Pytest fixtures, Testcontainers lifecycle, Playwright locator patterns).
  - `first-guardian` (Hybrid): Checks for test speed (Deterministic) and independence (Semantic).
  - `qa-process-guardian` (Semantic): Ensures **Shift-Left** alignment and **Risk-Based Testing** coverage.

## 8.3 UX Design
- **Guidance:** **Consistency**, **System Status Visibility**, **Error Prevention**, **Visual Hierarchy**, and **Recognition over Recall**.
- **Guardians:**
  - `ux-heuristic-guardian` (LLM): Critiques UI implementation or specs against established UX principles.

## 8.4 Architecture
- **Guidance:** **Low Coupling/High Cohesion**, **Resilience Patterns** (Circuit Breaker, Bulkhead), **CAP Theorem** trade-offs, **Data Sovereignty**, **Hexagonal Architecture**, and **Observability**.
- **Guardians:**
  - `resilience-guardian` (Semantic): Checks if the architecture accounts for failures.
  - `coupling-guardian` (Deterministic): Verifies low coupling via dependency analysis.
  - `fitness-function-guardian` (Deterministic): Executes automated architectural integrity checks.
  - `observability-guardian` (Semantic): Ensures logging, tracing, and metrics are architected into the solution.

---

# 9. Structured Findings

Example:

```json
{
  "guardian": "solid",
  "rule": "srp",
  "disposition": "concern",
  "summary": "The service mixes persistence and policy evaluation.",
  "evidence_refs": [
    "src/auth/token_service.rs"
  ],
  "confidence": 0.81,
  "recommended_action": "Separate policy decisions from persistence."
}
```

---

# 10. Lifecycle Integration

## Planning
Guidance influences decomposition and sequencing. Guardians like `yagni-guardian` critique plans for over-engineering.

## Architecture
Guardians verify dependency direction, resilience patterns, and hexagonal boundary integrity.

## Implementation
Guidance shapes coding style and AAA test patterns. Guardians verify SOLID and DRY conformance.

## Testing
Guardians like `test-intent-guardian` ensure tests aren't brittle by verifying they test behavior, not implementation.

---

# 11. Integration With S3 & S4

- **S3 (Councils):** Determines which Guardians are mandatory for a specific "Zone" (e.g., Security Zone requires `security-boundary-guardian`).
- **S4 (Governance):** Uses Guardian findings to trigger trust degradation or escalation (e.g., multiple SOLID violations block automated merge).

---

# 12. Recommended Initial Guardians

## Architecture
- `architecture-boundary`
- `dependency-direction`
- `circular-dependency`
- `resilience-check` (Circuit Breakers/Bulkheads)
- `fitness-functions`

## Design & Code Quality
- `solid`
- `pragmatic` (KISS/YAGNI/DRY)
- `complexity` (Cognitive/Cyclomatic)
- `naming-conventions`
- `maintainability`

## Testing & QA
- `test-intent` (Behavior vs Implementation)
- `testability`
- `flaky-patterns`
- `coverage-debt`
- `risk-based-coverage`

## UX & Surface
- `ux-heuristics`
- `accessibility`

## Domain & Logic
- `domain-language-drift`
- `invariant-preservation`
- `business-rule-conformance`

## Security
- `security-boundary`
- `secret-leak`
- `dependency-vulnerability`

---

# 13. Acceptance Criteria

Implementation is complete when:
- Packs may declare Guidance/Guardians for SOLID, Testing, UX, and Architecture pillars.
- **Workspace overrides for all engineering pillars are supported.**
- Semantic (LLM) guardians can evaluate abstract principles like SRP or YAGNI.
- Deterministic (Script/Linter) guardians can evaluate metrics and boundaries.
- **Fitness Functions** can be registered as architectural guardians.
- Multi-language support is verified across all pillars.

---

# 14. Final Thesis

Engineering standards become useful to AI systems only when they are operationalized.

Guidance shapes behavior before execution.
Guardians verify behavior after execution.

Together they create bounded engineering intelligence.
