# Data Model: Evals And Runtime Observability

**Feature**: 072-evals-runtime-observability
**Date**: 2026-06-05

## Entities

### StructuredRuntimeEvent

A typed, timestamped record of a runtime state transition.

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | `Uuid` | Unique identifier for this event |
| `event_type` | `EventType` (enum) | Discriminates the event kind |
| `schema_version` | `&'static str` | Per-event-type version (e.g., `"1.0"`) |
| `timestamp` | `DateTime<Utc>` | When the event was emitted |
| `session_id` | `Uuid` | Owning session |
| `trace_ref` | `Option<String>` | Link to the source trace item |
| `payload` | `EventPayload` (enum) | Type-specific data |

**EventType variants** (initial set):
- `PlanningAnalysisCompleted`
- `GuardianFindingEmitted`
- `ProviderCallCompleted`
- `PhaseRequested`
- `RouteDecisionMade`
- `ContextSelectionRecorded`
- `TraceCompacted`

**EventPayload variants** mirror `EventType` one-to-one with typed inner structs.

**Validation rules**:
- `event_id` must be unique within a session.
- `schema_version` must be a non-empty string following `MAJOR.MINOR` format.
- `timestamp` must be monotonically non-decreasing within a session's event stream.
- Additive fields may be added within the same major `schema_version`. Field removal or meaning change requires a new major version.

**State transitions**: Events are append-only. Once emitted, an event is immutable.

---

### TraceCompactionPolicy

The rule set mapping trace item types to retention classes.

| Field | Type | Description |
|-------|------|-------------|
| `policy_version` | `&'static str` | Version of the compaction policy (e.g., `"trace-compaction-v1"`) |
| `class_table` | `HashMap<String, RetentionClass>` | Item type → retention class mapping |

**RetentionClass** (enum):
- `Lossless` — exact preservation required
- `Structured` — normalized into a structured event record
- `Summary` — replaced with a lossy summary, source refs retained
- `IndexOnly` — reduced to searchable metadata only
- `Discardable` — removable under retention policy

**Validation rules**:
- Every known item type must map to exactly one `RetentionClass`.
- Unknown item types are classified by conservative tiebreaking (stricter class wins).
- `Lossless` classification cannot be overridden for accepted decisions, rejection reasons, or active stage evidence.

**State transitions**: The policy is immutable per version. A new policy version may be introduced in a future release.

---

### CompactionAction

A record of one trace item transformation during compaction.

| Field | Type | Description |
|-------|------|-------------|
| `item_ref` | `String` | Identifier of the trace item |
| `from_class` | `RetentionClass` | Original classification |
| `to_class` | `RetentionClass` | Target classification after compaction |
| `lossy` | `bool` | Whether the transformation is lossy |
| `tiebreak` | `bool` | Whether classification was resolved by conservative tiebreaking |

**Validation rules**:
- `from_class` must not be `Discardable` for active stage evidence.
- `lossy` must be `true` when `to_class` is `Summary` (relative to a non-Summary `from_class`).
- `tiebreak` must be `true` only when the item type was not found in the `class_table`.

**Relationships**: One `CompactionAction` belongs to exactly one `CompactionEvent`.

---

### CompactionEvent

A structured event emitted after every compaction run.

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | `Uuid` | Inherited from `StructuredRuntimeEvent` |
| `event_type` | `EventType::TraceCompacted` | Fixed event type |
| `policy_version` | `String` | Version of the policy used |
| `source_trace` | `String` | Identifier of the compacted trace |
| `actions` | `Vec<CompactionAction>` | All actions taken |
| `preserved_refs` | `Vec<String>` | References that survived exactly |
| `metrics` | `CompactionMetrics` | Before/after statistics |

**Validation rules**:
- `actions` must not be empty (a no-op compaction is still a valid event with zero changes).
- `preserved_refs` must include all `Lossless` items from the source trace.

---

### CompactionMetrics

Counters recorded during a compaction run.

| Field | Type | Description |
|-------|------|-------------|
| `compaction_count` | `u64` | Monotonic counter for this session |
| `class_distribution` | `HashMap<RetentionClass, u64>` | Items per class after compaction |
| `trace_size_before_bytes` | `u64` | Trace size before compaction |
| `trace_size_after_bytes` | `u64` | Trace size after compaction |
| `lossy_count` | `u64` | Number of lossy transformations |
| `preserved_decision_count` | `u64` | Accepted decisions preserved |
| `preserved_rejection_count` | `u64` | Rejection reasons preserved |

---

### EvalFixture

A test case validating a specific quality dimension.

| Field | Type | Description |
|-------|------|-------------|
| `eval_id` | `String` | Unique identifier (e.g., `"planning-quality-01"`) |
| `eval_name` | `String` | Human-readable name |
| `dimension` | `EvalDimension` (enum) | Quality dimension being tested |
| `fixture_ref` | `String` | Path to the session fixture or trace, relative to `.boundline/evals/fixtures/`. No absolute paths in committed fixtures. Session-scoped references must be explicit (e.g., `"sessions/blocked-planning.json"`). Missing fixture path fails validation with an actionable message before eval execution. |
| `expected_outcome` | `String` | What the eval expects to find |

**EvalDimension** variants (initial set):
- `PlanningQuality`
- `ContextSelectionQuality`
- `CriticalContextOmission`
- `GuardianFindingQuality`
- `CouncilRejectionBehavior`
- `ProviderCallFailureHandling`
- `CompactionSurvivalDecisions`
- `CompactionSurvivalRejections`

---

### EvalResult

The outcome of a single eval run.

| Field | Type | Description |
|-------|------|-------------|
| `eval_id` | `String` | Matches the fixture's `eval_id` |
| `eval_name` | `String` | From the fixture |
| `status` | `EvalStatus` (enum) | `Pass` or `Fail` |
| `failure_reason` | `Option<String>` | Explanation when failed |
| `source_refs` | `Vec<String>` | Fixture and trace references |
| `expected_outcome` | `String` | From the fixture |
| `actual_outcome` | `String` | What was observed |
| `duration_ms` | `u64` | Execution time in milliseconds |

---

### EvalSummary

The aggregate result of an eval suite run.

| Field | Type | Description |
|-------|------|-------------|
| `suite_status` | `EvalStatus` | AND of all required eval results |
| `results` | `Vec<EvalResult>` | Per-eval results |
| `total_count` | `u64` | Total evals run |
| `pass_count` | `u64` | Evals that passed |
| `fail_count` | `u64` | Evals that failed |
| `duration_ms` | `u64` | Total suite execution time |

**Validation rules**:
- `suite_status` is `Pass` only when all required evals pass.
- `total_count` must equal `pass_count + fail_count`.

---

### RuntimeMetrics

A snapshot of counters captured during a session.

| Field | Type | Description |
|-------|------|-------------|
| `compaction_count` | `u64` | Number of compaction runs |
| `compaction_class_distribution` | `HashMap<RetentionClass, u64>` | Items per class |
| `trace_size_before_bytes` | `u64` | Most recent before size |
| `trace_size_after_bytes` | `u64` | Most recent after size |
| `lossy_compaction_count` | `u64` | Lossy transformations |
| `preserved_decision_count` | `u64` | Decisions preserved |
| `preserved_rejection_count` | `u64` | Rejections preserved |
| `context_size_bytes` | `u64` | Assembled context size |
| `context_item_count` | `u64` | Number of context items |
| `provider_latency_ms` | `u64` | Provider call latency |
| `stop_reason` | `Option<String>` | Why execution stopped |
| `finding_count` | `u64` | Total findings emitted |

## Entity Relationships

```
EvalFixture 1 ──── * EvalResult
EvalSummary 1 ──── * EvalResult
TraceCompactionPolicy 1 ──── * CompactionAction
CompactionEvent 1 ──── * CompactionAction
CompactionEvent 1 ──── 1 CompactionMetrics
StructuredRuntimeEvent 1 ──── 1 EventPayload
RuntimeMetrics (standalone snapshot, no FK relationships)
```
