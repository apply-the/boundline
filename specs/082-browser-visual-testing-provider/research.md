# Research: Browser And Visual Testing Provider

**Feature Branch**: `082-browser-visual-testing-provider` | **Date**: 2026-06-20

## 1. Existing S10 Capability Provider Protocol

**Finding**: `src/domain/capability_provider.rs` defines `CapabilityProviderTransportKind` (Command, Http), `CapabilityProviderRegistrationSource`, `CapabilityProviderDiscoveryState`, and `CapabilityProviderActivationState`. The protocol line `capability-provider-v1` supports versioned additive fields. Transport `Command` means a local subprocess — a natural fit for JSON-over-stdio.

**Decision**: The browser provider uses `kind = "browser"` with `transport = "command"` (subprocess over stdio). Boundline spawns the configured provider binary, sends a JSON request via stdin, reads a JSON response from stdout, and captures stderr for diagnostics. No new transport type needed — `Command` already supports the stdio pattern. The `browser` capability kind is registered as a new variant in the existing provider type system with capability advertisement via the existing declaration mechanism.

**References**: `src/domain/capability_provider.rs`, `src/adapters/provider_runtime.rs`

## 2. JSON Stdio Message Format

**Finding**: The spec defines provider capabilities through a JSON request/response contract. The request carries a target URL, readiness locator, timeouts, and capability flags. The response carries an evidence packet with findings, artifact references, and timing metadata.

**Decision**: Adopt a request/response envelope pattern compatible with existing Boundline structured types:
- Request: `{ "validation_run_id", "url", "readiness", "interaction_script", "accessibility", "baseline_ref", "timeouts", "network_allowlist", "artifact_dir" }`
- Response: `{ "validation_run_id", "status", "evidence_packet", "findings", "timing", "retryability_hints" }`
- The message schema is documented in `contracts/browser-provider-protocol.md`
- Both request and response are versioned (`schema_version = 1`) for forward compatibility
- Error responses use the same envelope with `status = "error"` and an error finding

## 3. Artifact Hashing

**Finding**: FR-010 requires content hashes on every artifact record. The hash enables deduplication, integrity verification, and stable cross-reference.

**Decision**: Use SHA-256 for artifact content hashing. The hash is computed at artifact write time and embedded in the evidence packet's artifact record. Hash verification is best-effort — a missing or mismatched hash produces an `artifact_integrity` diagnostic finding but does not block evidence consumption. Content-addressable references allow artifacts to survive renames and moves within the session directory.

## 4. Finding Normalization

**Finding**: The spec defines 12+ finding categories, each with severity, description, optional artifact reference, and optional retryability hint. These must serialize into a format compatible with Boundline's existing structured findings.

**Decision**: Browser findings use a flat JSON object per finding with typed `kind` and `severity` fields. Retryability hints are embedded as an optional sub-object. The finding schema intentionally differs from Boundline's internal `StructuredRuntimeEvent` payloads — the browser provider's output is a provider-specific contract, not a core trace event. The adapter layer in `browser_provider_runtime.rs` maps provider findings into Boundline's trace event model when writing to the trace store.

## 5. Provider Concurrency Model

**Finding**: The spec requires FIFO queuing with configurable max concurrency, max queue size, and queue timeout. The provider manages its own queue internally — Boundline does not implement a cross-provider queue.

**Decision**: Concurrency enforcement lives inside the provider binary. Boundline only asserts the configured limits through provider configuration. The provider advertises its current queue depth and active concurrency on each response so Boundline can surface queue state in status output. If the provider exits unexpectedly, Boundline treats all queued requests as failed with `provider_unavailable`.

## 6. Existing Session Persistence

**Finding**: `ActiveSessionRecord` in `src/domain/session.rs` supports additive field extension via `#[serde(default)]`. Session-scoped artifact directories already exist under `.boundline/sessions/<id>/`.

**Decision**: Browser validation runs are tracked as additive fields on the session record (a `Vec` of browser validation run references). Each reference points to the evidence packet path. Artifact lifecycle follows the existing session archive/retention policy. No new persistence backend is needed.
