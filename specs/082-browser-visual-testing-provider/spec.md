# Feature Specification: Browser And Visual Testing Provider

**Feature Branch**: `082-browser-visual-testing-provider`

**Created**: 2026-06-19

**Status**: Draft

**Input**: User description: "Browser And Visual Testing Provider". Roadmap seed preserved as `roadmap/features/21-browser-and-visual-testing-provider.md`.

## Clarifications

### Session 2026-06-19

- Q: FR-021 states the provider MUST accept a concurrency limit and MUST reject **or** queue validation steps that exceed the limit — should it reject or queue? → A: Queue with bounded timeout. Requests beyond `max_concurrency` enter a FIFO queue up to `max_queue_size`. Each queued request records enqueue timestamp, queue position, and configured timeout. If a slot opens before the timeout, execution starts normally. If the timeout expires, the request is rejected with a structured `browser_concurrency_timeout` finding. When the queue is full, reject immediately with `browser_queue_full`. Cancellation of the parent task/run/session removes the queued request with `cancelled_before_start`. Queue waiting time appears in telemetry. Timed-out and queue-full requests are not reported as browser validation failures (the browser check was never executed). Retry is owned by the calling orchestrator.
- Q: How does Boundline discover and configure the browser provider? → A: Through the existing external capability provider registration and activation surface (`[providers.<id>]` in `.boundline/config.toml`). Configuration declares: stable provider ID, capability kind (`browser`), transport (`stdio`), executable command, arguments, working directory, explicitly inherited environment variables (allowlist, not full parent env), startup timeout, execution timeout, and permission envelope. Boundline must not auto-enable a provider merely because a matching executable exists on PATH — the operator explicitly registers and activates it. Before activation, Boundline invokes existing provider capability and health checks. The provider must advertise supported browser capabilities (URL navigation, readiness locators, screenshot, console, network-failure, DOM inspection, accessibility hooks). Missing required capabilities block activation with an explicit finding. Secrets use existing secret handles, never embedded in config. Provider command and env policy are visible in inspect with sensitive values redacted. Launch failure is a provider setup/health failure, not a browser validation failure. V1 uses JSON over stdio; future transports go through the existing protocol. This feature owns only browser-specific requests, findings, and evidence normalization; provider lifecycle remains owned by the capability provider protocol.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Browser Validation Step (Priority: P1)

As a delivery operator, I want to invoke a browser provider as a bounded validation step within an engineering workflow so that I can capture screenshots, console errors, and a normalized evidence packet for a single URL without leaving the Boundline execution model.

**Why this priority**: This is the minimum viable slice — one bounded browser check that produces structured evidence. It proves the provider protocol integration works end-to-end and delivers immediate value for frontend validation workflows that today have no Boundline coverage.

**Independent Test**: Configure a Playwright-backed browser provider, run `boundline validate browser --url http://localhost:3000`, and verify that a screenshot artifact, console log, and a normalized evidence packet are produced and linked in the session trace.

**Acceptance Scenarios**:

1. **Given** a configured browser provider with a reachable URL, **When** Boundline dispatches a validation step to the provider, **Then** the provider opens the URL, captures a full-page screenshot, collects all console messages (errors, warnings, logs), and returns a structured evidence packet containing the screenshot path, console summary, page title, and HTTP status.
2. **Given** a browser validation step that encounters a page load error (timeout, connection refused, invalid URL, certificate error), **When** the provider fails to load the target page, **Then** the provider returns a failure finding with the error category, the failing URL, and any partial console output captured before the failure, and Boundline surfaces the failure in trace and inspect output without crashing the session.
3. **Given** a browser validation step that loads successfully but the page emits JavaScript console errors, **When** the provider captures console output, **Then** the evidence packet includes a structured list of console entries with severity level, message text, and source location when available, and console errors are flagged as findings with a `console_error` category.
4. **Given** a browser provider configured with a network permission policy that restricts outbound requests to an allowlist of domains, **When** the page attempts to reach a disallowed domain, **Then** the provider records a network-access violation finding without blocking the overall validation step, and the finding is visible in the evidence packet.
5. **Given** a browser provider invoked without an explicit URL or with a malformed URL, **When** Boundline dispatches the step, **Then** the provider returns a configuration error finding before attempting navigation, and the session does not enter an unrecoverable state.
6. **Given** a successful browser validation step, **When** the evidence packet is produced, **Then** the packet includes a provider identifier, step duration, artifact references (screenshot, console log, network summary), and a normalized finding disposition (pass / fail_with_findings / error).

---

### User Story 2 - DOM Inspection And Accessibility Checks (Priority: P2)

As a delivery operator, I want the browser provider to inspect DOM state and run accessibility checks so that I can catch structural regressions and a11y violations that code review and unit tests cannot detect.

**Why this priority**: Accessibility and DOM inspection add validation depth beyond basic page-load checks. They are second because screenshot-and-console evidence (P1) delivers the core value; inspection enriches it.

**Independent Test**: Configure a browser provider, run a validation step targeting a page with known accessibility violations, and verify that the evidence packet includes DOM snapshot data and a structured accessibility finding listing violated rules, affected elements, and severity.

**Acceptance Scenarios**:

1. **Given** a browser provider with accessibility scanning enabled, **When** the page loads successfully, **Then** the provider runs an accessibility audit (e.g., axe-core) and includes findings for each violated rule with rule identifier, impact level, element selector, and description in the evidence packet.
2. **Given** a browser provider with DOM inspection enabled, **When** the page loads successfully, **Then** the provider captures a subset of the DOM (configurable root selector, depth limit) and includes it in the evidence packet for structural validation.
3. **Given** a page with no accessibility violations, **When** the accessibility audit completes, **Then** the evidence packet reports zero violations explicitly rather than omitting the accessibility findings section.
4. **Given** an accessibility scan that times out or fails to inject its runtime, **When** the provider encounters the failure, **Then** it records an `accessibility_scan_failed` finding without blocking the rest of the evidence collection, and the finding includes the failure reason.

---

### User Story 3 - Scripted Interactions And Baseline Comparison (Priority: P3)

As a delivery operator, I want the browser provider to execute scripted interactions (click, type, navigate) and compare screenshots against a stored baseline so that I can detect visual regressions in multi-step UI flows.

**Why this priority**: Interaction scripts and visual diff are the most advanced capabilities. They depend on stable screenshot capture (P1) and benefit from DOM inspection (P2) for debugging diffs. They are third because basic evidence collection delivers standalone value.

**Independent Test**: Define a scripted interaction sequence (navigate, click button, type text, submit), run the provider against a target app, and verify that the provider captures a screenshot after each step and compares the final screenshot against a stored baseline, producing a pass/diff/failure finding.

**Acceptance Scenarios**:

1. **Given** a browser provider with a scripted interaction sequence, **When** the provider executes each step in order, **Then** it captures a screenshot after each step, records step duration and success/failure, and produces a step-by-step evidence trail.
2. **Given** a stored baseline screenshot for a validation step, **When** the provider captures a new screenshot and a visual difference exceeds the configured threshold, **Then** the provider records a `visual_diff_detected` finding that includes the diff percentage, a diff image artifact reference, and the baseline identifier.
3. **Given** no stored baseline for a validation step, **When** the provider captures a screenshot, **Then** it records the screenshot as the initial baseline with a `baseline_created` finding rather than reporting a diff failure, and the operator is informed that future runs will compare against this baseline.
4. **Given** a scripted interaction step that fails (element not found, timeout, navigation error), **When** the provider encounters the failure, **Then** it halts the script at the failing step, captures the current page state as evidence, and reports the failure with the step index and the selector or action that failed.
5. **Given** a visual diff that falls within the configured tolerance threshold, **When** the provider compares screenshots, **Then** it reports a pass finding with the actual diff percentage and does not flag it as a regression.

---

### Edge Cases

- What happens when the browser binary is not installed or is an incompatible version?
- How does the system handle a page that triggers an infinite reload or navigation loop?
- What happens when the target URL redirects to a different origin not covered by the network permission policy?
- How does the provider handle pages that require authentication — should it support cookie injection, header passthrough, or neither in V1?
- What happens when the screenshot artifact exceeds a configured size limit?
- How does the system behave when the browser process is killed or crashes mid-validation?
- What happens when the evidence packet cannot be serialized (e.g., binary screenshot data corruption)?
- How does the provider handle pages with file downloads triggered on load?
- What happens when concurrent browser validation steps are attempted against the same provider instance?
- How does the system handle pages that use client-side rendering where meaningful content appears only after JavaScript execution completes?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST support a browser capability provider that communicates with Boundline through the existing external capability provider protocol (S10). The provider MUST be configured through the existing provider registration surface (`[providers.<id>]` in `.boundline/config.toml`) with a stable provider ID, capability kind (`browser`), transport (`stdio`), executable command, arguments, optional working directory, explicitly inherited environment variables (allowlist), startup timeout, execution timeout, and permission envelope. Boundline MUST NOT auto-enable a provider based solely on PATH presence — the operator must explicitly register and activate it. Before activation, Boundline MUST invoke existing provider capability and health checks. Missing required capabilities MUST block activation with an explicit finding. Launch failure of the configured command MUST be reported as a provider setup or health failure, not a browser validation failure.
- **FR-002**: The browser provider MUST advertise the specific browser capabilities it supports upon activation, including at minimum: bounded URL navigation, readiness locator support, screenshot capture, console capture, network-failure capture, DOM inspection, and accessibility hooks. Capability advertisement MUST use the existing provider capability declaration mechanism. The evidence packet and trace output MUST record which provider capabilities were active during the validation step.
- **FR-003**: The browser provider MUST accept a target URL and produce a structured evidence packet containing at minimum a screenshot artifact reference, console output summary, page title, and HTTP status.
- **FR-004**: The evidence packet MUST include a provider identifier, step start and end timestamps, artifact references, and a normalized step status (`completed`, `failed`, `timed_out`, `provider_error`, `cancelled`, `queue_timeout`, `queue_full`) matching the `StepStatus` enum defined in the data model.
- **FR-005**: Console output MUST be captured as structured entries with severity level (error, warning, log, info, debug), message text, and source location (URL, line, column) when available.
- **FR-006**: Console entries with severity `error` MUST be automatically flagged as findings with category `console_error`.
- **FR-007**: The provider MUST handle page-load failures (timeout, connection refused, invalid URL, TLS error) by returning an error finding with the failure category and partial console output rather than crashing or hanging.
- **FR-008**: The provider MUST respect a network permission policy that defines an allowlist of permitted outbound domains; requests to non-allowlisted domains MUST be recorded as `network_access_violation` findings without blocking the overall step.
- **FR-009**: The provider MUST support a configurable page-load timeout and MUST abort navigation when the timeout is exceeded, returning a `page_load_timeout` finding.
- **FR-010**: The provider MUST write all browser artifacts under the session-scoped path `.boundline/sessions/<id>/browser/<validation_run_id>/`, grouped by kind (screenshots, console logs, network logs, DOM snapshots, accessibility outputs, evidence packet). Artifact references in the evidence packet MUST be workspace-relative paths. The normalized evidence packet MUST record for each artifact: kind, relative path, content hash, media type, byte size, creation timestamp, retention class, and producing validation run identifier.
- **FR-011**: The provider MUST write console log artifacts as structured JSON or JSONL files referenced in the evidence packet.
- **FR-012**: Browser validation findings MUST be normalized into Boundline structured findings compatible with session trace and inspect output.
- **FR-013**: The provider MUST support an optional accessibility audit using an injected accessibility engine; accessibility findings MUST include rule identifier, impact level, element selector, and description.
- **FR-014**: When no accessibility violations are detected, the evidence packet MUST explicitly report zero violations.
- **FR-015**: When the accessibility audit fails to execute (timeout, injection failure), the provider MUST record an `accessibility_scan_failed` finding without blocking other evidence collection.
- **FR-016**: The provider MUST support capture of a configurable DOM subset identified by root CSS selector and maximum depth, and include the serialized DOM in the evidence packet.
- **FR-017**: The provider MUST support scripted interaction sequences where each step defines an action type (navigate, click, type, wait, screenshot), a target selector or URL, and optional parameters (text, timeout).
- **FR-018**: When a scripted interaction step fails, the provider MUST halt the script at the failing step, capture the current page state as evidence, and report the failure with step index and failure reason.
- **FR-019**: The provider MUST support screenshot comparison against a stored baseline image; visual differences exceeding a configurable threshold MUST produce a `visual_diff_detected` finding with diff percentage and diff image artifact reference.
- **FR-020**: When no baseline exists for a validation step, the provider MUST create the initial baseline and report a `baseline_created` finding rather than a diff failure.
- **FR-021**: Browser automation MUST be implemented as an external provider communicating over the capability protocol; browser runtime, automation library, and driver binaries MUST NOT be embedded in the Boundline core runtime.
- **FR-022**: The provider MUST enforce a configurable maximum number of concurrent browser executions (`max_concurrency`). Requests beyond this limit MUST enter a bounded FIFO queue with a configurable maximum size (`max_queue_size`) and a per-request timeout (`queue_timeout_seconds`). Each queued request MUST record an enqueue timestamp, queue position when observable, the configured queue timeout, and the originating validation step and session references. If a slot becomes available before the timeout, execution starts normally. If the timeout expires, the request MUST be rejected with a structured `browser_concurrency_timeout` finding. When the queue is full, requests MUST be rejected immediately with a `browser_queue_full` finding. Queue timeout MUST be independent from browser execution timeout. Cancellation of the parent task, execution run, or session MUST remove a queued request and record `cancelled_before_start`. Queue waiting time MUST appear in telemetry and trace output. Timed-out and queue-full requests MUST NOT be reported as browser validation failures because the browser check was never executed.
- **FR-023**: The provider MUST emit a structured retryability hint on each finding when environmental conditions that may justify a retry are observed during the validation step. The retryability hint MUST include a level (`not_indicated`, `possible`, `likely`, `unknown`), an environmental category (`network_transient`, `resource_contention`, `browser_process_failure`, `provider_unavailable`, `queue_timeout`, `environment_startup_delay`), the evidence that caused the hint, and the timing context. A retryability hint MUST NOT suppress, downgrade, or replace the original finding disposition, and MUST NOT turn a failed validation into a passed or inconclusive result. The provider MUST NOT classify application-level findings (selector never appears, JS exception from the page, accessibility violation, failed functional assertion, HTTP 4xx from the tested application, visual or DOM mismatch) as retryable. A finding MUST become `confirmed_intermittent` only after multiple execution attempts produce inconsistent outcomes under equivalent conditions. The provider MUST NOT perform retries itself — retry decisions are owned by the calling orchestrator or operator.
- **FR-024**: The provider MUST support an optional configurable readiness locator that declares a page condition to wait for before capturing screenshots, inspecting the DOM, or running accessibility checks. The readiness locator MUST support CSS selector, test ID, accessible role and name, and text locator types, and MUST accept a declared expected state (`attached`, `visible`, `hidden`, `detached`) with a configurable timeout. When the readiness timeout expires, the provider MUST produce a `browser_readiness_timeout` finding with a diagnostic screenshot and available console/network evidence. A readiness timeout MUST be recorded as a blocking finding distinct from a failed functional assertion. When no readiness condition is configured, the provider MAY fall back to the browser `load` event but MUST record that application-specific readiness was not configured. The provider MUST NOT use `networkidle` as the sole or default readiness criterion. Fixed delays MAY be used only as an explicit secondary stabilization delay applied after the readiness condition has already succeeded. Pages that continuously poll, stream, or maintain WebSocket connections MUST remain testable.
- **FR-025**: Evidence packets and findings MUST be linkable to Canon verification packets through a stable step identifier and artifact reference scheme.
- **FR-026**: Browser artifacts MUST follow the existing Boundline session archive and retention policy. Archiving a session MAY compact or move artifacts while preserving valid references. Removing a session MUST NOT delete browser artifacts when a durable verification, governance, or audit record still references them. Evidence promoted into a Canon verification packet MUST either remain retained with the archived session or be copied into the existing durable evidence store. Cleanup MUST never leave a successful proof record pointing to missing artifacts without an explicit `artifact_unavailable` finding. Secrets, credentials, tokens, cookies, and sensitive request or response data MUST be redacted before durable artifact storage. Large optional artifacts (full DOM snapshots, network traces) MAY use shorter retention classes (`diagnostic`, `verbose`, `ephemeral`) than blocking screenshots or final evidence packets (`required_evidence`). Retention policy MUST reuse Boundline's existing trace and session retention model rather than introducing a browser-specific cleanup subsystem.

### Key Entities *(include if feature involves data)*

- **Browser Validation Step**: A bounded unit of work dispatched to the browser provider, containing a target URL, optional readiness locator (type, value, expected state, timeout, stabilization delay), optional interaction script, optional accessibility flag, optional baseline reference, and timeout configuration.
- **Evidence Packet**: The structured result of a browser validation step, containing provider identifier, timestamps, finding disposition, artifact references (screenshot, console log, DOM snapshot, network summary), and a list of structured findings.
- **Browser Finding**: A normalized validation finding with category (see FR-023 for environmental categories and retryability levels), severity, description, optional artifact reference, and optional retryability hint (level, environmental category, evidence, timing context). Retryability hints are advisory and do not change the finding disposition. When a finding has been confirmed intermittent through multiple inconsistent outcomes, it carries a `confirmed_intermittent` flag.
- **Interaction Script**: An ordered sequence of browser actions (navigate, click, type, wait, screenshot) with per-step selectors, parameters, and timeouts.
- **Visual Baseline**: A stored reference screenshot associated with a validation step identifier, used as the comparison target for visual diff detection.
- **Network Permission Policy**: An allowlist of permitted outbound domains that the browser provider enforces at the network level; requests to non-allowlisted domains produce findings without blocking the step.

- **Concurrency Policy**: The provider-level configuration controlling (`max_concurrency`), (`max_queue_size`), (`queue_timeout_seconds`), and (`execution_timeout_seconds`). Queue semantics are FIFO with bounded waiting; queue-full rejects immediately; timeout rejects with a distinct finding category.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A browser validation step targeting a reachable, error-free URL completes and produces a valid evidence packet with screenshot, console log, page title, and HTTP status within 30 seconds under normal network conditions.
- **SC-002**: 100% of page-load failure scenarios (timeout, connection refused, invalid URL, TLS error) return a categorized error finding rather than causing an unrecoverable session state or provider hang.
- **SC-003**: 100% of JavaScript console messages at severity `error` and `warning` captured by the provider are surfaced as structured findings in the session trace and inspect output, with severity, message, and source location preserved. Messages at `log`, `info`, and `debug` severity are available in the console log artifact but are not required to be flagged as individual findings.
- **SC-004**: An accessibility audit of a page with known WCAG violations produces findings for at least 90% of the injected engine's detected violations, with rule identifiers and element selectors matching the engine's native output.
- **SC-005**: Visual diff detection correctly identifies a screenshot difference exceeding the configured threshold in 100% of controlled comparison tests, and correctly reports no diff when images are identical.
- **SC-006**: A scripted interaction sequence of up to 10 steps completes without provider-side timeout or resource exhaustion under normal page-load conditions, and each step failure is reported with the correct step index and failure reason.
- **SC-007**: The provider does not make any outbound network request to a non-allowlisted domain when a network permission policy is active, and all blocked requests are recorded as findings in 100% of policy-enforcement tests.
- **SC-008**: Evidence packets and individual findings can be linked to a Canon verification packet through a stable, non-colliding identifier scheme in 100% of linked-validation scenarios.

## Assumptions

- The existing external capability provider protocol (S10, seed 07) supports the message schemas needed for browser provider dispatch, evidence return, and finding normalization without protocol-level changes.
- Playwright or an equivalent browser-automation library is available as a separately installed dependency managed by the operator; Boundline does not bundle or auto-install browser binaries.
- A single browser provider instance handles one validation step at a time; concurrent steps require separate provider instances or explicit queuing.
- Screenshot baselines are stored as versioned artifacts in the session trace directory or a configurable baseline store; the provider does not manage baseline lifecycle beyond creation and comparison.
- The first delivery slice targets a single URL validation with screenshot and console capture; DOM inspection, accessibility scanning, interaction scripts, and visual diff are additive capabilities that can be delivered in later slices.
- Network permission policy is expressed as a static allowlist in provider configuration; dynamic policy resolution (per-session, per-task, per-zone) is deferred.
- The browser provider runs on the same machine as Boundline in V1; remote or containerized browser execution is a future concern.
- Authentication for target pages (cookies, headers, OAuth) is out of scope for the first slice; pages requiring authentication may be handled by a pre-configured browser profile in a later iteration.
