# Feature Specification: Reasoning Profile Closure

**Feature Branch**: `062-reasoning-profile-closure`  
**Created**: 2026-05-18  
**Status**: Draft  
**Input**: User description: "Close all residual S6.1 reasoning profile work by fully shipping or explicitly narrowing independent_pair_review, heterogeneous_security_review, bounded_reflexion, debate, and adjudication; align runtime, status, inspect, traces, validation reports, roadmap, documentation, changelog, and release/version updates; resolve the remaining maintainability cognitive complexity issues in session and session runtime surfaces; and finish the full validation bar with tests, clippy, and lcov coverage."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.
  When both a session-native workflow and a compatibility workflow exist, the spec MUST name which path is primary and keep compatibility behavior explicit rather than implicit.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Close Concrete Residual Profiles (Priority: P1)

A developer using Boundline's primary session-native workflow can trigger the residual concrete S6 profiles that were left partially closed in `061`, and each claimed shipped profile either completes with positive-path evidence or stops with an explicit bounded degraded, blocked, or interrupted result.

**Why this priority**: This is the core unfinished delivery value from S6.1. Without credible end-to-end closure for `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion`, Boundline still overstates what is concretely shipped.

**Independent Test**: Can be fully tested by running representative session-native delivery scenarios that activate `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion`, then verifying that `run`, `status`, `inspect`, and trace outputs expose a credible positive path and at least one bounded non-success path for each profile.

**Acceptance Scenarios**:

1. **Given** a stage whose governance or reasoning posture requires blind double-check, **When** the developer runs the normal workflow with distinct reviewer routes available, **Then** `independent_pair_review` activates, reaches a converged or adjudicated terminal result, and records aligned `status`, `inspect`, confidence, and trace evidence.
2. **Given** a stage whose reasoning posture requires heterogeneous challenge, **When** the workflow runs with supported heterogeneous participants, **Then** `heterogeneous_security_review` activates, completes with explicit confidence and next-action output, and remains inspectable through the normal session surfaces.
3. **Given** a stage whose policy requests bounded reflexion but the critique or revision path stalls or is interrupted, **When** the workflow reaches the profile budget or stop condition, **Then** `bounded_reflexion` ends in an explicit degraded, blocked, or interrupted result instead of silently collapsing into generic failure handling.

---

### User Story 2 - Make Debate And Adjudication Claims Honest (Priority: P2)

An operator or maintainer can tell that debate is shipped only as bounded substrate and adjudication only as a shared primitive, and every runtime, trace, inspect, roadmap, and validation surface agrees with that classification.

**Why this priority**: S6.1 is not complete until profile claims are precise. Debate and adjudication currently exist as substrate and vocabulary, but their shipped status is still ambiguous.

**Independent Test**: Can be fully tested by running the representative bounded-substrate and shared-primitive evidence paths, then confirming that `status`, `inspect`, contracts, roadmap, and validation artifacts never imply standalone shipped profiles for debate or adjudication.

**Acceptance Scenarios**:

1. **Given** an operator inspects the runtime and release-facing artifacts, **When** debate-related bounded reasoning appears in traces or summaries, **Then** no surface claims debate as a shipped standalone profile and the remaining supported behavior is described explicitly as bounded substrate.
2. **Given** a concrete profile reaches disagreement resolution, **When** the developer inspects the resulting output and trace vocabulary, **Then** adjudication is surfaced only as a shared primitive used by that profile and never as a standalone shipped profile.

---

### User Story 3 - Ship A Release-Ready Closure Slice (Priority: P3)

A maintainer can cut a release-ready closure slice whose versions, changelogs, docs, validation reports, and maintainability gates all match the final shipped S6 reasoning-profile claims in Boundline and the required Canon companion publication.

**Why this priority**: The residual work is not done when runtime code exists but release-facing claims, version windows, and quality gates still disagree or fail.

**Independent Test**: Can be fully tested by reviewing the release candidate artifacts and running the repository validation bar so that the updated versions, changelogs, docs, lcov report, test suite, clippy, and maintainability gates all agree on the shipped closure set.

**Acceptance Scenarios**:

1. **Given** the closure work changes Boundline's shipped behavior, **When** the maintainer prepares the release candidate, **Then** the Boundline version, changelog, docs, roadmap, contracts, and validation report all describe the same shipped profile set.
2. **Given** this closure publishes a new supported Boundline release pair, **When** the release candidate is prepared, **Then** Canon companion contract, version-window, changelog, and test artifacts are updated to the matching published pair without adding new Canon runtime behavior.
3. **Given** the closure work touches the session validation and reasoning-independence surfaces that currently trigger critical maintainability findings, **When** the maintainer runs the repository and CI quality gates, **Then** no release-blocking cognitive-complexity finding remains in those touched surfaces.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a claimed shipped profile can still activate only through fixture wiring or trace-only vocabulary rather than the real session-native runtime?
- How does the system handle reasoning-profile executions that reach reviewer, branch, debate, reflexion, or adjudication limits without credible convergence?
- How does the system surface the difference between a shipped standalone profile and a shared primitive when the same trace vocabulary is reused by multiple profiles?
- What happens when the sibling Canon repository is unavailable during Boundline-local compatibility validation?
- What happens when Boundline docs or roadmap language overstate debate or adjudication compared with the final runtime classification?
- What happens when the profile-closure implementation passes behavior tests but still fails the repository maintainability threshold for touched session or reasoning helpers?
- How does the system keep the primary session-native path explicit while preserving any necessary compatibility validation path?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST keep the session-native workflow as the primary operator story while closing residual reasoning-profile behavior inside the existing session and governance loop.
- **FR-002**: System MUST provide end-to-end runtime activation evidence for `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion`, including at least one credible positive-path scenario for each profile still claimed as shipped.
- **FR-003**: System MUST provide at least one explicit degraded, blocked, or interrupted scenario for each residual concrete profile where bounded non-success handling materially affects delivery credibility.
- **FR-004**: System MUST expose profile trigger, participant topology, disagreement or convergence state, confidence contribution, and next action consistently across `run`, `status`, `inspect`, and trace outputs.
- **FR-005**: System MUST classify debate explicitly as bounded substrate carried by shared primitive behavior, and every runtime, contract, roadmap, validation, and documentation surface MUST use that same classification.
- **FR-006**: System MUST classify adjudication explicitly as a shared primitive rather than a shipped standalone profile, and every runtime, contract, roadmap, validation, and documentation surface MUST use that same classification.
- **FR-007**: System MUST keep the runtime contract, validation report, roadmap, changelog, release-facing docs, and compatibility documentation aligned with the final shipped profile claims.
- **FR-008**: System MUST preserve explicit execution limits and terminal outcomes for every reasoning-profile activation, including budget exhaustion, interruption, blocked independence, and explicit operator stop conditions.
- **FR-009**: System MUST preserve Boundline-local compatibility validation even when the sibling Canon repository is unavailable by using repo-local compatibility artifacts where needed.
- **FR-010**: System MUST remain independently executable in Boundline and MUST NOT reopen the Canon posture boundary or depend on new Canon runtime control flow to close S6.1; the only Canon changes allowed are companion publication, version-window, changelog, and contract-test alignment for the new release pair.
- **FR-011**: System MUST resolve release-blocking maintainability findings in the touched session validation and reasoning-independence surfaces so local validation passes and the existing CI quality workflow no longer reports those cognitive-complexity issues, without suppressing the rule.
- **FR-012**: System MUST update the published release artifacts and compatibility windows for every repository whose published supported pair changes; for this closure slice that includes Boundline and the Canon companion publication surfaces.
- **FR-013**: System MUST complete the release validation bar for the closure slice with passing tests, clippy, and refreshed `lcov.info` coverage evidence before the work is considered done.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: closing the remaining first-wave S6 profile claims; adding missing positive-path and bounded non-success reasoning evidence; deciding and documenting debate and adjudication classification; aligning Boundline runtime, trace, inspect, contracts, roadmap, docs, changelog, versions, and validation outputs; fixing the release-blocking maintainability findings tied to the closure slice.
- **Out of Scope**: reopening the Canon challenge-posture boundary; introducing a second orchestration system; unbounded debate or swarm execution; UI work; new long-term memory systems; speculative future reasoning profiles beyond the audited S6.1 carry-forward; and Canon runtime changes beyond the required companion publication and version-alignment updates.

### Key Entities *(include if feature involves data)*

- **Profile Closure Claim**: The explicit classification for each residual S6 reasoning capability, including whether it is a shipped standalone profile, a shared primitive, or deferred work.
- **Profile Execution Evidence**: The runtime-visible activation, terminal result, operator projection, and trace story that proves a shipped profile claim is real and bounded.
- **Compatibility Artifact**: The repo-local or cross-repo contract material used to prove Boundline and Canon version-window and vocabulary alignment without assuming both repositories are always present together.
- **Release Alignment Record**: The set of roadmap, changelog, validation, and documentation statements that must stay synchronized with the final shipped profile set.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: Representative session-native scenarios for `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion` each reach an explicit positive-path or bounded non-success terminal result with aligned `run`, `status`, `inspect`, and trace evidence.
- **SC-002**: 100% of representative insufficient-independence, interruption, and budget-exhaustion scenarios stop, degrade, or block explicitly rather than silently falling back.
- **SC-003**: A maintainer can determine in under 2 minutes from repository artifacts that debate is bounded substrate and adjudication is a shared primitive, and all checked artifacts agree with that answer.
- **SC-004**: The release candidate passes the required test, clippy, and `lcov.info` validation suite, and the existing CI quality workflow no longer reports the touched session or reasoning-closure functions as release-blocking cognitive-complexity findings.
- **SC-005**: Boundline and Canon publish aligned version, changelog, and compatibility updates for the released closure pair, while Boundline still validates the same pair from repo-local artifacts when the sibling Canon repository is unavailable.

## Catalog Research & Currency *(mandatory)*

<!--
  ACTION REQUIRED: Every Boundline spec MUST verify the bundled assistant model catalog
  against current public provider documentation using web research.
  Record what sources were checked and whether the catalog changed.
-->

- **Public Sources Reviewed**: `https://developers.openai.com/api/docs/models`, `https://platform.claude.com/docs/en/docs/about-claude/models`, `https://ai.google.dev/gemini-api/docs/models`
- **Catalog Delta**: No bundled catalog change is required for this feature specification.
- **No-Change Rationale**: The bundled catalog in `assistant/catalog/model-catalog.toml` still matches the public coding and reasoning model lines relevant to Boundline routing: OpenAI continues to publish `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano`; Anthropic continues to publish `Claude Opus 4.7`, `Claude Sonnet 4.6`, and `Claude Haiku 4.5`; and Google continues to publish `Gemini 2.5 Pro`, `Gemini 2.5 Flash`, `Gemini 2.5 Flash-Lite`, `Gemini 3.1 Pro Preview`, `Gemini 3 Flash Preview`, and `Gemini 3.1 Flash-Lite` stable and preview lines. Newly listed audio, media, deep-research, and specialized task models remain outside the bundled runtime-slot catalog for this delivery slice.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- The `061-reasoning-profile-contracts` slice remains the baseline and this feature closes its audited carry-forward rather than redefining the first release contract boundary.
- Canon companion work is required for this closure because the published supported release pair changes, but it remains limited to version anchors, published compatibility docs, changelog entries, and contract-test alignment rather than new Canon runtime behavior.
- The release bar for this feature includes repository quality findings that are blocking the touched session and reasoning surfaces, not only behavioral correctness.
- Existing session-native operator surfaces (`run`, `status`, `inspect`, trace output, roadmap, validation report, and release docs) are the authoritative places where shipped-profile claims must agree.
