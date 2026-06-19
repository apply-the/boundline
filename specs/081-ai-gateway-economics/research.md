# Research: AI Gateway And Inference Economics

**Feature Branch**: `081-ai-gateway-economics` | **Date**: 2026-06-18

## 1. Existing Session Persistence Surface

**Finding**: `ActiveSessionRecord` (src/domain/session.rs) is the primary session type, persisted to `.boundline/session.json`. It supports additive field extension via `#[serde(default, skip_serializing_if = "...")]`.

**Decision**: Inference economics fields (budget limit, known spent, reserved, remaining known budget, unknown-cost call count, pricing snapshot identifier, cost basis, budget state) will be added as optional fields on `ActiveSessionRecord` and projected through `SessionStatusView`. No structural schema migration needed; existing sessions without economics fields will default to `None` (budget enforcement disabled).

**References**: `tests/unit/session_record.rs` shows existing deserialization of additive fields like plan-quality and completion-verification.

## 2. Existing Provider Protocol

**Finding**: `CapabilityProviderRegistration` in `src/domain/capability_provider.rs` models provider registration with transport descriptors, capability declarations, and health snapshots. The protocol line `capability-provider-v1` supports versioned additive fields.

**Decision**: Route economics metadata (pricing, health, capability tier) will extend `ProviderCapabilityDeclaration` with optional cost-related fields. `ProviderHealthSnapshot` gains optional `cost_quota_status`. A new `InferenceRouteProfile` struct will be introduced in a new module to avoid coupling cost logic to the general provider protocol.

## 3. Existing Trace/Telemetry Surface

**Finding**: `StructuredRuntimeEvent` (src/domain/observability.rs) has a per-event-type `schema_version` field and a flexible `serde_json::Value` payload. Existing event types include `ProviderCallCompleted` (v1.0) and `RouteDecisionMade` (v1.0).

**Decision**: 
- Extend `ProviderCallCompleted` payload with optional cost fields (native currency, normalized currency, cost quality, pricing snapshot identifier, conversion source) — additive within same major version.
- Add new event types: `BudgetStateChanged` (v1.0), `SpendExceptionApprovalRecorded` (v1.0).
- Extend `RouteDecisionMade` payload with cost reservation and snapshot staleness fields.

**References**: `tests/contract/trace_record.rs` validates schema compatibility.

## 4. Authority Zones and Governance Roles

**Finding**: `CanonAuthorityZone` (Green/Yellow/Red/Restricted) maps to stage authority floors. Governance personas include DeliveryEngineer, SystemArchitect, OperationsGovernor. `ControlLevel` (Advisory/Catch/Rule/Hook) determines enforcement strength.

**Decision**: Spend exception approval authority derives from existing roles without introducing new RBAC:
- `SessionOwner` role maps to the active session owner (already tracked in session state).
- `GovernanceApprover` maps to existing OperationsGovernor persona (when `CanonAuthorityGovernanceV1Envelope` is present) or a configurable workspace operator role.
- Red-zone routing enforcement uses existing `authority_zone` field to prevent capability downgrades.

## 5. CLI Command Structure

**Finding**: 30+ subcommands under `DeveloperCommand`, organized by feature area. Provider and config commands are in `src/cli/provider.rs` and `src/cli/config.rs`.

**Decision**: Economics commands will be added under existing subcommands:
- `boundline config set-budget` — configure session budget
- `boundline provider set-pricing` — activate a pricing snapshot
- `boundline status` extension — show budget projection in status output
- `boundline approve` — new subcommand for spend exception approval in interactive mode
- `boundline inspect cost` — inspect cost records and budget history

## 6. Configuration Surface

**Finding**: `ConfigFile` (src/domain/configuration.rs) with versioned TOML sections. Precedence: Built-in → Global → Cluster → Workspace → CLI.

**Decision**: New `[inference_economics]` config section with:
- `session_budget` (currency, limit, staleness_threshold_days)
- `unknown_cost_policy` (block/require_approval/allow_with_warning)
- `pricing_snapshots` (array of snapshot references)
- `approval_default_scope` (single_call/bounded_task/bounded_session)

## 7. Decimal Arithmetic

**Finding**: FR-040 requires exact monetary precision with no floating-point. The Rust ecosystem has `rust_decimal` and `rust_decimal_macros` crates for fixed-point decimal arithmetic.

**Decision**: Add `rust_decimal` as a workspace dependency. All monetary values will use `Decimal` internally with minor-unit precision (e.g., cents). Serialize as decimal strings in JSON/TOML to avoid floating-point precision loss. If the dependency is deemed too heavy, implement a lightweight `MonetaryAmount(i64)` newtype in minor units with explicit conversion from decimal strings.
