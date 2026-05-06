# Data Model: Native Canon CLI Surface

**Feature**: 042-native-canon-cli
**Date**: 2026-05-05

## Entity Diagram

```text
┌──────────────────────────────────┐
│          ConfigFile              │
│──────────────────────────────────│
│  version: u32                    │
│  routing: RoutingConfig          │
│  canon: Option<CanonPreferences> │ ← NEW
└───────────────┬──────────────────┘
                │ contains
                ▼
┌──────────────────────────────────┐
│       CanonPreferences           │ ← NEW
│──────────────────────────────────│
│  mode_selection:                 │
│    CanonModeSelectionPreference  │
│  default_risk: Option<String>    │
│  default_zone: Option<String>    │
│  default_owner: Option<String>   │
│  default_system_context:         │
│    Option<String>                │
└──────────────────────────────────┘

┌──────────────────────────────────┐
│  CanonModeSelectionPreference    │ ← NEW
│──────────────────────────────────│
│  Manual                          │
│  AutoConfirm                     │
│  Auto                            │
└──────────────────────────────────┘

┌──────────────────────────────────┐
│          CanonMode               │ ← EXTENDED
│──────────────────────────────────│
│  Requirements                    │
│  Discovery                       │
│  SystemShaping                   │ ← NEW
│  Architecture                    │
│  Backlog                         │
│  Change                          │
│  Implementation                  │
│  Refactor                        │ ← NEW
│  Review                          │ ← NEW
│  Verification                    │
│  Incident                        │ ← NEW
│  SecurityAssessment              │
│  SystemAssessment                │ ← NEW
│  Migration                       │ ← NEW
│  SupplyChainAnalysis             │ ← NEW
│  PrReview                        │  (existing, kept for backward compat)
└──────────────────────────────────┘

┌──────────────────────────────────────────┐
│        CanonCapabilitySnapshot           │ ← EXISTS, verified
│──────────────────────────────────────────│
│  canon_version: String                   │
│  supported_modes: Vec<CanonMode>         │
│  operations: Vec<String>                 │
│  status_values: Vec<String>              │
│  approval_state_values: Vec<String>      │
│  packet_readiness_values: Vec<String>    │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│        CanonSurfaceVerification          │ ← NEW
│──────────────────────────────────────────│
│  canon_path: PathBuf                     │
│  version_compatible: bool                │
│  operations_verified: bool               │
│  missing_operations: Vec<String>         │
│  modes_verified: bool                    │
│  missing_modes: Vec<CanonMode>           │
│  unsupported_modes: Vec<String>          │
│  capability_snapshot:                    │
│    Option<CanonCapabilitySnapshot>       │
│  ready: bool                             │
│  repair_actions: Vec<String>             │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│        CanonInstallStatus                │ ← EXISTS
│──────────────────────────────────────────│
│  state: CompanionState                   │
│  version: Option<String>                 │
│  location: Option<PathBuf>               │
│  bundled_with_boundline: bool            │
│  message: String                         │
│  suggested_actions: Vec<String>          │
│  surface_verification:                   │
│    Option<CanonSurfaceVerification>      │ ← NEW field
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│        GovernanceIntent                  │ ← EXISTS
│──────────────────────────────────────────│
│  requested: bool                         │
│  runtime_preference:                     │
│    Option<GovernanceRuntimeKind>          │
│  risk: Option<String>                    │
│  zone: Option<String>                    │
│  owner: Option<String>                   │
│  explicit_mode: Option<CanonMode>        │ ← NEW field
│  explicit_no_canon: bool                 │ ← NEW field
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│     ActiveSessionRecord                  │ ← EXISTS, extended
│──────────────────────────────────────────│
│  ...existing fields...                   │
│  governance_lifecycle:                   │
│    Option<GovernedSessionLifecycle>       │ ← NEW field
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│    GovernedSessionLifecycle               │ ← NEW
│──────────────────────────────────────────│
│  governance_runtime: GovernanceRuntimeKind│
│  explicit_opt_out: bool                  │
│  mode_selection_preference:              │
│    CanonModeSelectionPreference          │
│  selected_mode: Option<CanonMode>        │
│  selected_mode_sequence:                 │
│    Vec<CanonMode>                        │
│  current_stage_index: usize              │
│  stage_records:                          │
│    Vec<GovernedStageRecord>              │
│  accumulated_context:                    │
│    Vec<GovernedDocumentRef>              │
│  terminal_reason: Option<String>         │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│    GovernedDocumentRef                    │ ← NEW
│──────────────────────────────────────────│
│  stage_key: String                       │
│  canon_mode: CanonMode                   │
│  packet_ref: String                      │
│  document_path: Option<String>           │
│  readiness: PacketReadiness              │
└──────────────────────────────────────────┘
```

## Entity Details

### CanonPreferences (NEW)

Workspace-local Canon governance configuration, stored in `.boundline/config.toml`
under the `[canon]` section.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `mode_selection` | `CanonModeSelectionPreference` | Yes | `Manual` | How Boundline selects Canon modes for governed stages |
| `default_risk` | `Option<String>` | No | `"standard"` | Default risk level for Canon governance requests |
| `default_zone` | `Option<String>` | No | `"development"` | Default zone for Canon governance requests |
| `default_owner` | `Option<String>` | No | OS user or git user.name | Default owner for Canon governance requests |
| `default_system_context` | `Option<String>` | No | Inferred from mode | Default system context (new/existing) |

**TOML representation**:
```toml
[canon]
mode_selection = "auto-confirm"
default_risk = "standard"
default_zone = "development"
default_owner = "operator"
```

**Validation rules**:
- `mode_selection` must be one of `manual`, `auto-confirm`, `auto`
- `default_risk` and `default_zone` are free-form strings (Canon validates)
- `default_owner` is free-form; if absent, inferred from `$USER` or `git config user.name`

### CanonModeSelectionPreference (NEW)

| Variant | Serialized | Behavior |
|---------|------------|----------|
| `Manual` | `"manual"` | Require explicit `--mode` from operator; in chat, ask which mode |
| `AutoConfirm` | `"auto-confirm"` | Infer mode from evidence, ask operator to confirm before Canon invocation |
| `Auto` | `"auto"` | Infer mode and proceed without confirmation when confidence is high; fall back to confirmation on ambiguity or broad risk |

### CanonMode (EXTENDED)

New variants added to the existing enum.

| Variant | Serialized | Primary Document | Status |
|---------|-----------|------------------|--------|
| `Requirements` | `"requirements"` | `requirements.md` | Existing |
| `Discovery` | `"discovery"` | `discovery.md` | Existing |
| `SystemShaping` | `"system-shaping"` | `system-shaping.md` | **New** |
| `Architecture` | `"architecture"` | `architecture.md` | Existing |
| `Backlog` | `"backlog"` | `backlog.md` | Existing |
| `Change` | `"change"` | `change.md` | Existing |
| `Implementation` | `"implementation"` | `implementation.md` | Existing |
| `Refactor` | `"refactor"` | `refactor.md` | **New** |
| `Review` | `"review"` | `review.md` | **New** |
| `Verification` | `"verification"` | `verification.md` | Existing |
| `Incident` | `"incident"` | `incident.md` | **New** |
| `SecurityAssessment` | `"security-assessment"` | `security-assessment.md` | Existing |
| `SystemAssessment` | `"system-assessment"` | `system-assessment.md` | **New** |
| `Migration` | `"migration"` | `migration.md` | **New** |
| `SupplyChainAnalysis` | `"supply-chain-analysis"` | `supply-chain-analysis.md` | **New** |
| `PrReview` | `"pr-review"` | N/A | Existing (backward compat) |

**Backward compatibility**: `PrReview` is retained.  If Canon unifies `review`
and `pr-review`, Boundline treats `PrReview` as an alias for `Review` in the
capabilities verification step.

### CanonSurfaceVerification (NEW)

Result of verifying the actual Canon governance surface, beyond version check.

| Field | Type | Description |
|-------|------|-------------|
| `canon_path` | `PathBuf` | Absolute path to the Canon binary tested |
| `version_compatible` | `bool` | Whether the version string matches the supported window |
| `operations_verified` | `bool` | Whether required operations (`governance start`, `governance refresh`) are present |
| `missing_operations` | `Vec<String>` | Operations expected but not found |
| `modes_verified` | `bool` | Whether all 15 canonical modes are in `supported_modes` |
| `missing_modes` | `Vec<CanonMode>` | Canonical modes expected but not reported by Canon |
| `unsupported_modes` | `Vec<String>` | Modes reported by Canon that Boundline does not recognize |
| `capability_snapshot` | `Option<CanonCapabilitySnapshot>` | Full capability response if available |
| `ready` | `bool` | `version_compatible && operations_verified && modes_verified` |
| `repair_actions` | `Vec<String>` | Human-readable repair guidance when not ready |

### GovernanceIntent (EXTENDED)

Two new fields on the existing struct.

| Field | Type | Description |
|-------|------|-------------|
| `explicit_mode` | `Option<CanonMode>` | Explicit `--mode` from CLI or chat |
| `explicit_no_canon` | `bool` | Whether `--no-canon` or `--governance local` was passed |

### GovernedSessionLifecycle (NEW)

Persisted in `ActiveSessionRecord` to track the governed journey across session
continuations.

| Field | Type | Description |
|-------|------|-------------|
| `governance_runtime` | `GovernanceRuntimeKind` | `Canon` or `Local` for this session |
| `explicit_opt_out` | `bool` | Whether the operator explicitly chose local governance |
| `mode_selection_preference` | `CanonModeSelectionPreference` | Workspace preference at session creation |
| `selected_mode` | `Option<CanonMode>` | Single mode for the current governed stage |
| `selected_mode_sequence` | `Vec<CanonMode>` | Ordered mode sequence for multi-stage journeys |
| `current_stage_index` | `usize` | Index into `selected_mode_sequence` |
| `stage_records` | `Vec<GovernedStageRecord>` | Per-stage governance results (existing type) |
| `accumulated_context` | `Vec<GovernedDocumentRef>` | Governed documents from prior stages forwarded as bounded context |
| `terminal_reason` | `Option<String>` | Why the governed journey ended (completed, blocked, rejected, abandoned) |

**State transitions**:
```text
  [no lifecycle]
      │ run with Canon-ready workspace
      ▼
  governance_runtime = Canon
  mode_selection_preference = from config
  selected_mode = None (or explicit)
      │ mode selected/confirmed
      ▼
  selected_mode = Some(mode)
  current_stage_index = 0
      │ Canon start returns GovernedReady
      ▼
  stage_records[0] updated
  accumulated_context += governed doc
  current_stage_index += 1
      │ next stage or terminal
      ▼
  [completed | blocked | rejected | abandoned]
```

### GovernedDocumentRef (NEW)

Reference to a governed document produced by a prior stage, used for
`reused_packets` forwarding in multi-stage journeys.

| Field | Type | Description |
|-------|------|-------------|
| `stage_key` | `String` | Flow+stage identifier that produced this document |
| `canon_mode` | `CanonMode` | Canon mode that governed this document |
| `packet_ref` | `String` | Governed packet reference from Canon |
| `document_path` | `Option<String>` | Path to the produced document under `.canon/` if available |
| `readiness` | `PacketReadiness` | Readiness state of the governed packet |

## Relationships

```text
ConfigFile 1──1 CanonPreferences (optional; present after init)
CanonPreferences 1──1 CanonModeSelectionPreference

CanonInstallStatus 1──0..1 CanonSurfaceVerification (present after surface check)
CanonSurfaceVerification 1──0..1 CanonCapabilitySnapshot

ActiveSessionRecord 1──0..1 GovernedSessionLifecycle (present during governed runs)
GovernedSessionLifecycle 1──* GovernedStageRecord (existing type)
GovernedSessionLifecycle 1──* GovernedDocumentRef

AuthoredBriefBundle 1──1 GovernanceIntent (extended with explicit_mode, explicit_no_canon)
```

## Serialization Notes

- `CanonModeSelectionPreference` serializes as lowercase kebab-case:
  `"manual"`, `"auto-confirm"`, `"auto"`
- `CanonMode` new variants serialize as kebab-case matching the Canon mode
  identifier: `"system-shaping"`, `"refactor"`, `"review"`, `"incident"`,
  `"system-assessment"`, `"migration"`, `"supply-chain-analysis"`
- `CanonPreferences` serializes under the `[canon]` TOML section in
  `config.toml`
- `GovernedSessionLifecycle` serializes as a JSON object within the session
  record in `session.json`
- `CanonSurfaceVerification` is ephemeral (diagnostics output only) unless
  cached in the session for runtime gating
