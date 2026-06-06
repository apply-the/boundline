# Feature Specification: Review Councils And Role-Gated Governance

**Feature Branch**: `074-review-councils-governance`

**Created**: 2026-06-06

**Status**: Draft

**Input**: User description: "Guardian Activation Router + Review Councils for adjudicating guardian findings with role-gated escalation."

## Clarifications

### Session 2026-06-06

- Q: How are guardian activation rules defined — hardcoded, TOML config, or self-declared? → A: Versioned TOML ruleset under `.boundline/guardian-rules.toml` mapping stage conditions, file patterns, risk, and language/framework hints to guardian activation lists. Boundline ships safe built-in defaults; missing file uses defaults; invalid file fails closed. The activation trace records which rule matched, which guardians activated, and which mandatory guardians were skipped or unavailable.
- Q: What is the council voting and decision model for V1? → A: Single-reviewer adjudication. One adjudicator reviews all activated guardian findings, records rationale, preserves dissenting/unresolved findings, and produces binary outcome (clean/blocked). Mandatory guardian unavailability forces blocked. Multi-member voting deferred to later slice.
- Q: How is council adjudication invoked — CLI command or automatic hook? → A: CLI command (`boundline council adjudicate`) is the default. Operator runs it after guardians produce findings. Automatic hook may be added later as a configurable option but must use the same adjudication contract and remain trace-visible.
- Q: What happens when no guardians are activated, or when rules produce contradictory activation? → A: Zero-guardian (no mandatory rule applies) reports clean with a trace-visible note. Contradictory rules (same guardian both activate and skip for same matched condition) cause the ruleset to be rejected as invalid at load time. The validation output explains the conflicting rule IDs, affected guardian, matched condition, and required remediation.
- Q: What happens when a mandatory guardian produces no findings, or the council has zero members? → A: Mandatory guardian with zero findings is a pass ONLY if it also emits a successful execution record. Missing execution record, unavailable guardian, or malformed output = blocked. Zero-member council profile is invalid (fail closed); if no profile is configured at all, Boundline uses the built-in V1 single-adjudicator default with a trace-visible note.

### User Story 1 - Guardian Activation Router Determines Which Guardians Run (Priority: P1)

An operator initiates a stage (plan, run, review) and the router determines which guardians should activate based on the change surface: files touched, language, framework, risk classification, authority zone, active contracts, and guidance pillars. Mandatory guardians cannot be skipped; skipped guardians produce inspectable reasons.

**Why this priority**: The router is the prerequisite — without it, councils have no structured input to adjudicate.

**Independent Test**: Stage a Rust runtime change, run the router, confirm that `rust-guardian`, `error-handling-guardian`, and `traceability-guardian` are activated while irrelevant guardians are skipped with reasons.

**Acceptance Scenarios**:

1. **Given** a Rust runtime change (`src/domain/**/*.rs`, `src/orchestrator/**/*.rs`), **When** the router evaluates the change surface, **Then** mandatory Rust-specific guardians activate and documentation-only guardians are skipped with a reason.
2. **Given** a documentation-only change (`docs/**/*.md`), **When** the router evaluates, **Then** only `docs-consistency-guardian` and `release-surface-guardian` activate; runtime guardians are skipped.
3. **Given** a security-sensitive change with `risk: high`, **When** the router evaluates, **Then** `security-guardian`, `threat-model-guardian`, and `approval-gate-guardian` are activated as mandatory.
4. **Given** a mandatory guardian is skipped (missing capability), **When** the router completes, **Then** a missing-guardian-capability finding is emitted as a blocking diagnostic.

---

### User Story 2 - Council Adjudicates Guardian Findings (Priority: P2)

After guardians run, a review council groups findings, applies voting or adjudication policy, records dissent, and produces a final decision. The council trace includes authority zone, active profile, activation plan, findings, and decision.

**Why this priority**: Guardian findings without adjudication are noise. A council turns raw findings into actionable decisions.

**Independent Test**: Run guardians against a change that produces both blocking and warning findings, then adjudicate via a single-reviewer council, confirming the final decision reflects the blocking finding.

**Acceptance Scenarios**:

1. **Given** guardian findings include one blocker and two warnings, **When** a single-reviewer council adjudicates, **Then** the council decision is `blocked` with the blocker as the primary reason.
2. **Given** guardian findings with dissent recorded, **When** the council produces its trace, **Then** the trace includes the dissenting position, the majority decision, and the final outcome.

---

### Edge Cases

- What happens when no guardians are activated for a stage (trivial change)? → Resolved: Report clean with a trace-visible note, provided no mandatory guardian rule applies.
- What happens when a mandatory guardian produces no findings (silent pass)? → Resolved: Pass only if the guardian also emitted a successful execution record. Missing execution record, unavailable guardian, or malformed output = blocked.
- How does the system handle a council with zero members configured? → Resolved: Explicit zero-member profile is invalid (fail closed). No profile configured at all → built-in V1 single-adjudicator default with trace-visible note.
- What happens when a guardian activation rule produces contradictory results (both activate and skip)? → Resolved: Ruleset rejected as invalid at load time with conflict explanation.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST provide a guardian activation router that evaluates the change surface against a versioned TOML ruleset (`.boundline/guardian-rules.toml`) and produces an activation plan. The system MUST ship safe built-in default rules and fall back to them when the ruleset file is missing. An invalid ruleset file (including contradictory activation/skip rules for the same guardian under the same matched condition) MUST cause the router to fail closed, with a validation output explaining the conflicting rule IDs, affected guardian, matched condition, and required remediation.
- **FR-002**: The router MUST distinguish mandatory guardians from optional guardians; mandatory guardians MUST NOT be silently skipped. A mandatory guardian is considered passed only if it emits a successful execution record — zero findings is acceptable WITH an execution record, but missing execution record, unavailable guardian, malformed output, or skipped mandatory guardian MUST result in a blocked outcome.
- **FR-003**: Every skipped guardian MUST produce an inspectable reason in the activation plan trace.
- **FR-004**: The system MUST emit a `guardian.activation.plan.produced` structured event recording the activation plan, activated guardians, skipped guardians with reasons, and any missing capability findings.
- **FR-005**: The system MUST support four predefined routing rules at minimum: Rust runtime change, documentation-only change, contract change, and security-sensitive change.
- **FR-006**: The system MUST provide a single-reviewer review council that examines all activated guardian findings, records adjudication rationale (accepted, rejected, deferred), preserves dissenting findings, and produces a binary decision (clean or blocked). If any mandatory guardian is unavailable or skipped, the conservative outcome MUST be blocked.
- **FR-007**: Council traces MUST include: adjudicator role, authority zone, guardian activation plan, findings reviewed, accepted findings, rejected findings with rationale, deferred findings with reason, dissent record, and final decision.
- **FR-008**: The system MUST emit a `council.decision.produced` structured event with the council outcome, primary finding, dissent status, and decision metadata.
- **FR-009**: The system MUST provide a `boundline council adjudicate` CLI command that reads the latest guardian activation and findings state, applies the single-adjudicator decision model, and produces a trace-visible council outcome. Automatic adjudication hooks are deferred to a later slice and MUST use the same adjudication contract if introduced.

### Key Entities

- **GuardianActivationPlan**: The router output listing activated guardians, skipped guardians with reasons, mandatory vs optional classification, escalation recommendations, and missing-capability findings.
- **CouncilProfile**: Configuration defining the review council's authority zone, member roles, voting policy, and decision thresholds.
- **CouncilDecision**: The adjudicated outcome including the primary finding, dissent record, vote tally, human gate state, and final decision.

## Success Criteria

- **SC-001**: The guardian activation router produces a correct activation plan for all four predefined routing rules across validated regression fixtures.
- **SC-002**: Mandatory guardians can never be silently skipped — 100% of skipped mandatory guardians produce an inspectable finding.
- **SC-003**: Council decisions are trace-visible with complete finding context within 1 second of adjudication completion.

## Assumptions

- Guardian implementations already exist (rust-guardian, error-handling-guardian, etc.); this feature adds routing and adjudication logic only.
- The change surface data is available from the existing session and file-system inspection surfaces.
- Council profiles are configured via `.boundline/` configuration, not a new UI.
