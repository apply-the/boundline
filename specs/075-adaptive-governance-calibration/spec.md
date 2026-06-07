# Feature Specification: Adaptive Governance Calibration

**Feature Branch**: `075-adaptive-governance-calibration`

**Created**: 2026-06-06

**Status**: Draft

**Input**: User description: "Adaptive Governance Calibration — make one control-level decision fully inspectable including confidence input, override policy, degradation, and terminal outcome, built on existing adaptive governance, control graduation, degradation, escalation, and runtime confidence foundations."

See [source feature file](./feat-adaptive-governance-calibration.md).

## Clarifications

### Session 2026-06-06

- Q: Where is the calibration policy stored and how does it relate to guardian-rules.toml? → A: Separate `.boundline/calibration-policy.toml` that references guardian rules by `rule_id`. `guardian-rules.toml` decides which guardians activate; `calibration-policy.toml` decides how strictly their findings enforce. The calibration policy must be versioned, inspectable, and fail closed when invalid.
- Q: How does an operator provide an override to bypass a catch or rule block? → A: `boundline override` command writes an explicit override record (finding/control id, guardian id, requested level, reason, operator identity, timestamp, expiry/scope, policy satisfaction). `boundline run` and `boundline continue` consume the override record. `--override` CLI flag may be added later as a shortcut.
- Q: Where does the confidence score come from? → A: Hybrid model. Guardians provide an initial confidence score with each finding (0.0–1.0). Trust metrics (historical false positives, accepted overrides, eval pass rate, incident correlation) may adjust the effective confidence over time. Inspect must show both the raw guardian confidence and the effective calibrated confidence.
- Q: When does calibration evaluate trust metrics and promote/demote guardians? → A: Raw trust counters update after every council adjudication. Promotion/demotion evaluation occurs only after a configurable evidence window (default 5 adjudicated sessions). The system records adjudication outcomes continuously, accumulates trust metrics, evaluates calibration after the window, emits a trace-visible recommendation, and avoids promotion when evals are failing or evidence is insufficient.
- Q: What specific Canon policy data does calibration consume? → A: Authority zone and risk level only, when available in the active run context. If Canon authority zone or risk level is unavailable, Boundline uses its local runtime risk classification and records that Canon policy state was absent. Full Canon policy snapshot (mode, approvals, evidence refs) is out of scope for calibration v1.
- Q: What counts as a true positive vs false positive for guardian trust metrics? → A: A true positive is a guardian finding upheld by council adjudication as valid, actionable, and correctly classified. A false positive is a guardian finding rejected by council adjudication as invalid, not applicable, materially wrong, or incorrectly blocking. Deferred findings do not count until resolved. Partially valid findings count by final adjudication disposition: upheld = true positive, rejected = false positive, downgraded severity = true positive for existence with a separate severity calibration penalty. Trust metrics are computed only over adjudicated findings, not all emitted findings. No guardian may be promoted or demoted from an insufficient sample size; the calibration policy must define a minimum evidence threshold before true/false positive rates can affect control levels.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a Guardian's Control Level Decision (Priority: P1)

An operator runs `boundline inspect` or reviews a governance trace and needs to understand why a specific guardian rule is currently at advisory, catch, rule, or hook level. The operator sees the control level, the confidence score that drove it, the active override policy, any degradation state, and the terminal outcome (passed/blocked/deferred).

**Why this priority**: This is the first slice — making one control-level decision fully traceable and explainable. Without inspectability, adaptive governance cannot be trusted, tuned, or adopted by real teams. Every downstream feature (trust evolution, calibration table, escalation) depends on this visibility.

**Independent Test**: Can be fully tested by running `boundline inspect` on a workspace with guardian findings, verifying that the output includes the control level, confidence input, override policy state, degradation indicators, and terminal decision outcome for at least one guardian rule.

**Acceptance Scenarios**:

1. **Given** a workspace with an active guardian ruleset and at least one finding, **When** the operator runs `boundline inspect` with trace visibility, **Then** the output includes a control-level summary showing the current level (advisory/catch/rule/hook) for each activated guardian, with the reason for that level assignment.
2. **Given** a guardian finding that the council adjudicated as blocked, **When** the operator inspects the decision, **Then** the inspection output shows the confidence score that contributed to the blocking decision and whether an override policy was available but unsatisfied.
3. **Given** a provider or tool that became unavailable during execution, **When** the operator inspects the session trace, **Then** the inspection output shows which controls degraded (e.g., rule downgraded to advisory), why degradation occurred, and whether the degraded path was safe or required a human gate.
4. **Given** a guardian finding in the red zone with low confidence, **When** the council decision blocks execution, **Then** the inspection output explicitly shows that the red-zone blocker cannot silently downgrade and lists the escalation trigger that fired.

---

### User Story 2 - Council Adjudication Applies Graduated Control Levels (Priority: P2)

The council adjudicator reads the calibration policy for each activated guardian and applies the correct control level (advisory, catch, rule, or hook) based on the workspace's authority zone, risk level, lifecycle phase, guidance strength, authority source, and confidence inputs. Advisory findings are visible but do not block. Catch findings require attention but allow human bypass. Rule findings block unless an override policy is satisfied. Hook findings block unconditionally except for privileged processes.

**Why this priority**: This is the core runtime behavior that makes governance adoptable. Without graduated levels, every finding either blocks (too strict) or is advisory (too weak). Teams need the middle ground to adopt governance incrementally.

**Independent Test**: Can be tested by creating a workspace with a calibration policy that maps different authority zones and risk levels to different control levels, running `boundline run`, and asserting that the council decision correctly applies the level for each guardian.

**Acceptance Scenarios**:

1. **Given** a workspace in a low-risk zone with an advisory-level guardian rule, **When** the guardian produces a finding, **Then** the finding appears in the council decision as visible but does not block execution.
2. **Given** a workspace in a medium-risk zone with a catch-level guardian rule, **When** the guardian produces a finding, **Then** the council decision flags the finding for attention but allows the operator to bypass by writing an override record via `boundline override` and re-running.
3. **Given** a workspace in a high-risk zone with a rule-level guardian rule, **When** the guardian produces a finding, **Then** the council blocks execution unless the operator has written a satisfying override record via `boundline override` that meets the configured override policy.
4. **Given** a workspace with a hook-level security guardian, **When** the guardian produces a finding, **Then** the council blocks execution unconditionally and the only bypass path requires a privileged process (e.g., a Canon-approved exception).

---

### User Story 3 - Control Level Graduates Based on Trust and Evidence (Priority: P3)

Over time, as a guardian demonstrates reliability (high true positive rate, low false positives, strong eval performance), its control level can graduate from advisory to catch to rule. Conversely, a guardian with poor performance (many false positives, frequent overrides, incident correlations) can be demoted or kept at advisory.

**Why this priority**: Trust evolution is what makes adaptive governance "adaptive." It prevents governance from being either permanently too strict or permanently too loose. However, it depends on the visibility and graduated levels delivered by P1 and P2.

**Independent Test**: Can be tested by simulating a guardian's historical performance data (true positive rate, false positive count, override history, eval results), running the calibration logic, and asserting that the guardian's default level promotes or demotes correctly against defined thresholds.

**Acceptance Scenarios**:

1. **Given** a guardian with a high true positive rate and zero false positives over multiple sessions, **When** the calibration policy is evaluated, **Then** the guardian's default level promotes one step (e.g., advisory → catch, catch → rule).
2. **Given** a guardian with a high false positive rate or multiple accepted overrides, **When** the calibration policy is evaluated, **Then** the guardian's default level demotes or remains at advisory.
3. **Given** a guardian whose eval pass rate drops below the confidence threshold after the configured evidence window (default 5 sessions), **When** the calibration policy is evaluated, **Then** the guardian cannot be promoted to a stricter level until the eval pass rate recovers.
4. **Given** a guardian correlated with a past incident, **When** the calibration policy is evaluated, **Then** the guardian is locked at advisory or catch regardless of other metrics until the incident correlation is cleared.

---

### Edge Cases

- What happens when a calibration policy has contradictory level assignments for the same guardian under overlapping conditions? The system must fail closed and default to the stricter level (rule or hook).
- How does the system handle a guardian with no historical trust data (cold start)? The guardian defaults to its policy-defined default level, typically advisory or catch, until sufficient evidence accumulates.
- What happens when the Canon policy state is unavailable or stale? Degradation rules apply — controls may downgrade to advisory if safe, or block if mandatory evidence cannot be produced.
- How does the system handle simultaneous override and degradation? The stricter path wins: if a control is degraded but also has an active override, the override is evaluated first; if the override fails, degradation rules apply.
- What happens when a privileged process bypasses a hook-level block? The bypass must be trace-visible, include the authorizing identity and reason, and be surfaced in inspect output.
- What happens when Canon authority zone or risk level is unavailable? Boundline uses its local runtime risk classification and records in the trace that Canon policy state was absent for that calibration evaluation.
- How is trust computed when a guardian has too few adjudicated findings? The guardian's control level remains at its policy-defined default (advisory or catch). No promotion or demotion occurs until the adjudicated sample size meets the calibration policy's minimum evidence threshold.

## Requirements *(mandatory)*

### Hard Rules / Design Invariants

- **HR-001**: Adaptive governance must be more explainable than static governance, not less. Every decision, calibration, and degradation MUST be explicitly traceable and comprehensible to an operator.

### Functional Requirements

- **FR-001**: System MUST assign every activated guardian a control level (advisory, catch, rule, or hook) based on the active calibration policy. The control-level selection MUST participate with `guidance strength`, `authority source`, authority zone, risk level, lifecycle phase, confidence, override history, eval performance, and Canon policy state (when available) as first-class inputs.
- **FR-002**: System MUST expose the control level, guardian-provided confidence, effective calibrated confidence (after trust adjustment), override policy state, degradation indicators, and terminal outcome for each guardian finding through `boundline inspect` and trace output.
- **FR-003**: System MUST NOT allow red-zone blockers to silently downgrade; any degradation of a red-zone control must be explicit, trace-visible, and require a human gate or privileged process.
- **FR-004**: System MUST provide a `boundline override` command that writes an explicit override record containing: finding/control id, guardian id, requested control level, override reason, operator identity (when available), timestamp, expiry or scope, and whether the override satisfies the configured policy. `boundline run` and `boundline continue` MUST consume override records before adjudicating findings.
- **FR-005**: System MUST track guardian trust metrics (true positive rate, false positive count, accepted overrides, repeated violations, incident correlations, eval pass rate) across sessions within the workspace's trace store. Trust counters MUST update after every council adjudication. A true positive is a finding upheld by council adjudication as valid, actionable, and correctly classified. A false positive is a finding rejected as invalid, not applicable, materially wrong, or incorrectly blocking. Deferred findings MUST NOT count until resolved. Trust metrics MUST be computed only over adjudicated findings. Guardians MUST provide an initial confidence score (0.0–1.0) with each finding; trust metrics MAY adjust the effective calibrated confidence over time.
- **FR-006**: System MUST accumulate trust metrics continuously after every council adjudication, but evaluate promotion/demotion only after a configurable evidence window (default 5 adjudicated sessions). The evaluation MUST emit a trace-visible recommendation or level decision and MUST NOT promote a guardian when eval pass rate is below the confidence threshold or evidence is insufficient.
- **FR-007**: System MUST apply degradation rules when a provider, model, or tool is unavailable: downgrade to advisory if safe, require human gate if safety is uncertain, block if mandatory evidence cannot be produced.
- **FR-008**: System MUST escalate findings when: repeated unresolved findings exceed a threshold, the workspace is in a red zone, confidence is low but impact is high, mandatory evidence is missing, or a security/domain/contract boundary risk is detected.
- **FR-009**: System MUST persist the calibration policy in `.boundline/calibration-policy.toml`, a versioned TOML file separate from `.boundline/guardian-rules.toml`. The calibration policy MUST reference guardian rules by `rule_id`, be validated on load (fail closed on invalid or contradictory configuration), and be surfaced through `boundline inspect`.
- **FR-010**: System MUST emit structured runtime events (`control_level.assigned`, `control_level.graduated`, `control.degraded`, `control.escalated`) into the trace for observability.

### Key Entities

- **Calibration Policy**: A versioned TOML configuration stored in `.boundline/calibration-policy.toml`, separate from `.boundline/guardian-rules.toml`. The policy MUST formalize the calibration table shape to include at least: `rule_id`, `authority_source`, `default_level`, `green_level`, `yellow_level`, `red_level`, `confidence_threshold`, and `override_policy`. The `green_level`, `yellow_level`, and `red_level` fields represent the authority-zone or risk-zone bands used by Boundline, with Canon authority zone and risk level consumed as read-only inputs when available. Must be validated on load and fail closed when invalid or contradictory.
- **Control Level Assignment**: The current level (advisory, catch, rule, hook) of a specific guardian rule for a specific workspace context, including the reason for the assignment, the guardian-provided confidence score, the effective calibrated confidence after trust-metric adjustment, and the explicit inputs (`guidance strength`, `authority source`, authority/risk zones) that drove the assignment.
- **Guardian Trust Record**: Accumulated metrics for a guardian across adjudicated sessions only: true positive count (findings upheld as valid), false positive count (findings rejected as invalid or incorrectly blocking), deferred count (pending resolution), accepted override count, repeated violation count, incident correlation flags, eval pass rate. True positive rate = true positives / (true positives + false positives). No rate is computed when the adjudicated sample size is below the calibration policy's minimum evidence threshold.
- **Override Record**: A trace-visible record written by `boundline override` for a specific finding, containing: finding/control id, guardian id, requested control level, override reason, operator identity (when available), timestamp, expiry or scope, and whether the override satisfies the configured policy. Consumed by `boundline run` and `boundline continue` before adjudication.
- **Degradation Event**: A trace event recording that a control was downgraded (e.g., rule → advisory) due to provider/tool unavailability, including whether the degradation was safe or required a human gate.
- **Escalation Event**: A trace event recording that a finding was escalated due to repeated unresolved findings, red zone, low-confidence/high-impact, missing evidence, or boundary risk.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can determine why a specific guardian blocked or allowed execution within 30 seconds of running `boundline inspect`, without consulting external documentation.
- **SC-002**: A guardian with a true positive rate above 90% (computed only over adjudicated findings meeting the minimum evidence threshold) and zero false positives within the configured evidence window (default 5 adjudicated sessions) promotes at least one control level.
- **SC-003**: A guardian with a false positive rate above 20% (computed only over adjudicated findings meeting the minimum evidence threshold) within the configured evidence window (default 5 adjudicated sessions) does not promote and may demote to a less strict level.
- **SC-004**: 100% of red-zone control degradations are trace-visible and include the explicit reason, safety assessment, and required action (human gate or privileged bypass).
- **SC-005**: Every control level change (assignment, graduation, degradation, escalation) produces a structured trace event observable through `boundline inspect` and JSONL export.
- **SC-006**: A workspace with a freshly initialized calibration policy (no historical trust data) defaults all guardians to advisory or catch, and no guardian blocks execution on the first run.

## Assumptions

- The existing guardian activation router (`boundline council adjudicate`) and guardian ruleset (`.boundline/guardian-rules.toml`) provide the foundation for control level assignment; this feature extends, not replaces, that foundation. Boundline strictly owns runtime calibration, control-level selection, degradation, escalation, override handling, trace, inspect, and terminal runtime outcome. This feature must not create a second policy engine or second trust model.
- Trust metrics are accumulated within the workspace's `.boundline/traces/` store; cross-workspace or global trust aggregation is out of scope for this feature.
- The calibration policy file format is TOML, consistent with existing `.boundline/` configuration files.
- Canon policy state consumed by calibration is limited to authority zone and risk level from the active run context. Full Canon policy snapshot (mode, approvals, evidence refs, packet state) is consumed by council adjudication, not by calibration v1.
- If Canon authority zone or risk level is unavailable, Boundline uses its local runtime risk classification and records the absence in the trace.
- The first slice (P1) targets a single guardian rule end-to-end; multi-guardian calibration and cross-guardian interaction are deferred to future slices.
- Override policies are per-guardian-rule, not per-finding; per-finding override granularity is out of scope for v1.
- Degradation rules apply only to provider/model/tool unavailability, not to partial output quality degradation.
