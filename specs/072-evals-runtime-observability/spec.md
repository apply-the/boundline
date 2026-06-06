# Feature Specification: Evals And Runtime Observability

**Feature Branch**: `072-evals-runtime-observability`

**Created**: 2026-06-05

**Status**: Draft

**Input**: User description: "Evals And Runtime Observability — trace compaction policy, structured event vocabulary, JSONL export, runtime metrics, eval fixtures, and dashboards integration."

## Clarifications

### Session 2026-06-05

- Q: How should the system handle event schema evolution (JSONL export contract)? → A: Per-event-type `schema_version`. Additive fields allowed within the same major version. Meaning changes, removed fields, or incompatible shape changes require a new major version for that event type. Optional export-level metadata may exist but must not replace per-event versioning.
- Q: What is the expected upper bound for a single trace before compaction becomes necessary? → A: 10k–50k trace items is the designed range for a single bounded local classification pass. Traces exceeding 50k items or the configured byte-size limit must not silently continue — they require explicit operator confirmation, chunked processing, or fail with an actionable message explaining the bound.
- Q: What is the tiebreaking rule when a trace item's classification is ambiguous between two retention classes? → A: The stricter (higher-preservation) class wins: ambiguous between structured and summary → keep structured; ambiguous between summary and index-only → keep summary; ambiguous between lossless and anything else → keep lossless. The compaction event must record that the item was classified by conservative tiebreaking so later inspection can explain why storage savings were not maximized.
- Q: What is the scope of compaction triggering — operator command, retention policy config, or both? → A: Operator command only for this slice (`boundline trace compact`). No retention policy configuration file is introduced. Policy-driven compaction is a follow-on slice after the explicit command path is proven safe.
- Q: What granularity should the eval pass/fail summary have for CI integration? → A: Per-eval pass/fail status with suite-level AND aggregate. The summary must include eval id, name, status, failure reason (when failed), source/fixture references, expected and actual outcome, duration, and suite aggregate status. CI exit code fails when any required eval fails.

### User Story 1 - Run Quality Evals To Validate Behavior Changes (Priority: P1)

An operator or CI pipeline runs an eval suite against the runtime to confirm that a behavior change, provider integration, or planning gate still meets the defined quality criteria. The eval suite covers planning quality, context selection quality, guardian finding quality, council rejection behavior, provider call failure handling, trace compaction survival of accepted decisions, and trace compaction survival of rejection reasons.

**Why this priority**: Without evals, every behavior change is a blind regression risk. Evals are the foundation that makes the rest of observability meaningful — metrics and dashboards are decorations without a validated quality baseline.

**Independent Test**: Can be fully tested by running a single eval fixture (e.g., planning-quality eval) against a known-good session and a known-broken session and confirming that only the broken session fails.

**Acceptance Scenarios**:

1. **Given** a session where planning analysis correctly blocked execution, **When** the planning-quality eval runs, **Then** the eval reports the blocked finding as expected and passes.
2. **Given** a session where a critical context item was omitted from the assembly, **When** the critical-context-omission eval runs, **Then** the eval detects the omission and fails with a source-attributed explanation.
3. **Given** a session with no defects, **When** the full eval suite runs, **Then** all evals pass and produce a machine-readable summary suitable for CI integration.
4. **Given** a provider call that returned a recoverable failure, **When** the provider-call-failure eval runs, **Then** the eval verifies that the failure was recorded as a structured event with correct attribution.

---

### User Story 2 - Protect Critical Evidence With Trace Compaction (Priority: P2)

An operator or retention policy triggers trace compaction to reduce trace storage. The compaction policy classifies every trace item into one of five retention classes: lossless, structured, summary, index-only, or discardable. Accepted decisions, rejection reasons, and active stage evidence must never be destructively compacted.

**Why this priority**: Trace growth is unbounded without compaction, but naive compaction destroys forensic evidence. A defined policy with hard survival rules protects auditability while managing storage.

**Independent Test**: Can be fully tested by running compaction against a trace that contains accepted decisions, rejection reasons, and assistant transcripts, then confirming that decisions and rejections survived exactly while transcripts were summarized with lossy markers.

**Acceptance Scenarios**:

1. **Given** a trace containing accepted decisions, rejection reasons, and long assistant transcripts, **When** trace compaction runs, **Then** accepted decisions and rejection reasons remain in their exact original form and long transcripts are replaced with lossy-marked summaries that preserve source references.
2. **Given** a trace where every item is classified as lossless, **When** compaction runs, **Then** no item is destructively compacted and the compaction event records zero lossy actions.
3. **Given** active stage evidence exists in a trace, **When** compaction is requested, **Then** active stage evidence is never compacted, regardless of its classification class.
4. **Given** duplicate generated output and temporary debug dumps exist, **When** compaction runs under a configured retention policy, **Then** discardable items are removed and the compaction action is recorded as a structured event.

---

### User Story 3 - Export And Visualize Runtime Observability (Priority: P3)

An operator or dashboard consumer exports structured runtime events — including metrics, finding summaries, and compaction records — to feed external dashboards, analysis tools, or long-term observability pipelines.

**Why this priority**: Structured export and dashboards are the consumer-facing surface of observability. They are valuable only after evals and compaction produce trustworthy data.

**Independent Test**: Can be fully tested by exporting a session's structured events as JSONL, confirming that each event has a recognized event type, and verifying that metrics fields (context size, provider latency, finding count) are present and accurate.

**Acceptance Scenarios**:

1. **Given** a session where planning analysis, guardian checks, and provider calls ran, **When** the runtime exports structured events, **Then** the export includes events for each phase with reproducible identifiers and source references.
2. **Given** a dashboard consumer reads the JSONL export, **When** it queries for compaction metrics, **Then** compaction count, class distribution, trace size before and after, and lossy compaction count are all present.
3. **Given** a CI pipeline consumes the exported events, **When** it filters for stop-reason and finding-count metrics, **Then** those fields are available and match the runtime state.

---

### Edge Cases

- What happens when a trace contains items whose classification is ambiguous between structured and summary? → Resolved: The stricter (higher-preservation) class wins — e.g., structured over summary, lossless over anything else. The compaction event records the tiebreaking.
- How does the system handle a compaction run where the retention policy conflicts with the hard rule that active stage evidence must never be compacted? → Resolved: Hard survival rules always win. Active stage evidence is classified as lossless regardless of what the retention policy or classification table would otherwise assign. The compaction event records a policy-override action for each overridden item.
- What happens when an eval fixture references a session that no longer has complete trace data due to a prior compaction? → Resolved: The eval runner detects missing or incomplete fixture data before evaluation begins and fails with an actionable message identifying which fixture data is unavailable and why (e.g., "fixture references compacted trace item decision-12 — re-run session or restore from backup").
- How does the system handle duplicate events in the JSONL export (e.g., two compaction events for the same trace)? → Resolved: Events are deduplicated by stable `event_id`. If two events share the same `event_id`, only the first occurrence is exported. The export summary records the deduplication count.
- What happens when observability export is requested but no structured events have been emitted yet? → Resolved: The system produces a valid empty JSONL stream (zero lines) and emits a status-level message indicating that no structured events were present for the session. The export command exits with code 0.
- How does the system prevent sensitive data (secrets, tokens, PII) from leaking into exported events or metrics?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a locally-runnable eval suite that covers at minimum planning-quality, context-selection quality, critical-context omission, guardian finding quality, council rejection behavior, provider call failure handling, trace compaction survival of accepted decisions, and trace compaction survival of rejection reasons.
- **FR-002**: The system MUST allow evals to run in CI and produce a machine-readable summary that includes, per eval: eval id, eval name, pass/fail status, failure reason (when failed), source or fixture references, expected outcome, actual outcome, and duration. The suite aggregate status MUST be the AND of all required eval results, and the CI exit code MUST fail when any required eval fails.
- **FR-003**: The system MUST emit structured runtime events for major state transitions including planning analysis outcomes, guardian findings, provider calls, phase requests, route decisions, and context selection records.
- **FR-004**: The system MUST implement a trace compaction policy that classifies every trace item into one of five retention classes: lossless, structured, summary, index-only, or discardable.
- **FR-005**: The system MUST preserve accepted decisions, approvals, final stage outputs, rejection reasons, operator answers, contract validation results, evidence references, and release validation results as lossless — never destructively compacted and never replaced with summary only.
- **FR-006**: The system MUST normalize guardian findings, provider findings, test summaries, lint summaries, phase requests, route decisions, and context selection records into structured event records that preserve source references.
- **FR-007**: The system MUST mark lossy summaries (including summarized assistant transcripts and compacted command logs) as lossy and must retain source references so the summary cannot become the sole authority for completion or approval decisions.
- **FR-008**: The system MUST never compact active stage evidence, regardless of classification.
- **FR-009**: The system MUST never discard rejection reasons, even when they would otherwise fall into a non-lossless class.
- **FR-010**: The system MUST emit a trace-visible compaction event for every compaction run, including source trace identifier, compaction actions taken, per-item from/to classification, lossy flags, and preserved references.
- **FR-011**: The system MUST record runtime metrics including compaction count, compaction class distribution, trace size before and after compaction, lossy compaction count, preserved decision count, preserved rejection count, context size, context item count, provider latency, stop reason, and finding count.
- **FR-012**: The system MUST support JSONL export that can feed external dashboards and analysis tools, with every exported event carrying a per-event-type `schema_version` so consumers can handle compatibility per event type.
- **FR-013**: The system MUST ensure that every behavior change affecting AI decision-making has an associated eval path before the change is accepted. Enforcement in this slice is through review process and CI checklist gates, not through automated source-code analysis.
- **FR-014**: The system MUST prevent sensitive data (secrets, tokens, PII) from appearing in structured events, metrics exports, or JSONL exports.
- **FR-015**: Every structured runtime event in the JSONL export MUST carry a per-event-type `schema_version` field. Additive field additions are permitted within the same major version. Field removals, meaning changes, or incompatible shape changes require a new major version for that event type. Optional export-level metadata may supplement per-event versioning but MUST NOT replace it.
- **FR-016**: The system MUST handle traces exceeding the designed compaction bound (50k items or the configured byte-size limit) by requiring explicit operator confirmation, using chunked processing, or failing with an actionable message that explains the bound — it MUST NOT silently continue as if the trace were within the normal range.
- **FR-017**: When a trace item's classification is ambiguous between two retention classes, the system MUST apply conservative tiebreaking: the stricter (higher-preservation) class wins. The compaction event MUST record that the item was classified by tiebreaking so later inspection can explain why storage savings were not maximized.
- **FR-018**: The system MUST provide an explicit operator command to trigger trace compaction. No retention policy configuration file, threshold-based automatic trigger, or background compaction is introduced in this slice.
- **FR-019**: When a retention policy or classification table would assign a non-lossless class to active stage evidence, the hard survival rule MUST override the policy: active stage evidence is classified as lossless regardless of what the policy would otherwise assign. The compaction event MUST record each policy-override action with the original classification, the overridden classification, and the reason.
- **FR-020**: The eval runner MUST detect when a fixture references trace data that is no longer available due to a prior compaction and MUST fail with an actionable message identifying which fixture data is missing and why, before any eval assertions are evaluated.
- **FR-021**: The JSONL export MUST deduplicate events by stable `event_id`: when two or more events share the same `event_id`, only the first occurrence is included in the export. The export summary MUST record the deduplication count.
- **FR-022**: When a session has emitted no structured events, the JSONL export MUST produce a valid empty stream (zero lines) and the export command MUST exit with code 0 while emitting a status-level message indicating that no structured events were present.

### Key Entities

- **Structured Runtime Event**: A typed, timestamped record of a runtime state transition (planning analysis, guardian finding, provider call, phase request, route decision, context selection, compaction action) with reproducible identifiers, source references, a per-event-type `schema_version` that governs additive and breaking change compatibility, and relevant metrics.
- **Trace Compaction Policy**: The rule set that maps every trace item to one of five retention classes (lossless, structured, summary, index-only, discardable) and governs which items survive compaction in what form.
- **Compaction Action**: A record of a single trace item transformation during compaction, including the item reference, the original classification, the target classification, whether the transformation was lossy, and whether the classification was resolved by conservative tiebreaking.
- **Compaction Event**: A structured event emitted after every compaction run, documenting the source trace, the policy version, all compaction actions taken, and the list of preserved references.
- **Eval Fixture**: A test case that validates a specific quality dimension (planning, context, guardian, council, provider) against a known session state and produces a pass/fail result with source-attributed explanation.
- **Metrics Snapshot**: A collection of counters and measurements captured during a runtime session (compaction statistics, context dimensions, provider latency, stop reasons, finding counts) suitable for export and dashboard consumption.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The eval suite can be executed locally and in CI, producing a machine-readable pass/fail summary within 5 minutes for a session with a trace of up to 50k items.
- **SC-002**: After trace compaction, 100% of accepted decisions and rejection reasons survive in their exact original form across validated regression cases.
- **SC-003**: Every lossy compaction action produces a trace-visible event that records the source item, target class, and lossy flag within 1 second of compaction completion.
- **SC-004**: Structured events for planning analysis, guardian findings, and provider calls are emitted before the next runtime phase begins, with no event dropped or duplicated in validated regression cases.
- **SC-005**: JSONL export contains all structured events from a session with correct event-type labels, reproducible identifiers, and metrics fields that match the runtime state exactly.
- **SC-006**: Sensitive data (secrets, tokens, PII) is absent from 100% of exported events and metrics in validated regression fixtures that include simulated sensitive inputs.
- **SC-007**: An operator can determine from a compaction event which items were compacted, which class each item moved to, whether the transformation was lossy, and which references were preserved — without opening the raw trace.

## Assumptions

- The initial eval corpus will be seeded from curated regression fixtures already present in the repository; a separate eval-corpus expansion is out of scope for this slice.
- Trace compaction is triggered explicitly by an operator command (`boundline trace compact`). No retention policy configuration file or automatic trigger is introduced in this slice; policy-driven compaction is deferred to a follow-on slice.
- Dashboard and visualization tooling consumes the JSONL export but is built and maintained externally; this feature provides only the export surface.
- The existing trace storage format under `.boundline/traces/` is the input for compaction; a new storage backend is not introduced.
- The designed compaction bound is 10k–50k trace items in a single local classification pass. Traces exceeding this bound or a configured byte-size limit require explicit handling (confirmation, chunking, or actionable failure).
- Sensitive-data filtering in exports relies on field-level allowlists defined per event type rather than a general-purpose scanner.
- The hard rule "every AI behavior change must have an eval path" is enforced through review process and CI gates, not through automated code analysis in this slice.

