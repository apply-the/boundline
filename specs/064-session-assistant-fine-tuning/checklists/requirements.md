# Requirements Checklist: Provider Auth, Probe Readiness, and Assistant Handoff Fine-Tuning

**Purpose**: Validate the quality, completeness, and traceability of the rewritten 064 requirements before further retrospective refinement.
**Created**: 2026-05-28
**Feature**: [spec.md](../spec.md)

**Note**: This checklist validates the written requirements in spec 064. It does not verify code behavior or replace the runtime and contract test suites.

## Requirement Completeness

- [ ] CHK001 Are the provider-auth lifecycle requirements complete for login, status, remove, persistence, and unsupported-provider handling? [Completeness, Spec §User Story 1, Spec §FR-001-006]
- [ ] CHK002 Are planning-gate requirements complete for goal quality, plan quality, backlog quality, planning analysis, and assistant-safe continuation rules? [Completeness, Spec §User Story 2, Spec §FR-007-009]
- [ ] CHK003 Are probe requirements complete for bootstrap, doctor-required, no-active-session, current-workspace resolution, and host-envelope JSON behavior? [Completeness, Spec §User Story 3, Spec §FR-010-013]
- [ ] CHK004 Are cross-host assistant requirements complete for Copilot, Claude, Codex, and Antigravity, including readiness-sensitive commands and host-specific action syntax? [Completeness, Spec §User Story 4, Spec §FR-014-018]

## Requirement Clarity

- [ ] CHK005 Is the supported-provider boundary explicit enough that readers can tell this slice only adds device-flow login for `github-copilot`? [Clarity, Spec §FR-002, Spec §Scope Boundaries]
- [ ] CHK006 Is the term `assistant-safe` defined clearly enough to distinguish runtime-reported handoffs from generic shell guidance? [Clarity, Spec §User Story 2, Spec §FR-009, Spec §Assistant Handoff Definition]
- [ ] CHK007 Is `bootstrap-only` behavior described precisely enough to prevent readers from inferring a repo-local assistant route during init-only states? [Clarity, Spec §User Story 3, Spec §FR-012, Spec §Edge Cases]
- [ ] CHK008 Are host-specific action syntax rules explicit enough to distinguish Copilot command URIs from non-Copilot `/boundline:*` actions? [Clarity, Spec §User Story 4, Spec §FR-015-016]

## Requirement Consistency

- [ ] CHK009 Do the probe user story, functional requirements, and success criteria all describe the same bootstrap, doctor, and goal-ready routing outcomes without contradiction? [Consistency, Spec §User Story 3, Spec §FR-010-013, Spec §SC-003]
- [ ] CHK010 Do the planning-gate user story, edge cases, and functional requirements agree on stop conditions and precedence for clarification-required versus blocked states? [Consistency, Spec §User Story 2, Spec §Edge Cases, Spec §FR-007-009]
- [ ] CHK011 Do the scope boundaries and assumptions stay consistent about additive auth-profile support versus existing environment-based credentials? [Consistency, Spec §Scope Boundaries, Spec §Assumptions]

## Acceptance Criteria Quality

- [ ] CHK012 Are the provider-auth acceptance scenarios measurable through observable CLI outcomes rather than subjective wording? [Acceptance Criteria, Spec §User Story 1, Spec §SC-001]
- [ ] CHK013 Are the probe acceptance scenarios specific enough to verify both plain CLI output and `--json` host-envelope behavior? [Acceptance Criteria, Spec §User Story 3, Spec §SC-003]
- [ ] CHK014 Are assistant-parity success criteria framed as observable contract outcomes rather than broad quality claims like “consistent” or “reliable” alone? [Measurability, Spec §User Story 4, Spec §SC-004-SC-006]

## Scenario Coverage

- [ ] CHK015 Are negative and recovery scenarios defined for unsupported provider login, missing stored auth, missing provider credentials, and no active session? [Coverage, Spec §User Story 1, Spec §User Story 3, Spec §Edge Cases]
- [ ] CHK016 Are readiness-sensitive assistant scenarios covered for goal, plan, status, and recover rather than only a subset of commands? [Coverage, Spec §User Story 4, Spec §FR-014]
- [ ] CHK017 Are both current-directory probe invocation and explicit `--workspace` probe invocation covered by the written requirements or assumptions? [Coverage, Spec §User Story 3, Spec §FR-010, Spec §Assumptions]
- [ ] CHK018 Are both Copilot and non-Copilot host behaviors represented in the written assistant parity requirements? [Coverage, Spec §User Story 4, Spec §FR-015-017]

## Edge Case Coverage

- [ ] CHK019 Does the spec explicitly cover secret-handling boundaries, including not printing token or API key values in auth status output? [Edge Case, Spec §Edge Cases, Spec §FR-004]
- [ ] CHK020 Does the spec define how probe path reporting should behave across macOS temp-path normalization differences strongly enough to avoid ambiguous test expectations? [Edge Case, Spec §Edge Cases, Spec §User Story 3]
- [ ] CHK021 Are prompt-syntax edge cases covered so Copilot prompts do not rely solely on `/boundline:*` text and non-Copilot assets do not emit Copilot-specific command URIs? [Edge Case, Spec §Edge Cases, Spec §FR-015-016]

## Dependencies & Assumptions

- [ ] CHK022 Are persistence assumptions explicit for the global auth profile path and the workspace-local readiness surfaces consumed by probe? [Assumption, Spec §Assumptions, Spec §AuthProfileStore, Spec §ProbeReport]
- [ ] CHK023 Are out-of-scope exclusions explicit enough to prevent readers from expecting broader OAuth-provider coverage, secret-vault integration, or assistant-architecture redesign in this slice? [Boundary, Spec §Scope Boundaries]

## Ambiguities & Conflicts

- [ ] CHK024 Is the phrase “readiness-sensitive assistant assets” mapped clearly enough to concrete command groups that reviewers can tell exactly which prompts must honor probe preflight? [Ambiguity, Spec §FR-014, Gap]
- [ ] CHK025 Do any requirements mix implementation validation and requirements intent in a way that would make retrospective acceptance ambiguous? [Conflict, Spec §Functional Requirements, Spec §Success Criteria]

## Notes

- Refresh this checklist whenever spec 064 changes scope again, especially if additional providers, prompt hosts, or probe states are added.
- This file supersedes the older generic 064 checklist that referenced session and audit fine-tuning instead of the rewritten provider-auth and probe scope.
