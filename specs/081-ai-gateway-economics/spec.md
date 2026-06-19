# Feature Specification: AI Gateway And Inference Economics

**Feature Branch**: `081-ai-gateway-economics`

**Created**: 2026-06-17

**Status**: Draft

**Input**: User description: "AI Gateway And Inference Economics". Roadmap seed preserved as `feat-ai-gateway-and-inference-economics.md`.

## Clarifications

### Session 2026-06-17

- Q: How should session budgets be measured and enforced when exact provider cost is missing? → A: Use one provider-agnostic session budget in a configured currency with USD as the default reference currency; reserve a conservative estimated amount before each admitted call, replace it with the exact or best available actual cost after completion, never default missing cost to zero, and apply an explicit unknown-cost policy when exact and estimated costs are unavailable.
- Q: What should happen when a provider-backed call has unknown cost? → A: Set `cost_quality` to `unknown`, pause before execution, project `budget_state` as `approval_required`, require explicit operator approval scoped to the specific call or an explicitly bounded equivalent group, keep unknown cost visible after execution unless exact cost arrives later, never reconcile unknown cost to zero, and block non-interactive execution unless an explicit pre-authorized unknown-cost policy exists.
- Q: Who may approve spend exceptions (unknown-cost and over-budget) for inference calls? → A: Use authority-zone-based approval: the active session owner may approve a bounded spend exception for low-risk, non-egress calls; red-zone calls and calls that transmit repository content outside the approved execution boundary require a governance approver; repository-egress calls also require the actor authorized by the existing provider permission and data-transmission policy; `unknown_cost_approval` and `budget_override` are distinct decision types that share the same resolution mechanism; V1 defaults approval scope to `single_call` and derives approval authority from existing session, workspace, and governance roles without introducing a separate identity or RBAC system.
- Q: How should pricing snapshots be maintained so budget reservations stay trustworthy over time? → A: Pricing snapshots are explicit, versioned, operator-owned configuration artifacts; Boundline records the snapshot used for every pre-call reservation and reports staleness when its age exceeds a configurable threshold; staleness affects reservation confidence but must never downgrade an exact provider-reported post-call cost to `estimated`; operators own creation, review, and activation; Boundline does not silently fetch or activate new prices in V1; activating a new snapshot affects future reservations only and historical call records remain bound to the snapshot used at admission time.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Enforce Session Budget (Priority: P1)

As a delivery operator, I want each session to respect a configurable monetary AI spend budget so that governed work cannot overrun cost limits without an explicit stop, escalation, or override signal.

**Why this priority**: Budget enforcement is the minimum safe slice because it gives immediate economic control without requiring a new routing strategy to replace current defaults.

**Independent Test**: Configure a session budget currency and limit, run a mix of low-cost and high-cost AI-backed tasks, including calls with unknown cost, and verify that conservative reservations, operator-approval pauses, actual cost replacement, and blocked outcomes keep spent plus reserved cost inside policy.

**Acceptance Scenarios**:

1. **Given** a session budget in a configured currency and a route with an applicable pricing snapshot, **When** Boundline admits a provider-backed inference call, **Then** it reserves a conservative estimated amount using estimated input tokens, configured maximum output tokens, and the active pricing snapshot before execution starts.
2. **Given** a pending provider-backed call whose conservative reservation exceeds the remaining session budget, **When** Boundline evaluates route admission, **Then** the route does not start unless an explicit spend exception approval is granted by the required approver for the call's authority zone, and the session surfaces the budget reason clearly.
3. **Given** a low-risk, non-egress inference call whose conservative reservation exceeds the remaining budget or whose cost is unknown, **When** the active session owner approves a bounded spend exception, **Then** Boundline admits the call, records the approval with the session owner's identity and the declared scope, and does not create a permanent route exemption.
4. **Given** a red-zone or repository-egress inference call that requires a spend exception, **When** the session owner attempts to self-approve, **Then** Boundline rejects the approval unless the existing governance policy explicitly assigns both roles to the same actor, and the call remains blocked until a governance approver grants the exception.
5. **Given** a provider-backed call whose cost is neither exactly reported nor estimable from configured pricing, **When** Boundline evaluates admission, **Then** it pauses before execution, projects `budget_state = approval_required`, and requests explicit operator approval with provider, model, route, task or lifecycle context, current spent and reserved amounts, remaining known budget, the reason cost is unknown, and whether repository content will leave the local environment.
6. **Given** an approved unknown-cost call, **When** execution completes without exact cost data, **Then** Boundline keeps `cost_quality = unknown`, never reconciles the call to zero, increments the unknown-cost call count in session projections, and labels remaining budget as based on known spend only.
7. **Given** an unknown-cost call whose exact provider cost arrives after execution, **When** Boundline reconciles the call, **Then** it updates session totals while preserving the original approval record and prior unknown-cost audit trail.
8. **Given** a repository-egress call that requires both a spend exception and a data-transmission authorization, **When** Boundline evaluates the required approvals, **Then** it records both decisions separately and does not allow spend approval alone to authorize repository-content transmission.
9. **Given** a pricing snapshot whose age exceeds the configured staleness threshold, **When** Boundline estimates a reservation, **Then** it records the estimate as `stale_estimate` with the snapshot identifier and age but does not automatically block the call; the call proceeds or pauses according to the configured staleness policy for its authority zone.

---

### User Story 2 - Route By Risk And Health (Priority: P2)

As a governance lead, I want low-risk work to use cheaper eligible routes and high-risk work to use stronger eligible routes so that cost control never silently weakens delivery quality or authority controls.

**Why this priority**: Route selection is the core policy layer that turns telemetry and provider health into safe decisions. It is second because budget tracking can provide value before policy changes are activated.

**Independent Test**: Run tasks across different risk levels, authority zones, provider health conditions, remaining budget states, and unknown-cost outcomes, and verify that route choice, approval gates, fallback, and degraded outcomes match the configured policy.

**Acceptance Scenarios**:

1. **Given** a low-risk task with multiple healthy eligible routes, **When** the route is selected, **Then** the system chooses the lowest-cost route that satisfies the task requirements and budget policy.
2. **Given** a red-zone or governance-critical task, **When** the route is selected under budget pressure, **Then** the system keeps the required capability tier and does not silently downgrade to a weaker route.
3. **Given** a preferred route that is unhealthy or unavailable, **When** the task is evaluated, **Then** the system either selects an approved fallback route or enters a clearly explained degraded state.
4. **Given** a provider-backed route whose cost quality is unknown, **When** Boundline runs non-interactively without an explicit pre-authorized unknown-cost policy, **Then** the route is blocked before execution rather than admitted with a warning only.
5. **Given** a low-risk, non-egress inference call that requires a spend exception, **When** the session owner approves it, **Then** the approval is limited to one call or a bounded monetary amount and does not create a permanent route exemption.
6. **Given** a red-zone inference call that requires a spend exception, **When** the governance approver is unavailable in non-interactive execution, **Then** Boundline blocks the call unless a pre-authorized policy with an explicit scope and monetary ceiling exists.

---

### User Story 3 - Govern Route Changes And Private Routes (Priority: P3)

As a platform owner, I want route changes to require evaluation approval and I want private or local routes to be available for eligible work so that routing policy can evolve without uncontrolled quality regressions or privacy leaks.

**Why this priority**: This expands the system from operational control to safe policy evolution. It depends on telemetry and route decisions already being visible and enforceable.

**Independent Test**: Propose a route-policy change, attempt to activate it without evaluation approval, then activate it with approval and verify that private-route eligibility, zero-marginal-cost reporting, route visibility, and approval-scope governance are enforced.

**Acceptance Scenarios**:

1. **Given** a proposed route-policy change without the required evaluation approval, **When** activation is attempted, **Then** the change remains inactive and the reason is visible.
2. **Given** a privacy-sensitive task and an eligible private route, **When** the route is selected, **Then** the system can use the private route instead of a public route.
3. **Given** a local route with zero marginal monetary cost, **When** Boundline records the call outcome, **Then** it records the cost source explicitly as `local_zero_marginal_cost` rather than inferring zero from missing telemetry.
4. **Given** an operator approval for an unknown-cost call, **When** Boundline stores the approval, **Then** the record includes operator identity, timestamp, scope, provider, model, route, and reason, and does not silently create a permanent provider-wide exemption.
5. **Given** a consumed spend exception approval recorded with scope `single_call`, **When** a subsequent call attempts to reuse that approval, **Then** Boundline rejects the reuse and requires a new approval.
6. **Given** an unused spend exception approval at session end, **When** Boundline closes the session, **Then** the unused approval capacity does not carry into another session or route.

### Edge Cases

- What happens when a session budget is nearly exhausted but the next required task is red-zone and no lower-cost equivalent route exists?
- How does the system behave when all eligible routes for a task are unhealthy, blocked, or missing required capability metadata?
- How does the system report costs when a provider returns latency but no token counts, or token counts but no direct price information?
- What happens when the provider returns a native currency that differs from the session currency and no approved conversion source is available at decision time?
- What happens when a private or local route is configured for a task class but is temporarily unavailable at execution time?
- How does the system behave when a route-policy change improves cost but fails evaluation thresholds for quality or governance posture?
- What happens when repeated equivalent unknown-cost calls are requested after a `single_call` approval has already been consumed?
- What happens when the required approver for a red-zone or egress spend exception is unavailable during an active run?
- How does the system behave when a session owner attempts to approve a red-zone spend exception without an explicit governance policy assigning both roles?
- What happens when a repository-egress call has spend approval but the data-transmission authorization has not yet been granted?
- What happens when the configured pricing snapshot for a provider or model is `missing` or `invalid` at reservation time?
- How does the system behave when a newer pricing snapshot is activated mid-session and in-flight reservations used the prior snapshot?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow operators to configure one provider-agnostic session budget in a configured currency, with USD as the default reference currency.
- **FR-002**: The system MUST track budget limit, known spent amount, reserved amount, remaining known budget, unknown-cost call count, cost basis, pricing snapshot identifier, and budget state for each active session.
- **FR-003**: The system MUST classify each AI-backed task against a routing policy that considers task class, lifecycle phase, authority zone, risk, context size, reasoning depth, privacy requirement, latency budget, cost budget, provider health, and evaluation posture.
- **FR-004**: The system MUST reserve a conservative estimated monetary amount before every admitted provider-backed inference call using estimated input tokens, configured maximum output tokens, and the applicable pricing snapshot.
- **FR-005**: The system MUST prevent a route from starting when its conservative reservation exceeds the remaining session budget unless a spend exception approval is granted by the required approver for the call's authority zone.
- **FR-006**: After a provider-backed call completes, the system MUST replace the reservation with exact provider-reported cost when available, otherwise with the best available versioned configured pricing estimate, otherwise with an explicit unknown-cost state.
- **FR-007**: The system MUST NOT default cost to zero when usage, pricing, or conversion data is unavailable.
- **FR-008**: The system MUST enforce an explicit unknown-cost policy for calls whose exact and estimated costs are unavailable.
- **FR-009**: Operators MUST own pricing snapshot creation, review, and activation; Boundline MUST NOT silently fetch or activate new prices in V1.
- **FR-010**: Each pricing snapshot MUST include: snapshot identifier, schema version, effective timestamp, source or provenance, provider and model identifiers, input pricing, output pricing, cached-input pricing, other applicable pricing dimensions, and native currency.
- **FR-011**: Every estimated pre-call reservation MUST record: pricing snapshot identifier, snapshot age at reservation time, pricing entries used, estimated input and maximum output usage, normalized session currency, and reservation confidence as `current_estimate`, `stale_estimate`, or `unknown`.
- **FR-012**: A configurable staleness threshold MUST determine whether a reservation estimate is `current_estimate`, `stale_estimate`, or `unavailable`; a stale snapshot MUST NOT automatically block a call, and policy MAY allow the call with a stale-pricing warning, require approval, or block when the route or authority zone requires stronger cost certainty.
- **FR-013**: When a model has no applicable pricing entry in any active snapshot, the system MUST set `cost_quality` to `unknown` and follow the unknown-cost approval policy.
- **FR-014**: Snapshot staleness MUST affect the confidence of estimated reservations but MUST NOT downgrade an exact provider-reported post-call cost from `exact` to `estimated` or to any lower-confidence state.
- **FR-015**: After call completion, the system MUST reconcile cost using exact provider cost when available, otherwise the best available estimate with its snapshot provenance; reconciliation MUST preserve both the original reservation snapshot identifier and the final accounting basis.
- **FR-016**: Activating a new pricing snapshot MUST affect future reservations only; historical call records MUST remain bound to the snapshot used at admission time, and the system MUST NOT rewrite historical costs because a newer snapshot becomes available.
- **FR-017**: Snapshot refresh MUST be an explicit operator action in V1; automated retrieval MAY be added later as a source adapter, but activation MUST remain explicit and auditable.
- **FR-018**: The system MUST support snapshot state values `current`, `stale`, `missing`, and `invalid`.
- **FR-019**: The system MUST support reservation cost quality values `current_estimate`, `stale_estimate`, and `unknown`.
- **FR-020**: The system MUST support reconciled cost quality values `exact`, `estimated`, `unknown`, and `local_zero_marginal_cost`.
- **FR-021**: When neither provider-reported cost nor a reliable configured estimate is available, the system MUST set `cost_quality` to `unknown`, pause before execution, and project `budget_state` as `approval_required`.
- **FR-022**: The approval request for an unknown-cost or over-budget call MUST show provider, model, route, task or lifecycle context, authority zone, repository-egress status, current known spent and reserved amounts, remaining known budget, why the exception is needed, and the required approver role.
- **FR-023**: The system MUST distinguish two spend exception decision types: `unknown_cost_approval` and `budget_override`, which share the same authority-zone-based approval-resolution mechanism.
- **FR-024**: For low-risk, non-egress inference calls, the active session owner MAY approve a bounded spend exception limited to one call or a bounded monetary amount; the approval MUST NOT create a permanent route exemption.
- **FR-025**: For red-zone calls and calls that transmit repository content outside the approved execution boundary, approval MUST come from a governance approver; the session owner MUST NOT self-approve unless the existing governance policy explicitly assigns both roles to the same actor.
- **FR-026**: For repository-egress calls, spend exception approval MUST be recorded separately from the data-transmission authorization required by the existing provider permission and data-transmission policy; spend approval alone MUST NOT authorize repository-content transmission.
- **FR-027**: Non-interactive execution MUST block spend exceptions when the required approver is unavailable, unless a pre-authorized policy with an explicit scope and monetary ceiling exists.
- **FR-028**: Approval authority MUST be derived from existing session ownership, workspace roles, authority-zone policy, and governance roles; V1 MUST NOT introduce a separate identity or RBAC system solely for inference economics.
- **FR-029**: Every spend exception approval record MUST include: approval type (`unknown_cost_approval` or `budget_override`), approver identity and role, session and execution-run references, provider, model, route, authority zone, repository-egress status, approved amount or bounded scope, reason, and timestamp with expiry or consumption state.
- **FR-030**: The system MUST support approval scope values `single_call`, `bounded_task`, and `bounded_session`, and MUST default to `single_call` in V1.
- **FR-031**: One approval MUST be consumable only within its declared scope; unused approval capacity MUST NOT silently carry into another session or route.
- **FR-032**: A spend exception approval MUST NOT authorize a weaker model for red-zone governance; model capability and governance requirements MUST remain unchanged by any approval.
- **FR-033**: A call approved with unknown cost MUST remain visible as unknown after execution unless exact provider cost becomes available later.
- **FR-034**: If exact provider cost arrives after execution for a previously unknown-cost call, the system MUST reconcile the call and update session totals while preserving the original approval record.
- **FR-035**: If cost remains unknown after execution, session projections MUST report known spend separately from unknown-cost call count, and remaining budget MUST be labeled as based on known spend only.
- **FR-036**: Non-interactive execution MUST block unknown-cost calls unless an explicit pre-authorized unknown-cost policy exists.
- **FR-037**: Red-zone governance MUST NOT be downgraded to avoid the unknown-cost or over-budget approval requirement or to reduce cost, and unknown cost MUST be treated as economic uncertainty rather than a change in task risk classification.
- **FR-038**: The system MUST record per-call telemetry that includes provider identity, model identity, selected route, latency, fallback route when used, failure reason when present, pricing snapshot identifier, snapshot age at reservation time, and token or cache details when available.
- **FR-039**: The system MUST record native provider currency when supplied, normalized session currency, conversion source and timestamp when conversion is required, and whether each monetary amount is exact, estimated, unknown, or `local_zero_marginal_cost`.
- **FR-040**: The system MUST preserve exact monetary precision for reservation, spend, conversion, and projection calculations, and MUST NOT rely on floating-point arithmetic that can silently change budget outcomes.
- **FR-041**: The system MUST support a deterministic non-LLM route for work that does not require model inference.
- **FR-042**: The system MUST support policy-managed hosted routes, gateway-mediated routes, and local or self-hosted routes within the same route-selection framework.
- **FR-043**: The system MUST maintain route readiness and health state for each configured route and MUST exclude unavailable routes from selection for eligible requests.
- **FR-044**: When a preferred route is unavailable, the system MUST either select an approved fallback route or move the session into a clearly explained degraded or blocked state.
- **FR-045**: The system MUST prefer the lowest-cost eligible route for low-risk work when multiple healthy routes satisfy the task requirements.
- **FR-046**: The system MUST NOT allow cost optimization, reservation pressure, fallback policy, snapshot staleness, unknown-cost approval handling, or spend exception approvals to silently reduce the required capability level for red-zone, governance-critical, or architecture-critical work.
- **FR-047**: The system MUST allow at least one private or local route to be configured for eligible work classes without requiring all sessions to use a gateway.
- **FR-048**: The system MUST expose route choice, fallback behavior, budget consumption, reservation state, snapshot staleness, approval-required state, required approver role, unknown-cost call count, known spend, remaining known budget, and degraded-state reasons in trace and session-facing status or inspection outputs.
- **FR-049**: The system MUST require route-policy changes to satisfy evaluation approval criteria before those changes can become active for governed work.
- **FR-050**: The system MUST preserve existing default model-selection behavior for work outside the explicitly configured routing and budget policies in the first delivery slice.

### Key Entities *(include if feature involves data)*

- **Pricing Snapshot**: A versioned, operator-owned configuration artifact containing snapshot identifier, schema version, effective timestamp, source or provenance, provider and model identifiers, input/output/cached-input pricing dimensions, native currency, and snapshot state (`current`, `stale`, `missing`, `invalid`).
- **Route Policy**: A governed rule set that maps task characteristics and operational constraints to an eligible routing tier, fallback behavior, and override posture.
- **Session Budget Projection**: A per-session monetary budget view containing currency, budget limit, known spent amount, reserved amount, remaining known budget, unknown-cost call count, pricing snapshot identifier, cost basis, budget state, and required action when approval is needed.
- **Spend Exception Approval Record**: The audit record for an approved spend exception, including approval type (`unknown_cost_approval` or `budget_override`), approver identity and role, session and execution-run references, provider, model, route, authority zone, repository-egress status, approved amount or bounded scope, reason, and timestamp with expiry or consumption state.
- **Spend Exception Decision Projection**: The runtime projection of a pending spend exception decision, including approval type, approval state, required approver role, authority zone, repository-egress status, requested amount, currency, and list of required actions.
- **Route Health Snapshot**: A current view of route readiness, availability, and capability eligibility used during route selection.
- **Invocation Cost Record**: A per-call record containing reservation amount, final amount, native provider currency, normalized session currency, conversion provenance, pricing snapshot identifier, and cost quality classification.
- **Route Telemetry Record**: The traceable record of a single provider-backed inference decision and outcome, including route identity, provider identity, model identity, latency, cache state, fallback behavior, and failure information.
- **Route Change Proposal**: A governed change record that describes a routing-policy update, its evaluation status, and whether it is eligible for activation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of governed validation scenarios with a configured session budget, the system maintains visible known spent, reserved, and remaining known budget amounts and prevents admitted work from exceeding the budget by more than one already-authorized in-flight request.
- **SC-002**: In 100% of unknown-cost or over-budget admission scenarios without a pre-authorized policy, the system pauses before execution, projects the required approver role based on authority zone, and blocks non-interactive execution until an allowed approval from the correct role is present.
- **SC-003**: In 100% of red-zone or repository-egress spend exception scenarios, the system rejects a session-owner self-approval unless governance policy explicitly assigns both roles, and the call remains blocked until a governance approver grants the exception.
- **SC-004**: In 100% of approved spend exception scenarios, the system preserves the approval record with all required fields, keeps cost quality and approval type visible after execution, and never reconciles unresolved unknown cost to zero.
- **SC-005**: In 100% of provider-backed calls, the system records the pricing snapshot identifier and snapshot age used for the pre-call reservation, and exact provider-reported post-call cost is never downgraded due to snapshot staleness.
- **SC-006**: In routing-policy validation scenarios covering low-risk, balanced, and red-zone work, 100% of red-zone tasks either use an approved high-governance route or halt with an explicit escalation outcome; none are silently downgraded, and no spend exception approval authorizes a weaker model for red-zone governance.
- **SC-007**: At least 95% of AI-backed calls surface provider, route, latency, normalized session-currency amount, pricing snapshot identifier, snapshot staleness, cost quality, approval-required state, and required approver role when applicable in trace or session outputs; unavailable fields are explicitly marked rather than silently omitted.
- **SC-008**: Governed route-policy changes and model upgrades remain inactive until the required evaluation gate passes in 100% of gated activation tests.

## Assumptions

- Existing provider integrations already expose enough capability and health metadata to determine route eligibility.
- The first delivery slice applies budgets at the active-session level, not at the organization, workspace, or user-account level.
- Versioned configured pricing snapshots are owned, reviewed, and activated by operators; Boundline does not silently fetch or activate new prices in V1, and snapshot refresh is an explicit operator action.
- A configurable staleness threshold determines reservation estimate confidence; stale snapshots do not automatically block calls, and exact provider-reported cost is never downgraded due to snapshot age.
- A supported conversion source exists for providers whose native billing currency differs from the session currency; when conversion data is unavailable, the system treats cost as non-zero and non-assumed.
- V1 defaults unknown-cost and spend exception approval scope to `single_call`; broader scopes (`bounded_task`, `bounded_session`) require explicit configured policy and a monetary ceiling.
- Existing trace, session status, and inspection surfaces can be extended to present route, budget, approval-required, and required-approver-role decisions without replacing their current responsibilities.
- Private or local routes are configured explicitly by operators and are only considered for task classes where their capability and privacy posture are acceptable.
- Approval authority for inference economics is derived from existing session ownership, workspace roles, authority-zone policy, and governance roles; V1 does not introduce a separate identity or RBAC system solely for this feature.
- Existing model-selection defaults stay in place unless an explicit budget, policy, health, or governance rule requires a different routing outcome.