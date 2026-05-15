# Mobile Application Guidance

## Purpose

This guidance defines mobile application practices for AI-assisted planning, architecture, implementation, testing, review, and release readiness.

It applies to iOS, Android, React Native, Flutter, Kotlin Multiplatform, and other mobile client systems.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, mobile platform policy, release policy, or Canon-governed standard.

## Core Thesis

Mobile changes are distributed software changes.

Once released, clients may remain in the wild for weeks or months.

Mobile work must account for:

- version skew
- offline behavior
- permissions
- privacy
- app lifecycle
- network variability
- backward compatibility
- release rollout
- crash observability
- store review and rollback constraints

## Client/Server Compatibility

Mobile clients cannot be updated instantly.

Before changing API behavior, consider:

- old clients still calling new backend
- new clients calling old backend during rollout
- feature flags
- schema compatibility
- optional fields
- graceful degradation
- server-side kill switches
- minimum supported app version

Guardians should flag server changes that assume immediate mobile update.

## Offline And Network Handling

Mobile networks are unreliable.

Handle:

- offline mode
- slow network
- request cancellation
- retry safety
- partial sync
- stale cached data
- conflict resolution
- background sync constraints

Avoid:

- assuming every request succeeds
- losing user input on connectivity failure
- retrying writes without idempotency
- treating cache as always fresh

## Permissions And Privacy

Mobile apps handle sensitive permissions.

Check:

- location
- camera
- microphone
- contacts
- photos
- notifications
- background activity
- tracking identifiers

Rules:

- request permission only when needed
- explain user value
- handle denial
- avoid permission prompts at app start unless justified
- avoid collecting more data than needed

## App Lifecycle

Mobile apps are interrupted.

Handle:

- backgrounding
- foregrounding
- process death
- state restoration
- token refresh
- network reconnect
- push notification entry
- deep links

AI-generated UI code often assumes uninterrupted foreground sessions.

## State Management

Keep state ownership explicit.

Classify:

- view state
- navigation state
- cached server state
- persisted local state
- sync queue state
- authentication/session state

Avoid:

- global mutable state for everything
- local persistence without migration strategy
- business rules duplicated across screens
- untestable state machines

## Security

Mobile security concerns:

- secure token storage
- certificate pinning where policy requires
- avoiding secrets in app bundle
- jailbreak/root assumptions
- deep link validation
- local data encryption where needed
- PII in logs/crash reports

Never rely on client-only enforcement for authoritative security.

## Release Readiness

Mobile release requires:

- feature flag or staged rollout plan for risky work
- crash monitoring
- analytics or operational signals
- backward compatibility notes
- store review considerations
- rollback or kill-switch plan
- minimum version policy

## Testing Guidance

Recommended:

- unit tests for view models/state machines
- integration tests for API/client boundary
- UI tests for critical flows
- offline/network simulation
- permission denial tests
- deep link tests
- upgrade/migration tests
- crash-free startup tests

Avoid:

- only testing happy path with perfect network
- ignoring permission denial
- no tests for app lifecycle interruption
- no tests for stale cache or sync conflict
- relying solely on manual QA for critical flows

## Anti-Patterns

- assuming instant client upgrades
- breaking API compatibility for old clients
- retrying non-idempotent writes
- permission prompt without context
- secrets embedded in client
- PII in logs/crash reports
- unbounded local cache
- offline input loss
- global mutable client state
- no crash/analytics signal for new critical flow
- release without kill switch for risky feature

## Guardian Hooks

Recommended guardians:

- mobile-compatibility-guardian
- offline-state-guardian
- mobile-permission-guardian
- mobile-privacy-guardian
- app-lifecycle-guardian
- mobile-release-readiness-guardian
- mobile-secure-storage-guardian
- mobile-api-version-skew-guardian

## Structured Finding Example

```json
{
  "guardian": "mobile-api-version-skew",
  "rule": "backend-change-assumes-immediate-client-update",
  "disposition": "blocker",
  "summary": "The API removes a response field still used by currently supported mobile clients.",
  "evidence_refs": ["api/orders/schema.json", "ios/Orders/OrderViewModel.swift"],
  "recommended_action": "Preserve the field through a compatibility window or add versioned response handling."
}
```

## Lifecycle Usage

Planning:
- identify version skew, offline, permissions, and rollout risks

Architecture:
- define client/server compatibility and state ownership

Implementation:
- guide lifecycle, secure storage, and network handling

Testing:
- verify offline, permissions, upgrade, and lifecycle behavior

Review:
- check privacy, compatibility, retry safety, and release readiness

Release:
- verify rollout, monitoring, and kill-switch posture
