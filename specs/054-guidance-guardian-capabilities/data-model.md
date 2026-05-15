# Data Model: Guidance And Guardian Capabilities

## GuidanceCapability

Represents one resolved guidance entry available to planning or execution.

Fields:
- `capability_id`: stable guidance identifier.
- `title`: operator-facing capability title.
- `applies_to`: ordered lifecycle phases where the guidance may influence work.
- `roles`: ordered expert or reviewer roles that consume the guidance.
- `content_ref`: repository or workspace path to the Markdown guidance content.
- `priority`: relative importance within the resolved set.
- `authority_source`: `workspace_override`, `canon_governed`, `shared_pack`, or `built_in`.
- `source_ref`: operator-visible reference for the winning source.
- `pack_id`: optional shared-pack identifier when the guidance came from a pack.

Validation rules:
- `capability_id`, `title`, `content_ref`, and `source_ref` must be non-empty.
- `applies_to` must not be empty.
- `authority_source` must always be present.

## GuardianCapability

Represents one resolved verification capability that may run after bounded work.

Fields:
- `guardian_id`: stable guardian identifier.
- `title`: operator-facing guardian title.
- `kind`: `deterministic`, `llm`, or `hybrid`.
- `applies_to`: ordered lifecycle phases where the guardian may run.
- `rules`: ordered rule identifiers covered by the guardian.
- `severity_floor`: lowest disposition this guardian may emit.
- `command_ref`: deterministic command or script reference when applicable.
- `instruction_ref`: semantic evaluation prompt or instruction reference when applicable.
- `authority_source`: `workspace_override`, `canon_governed`, `shared_pack`, or `built_in`.
- `source_ref`: operator-visible reference for the winning source.
- `pack_id`: optional shared-pack identifier when the guardian came from a pack.

Validation rules:
- `guardian_id`, `title`, `source_ref`, and `rules` must be non-empty.
- `applies_to` must not be empty.
- `deterministic` guardians require `command_ref`.
- `llm` guardians require `instruction_ref`.
- `hybrid` guardians require both `command_ref` and `instruction_ref`.

## CapabilityResolutionRecord

Represents the ordered outcome of discovering and resolving guidance or guardian entries from all supported sources.

Fields:
- `target_ref`: bounded workspace target or workspace-level scope.
- `phase`: lifecycle phase for which the resolution was computed.
- `loaded_guidance`: ordered resolved guidance identifiers.
- `loaded_guardians`: ordered resolved guardian identifiers.
- `loaded_sources`: operator-visible source refs that contributed winning entries.
- `skipped_sources`: ordered source refs that were ignored, shadowed, unavailable, or invalid.
- `authority_decisions`: ordered lines explaining precedence decisions.
- `summary`: operator-facing headline for the resolved capability set.

Validation rules:
- `target_ref`, `phase`, and `summary` must be non-empty.
- Every skipped source must have a matching reason in `authority_decisions`.

## GuardianExecutionRecord

Represents one ordered guardian execution attempt inside a bounded lifecycle phase.

Fields:
- `guardian_id`: executed guardian identifier.
- `phase`: lifecycle phase where execution occurred.
- `execution_state`: `completed`, `skipped`, `degraded`, or `failed`.
- `route_slot`: existing runtime routing slot used for semantic execution when applicable.
- `evidence_refs`: operator-visible references passed into the guardian.
- `finding_ids`: structured findings emitted by the guardian attempt.
- `degradation_reason`: explicit explanation when execution could not proceed normally.

State transitions:
- `completed` when the guardian runs and emits one or more findings.
- `skipped` when bounded ordering or blocking findings make execution unnecessary.
- `degraded` when no suitable route or required input is available.
- `failed` when the guardian command or semantic invocation errors.

## GuardianFinding

Represents one structured verification result emitted by a guardian.

Fields:
- `finding_id`: stable runtime identifier for the finding.
- `guardian_id`: guardian that emitted the finding.
- `rule_id`: specific rule or principle that triggered.
- `disposition`: `advise`, `warn`, `concern`, `error`, or `block`.
- `summary`: concise operator-facing headline.
- `evidence_refs`: ordered supporting evidence references.
- `confidence`: bounded confidence score or bucket.
- `recommended_action`: operator-facing remediation text.
- `authority_source`: winning guidance authority used to calibrate the finding.
- `source_ref`: operator-visible guidance or guardian source reference.
- `phase`: lifecycle phase that produced the finding.

Validation rules:
- `finding_id`, `guardian_id`, `rule_id`, `summary`, `recommended_action`, and `source_ref` must be non-empty.
- `evidence_refs` may be empty only for explicit guardian-failure findings.
- `confidence` must always be present, even for degraded or failure outcomes.

## GuidanceGuardianProjection

Represents the read-side projection carried into goal-plan, status, next, and inspect surfaces.

Projected fields:
- `capability_resolution_summary`
- `loaded_guidance_sources`
- `skipped_guidance_sources`
- `loaded_guardian_sources`
- `skipped_guardian_sources`
- `guardian_timeline`
- `guardian_findings_summary`
- `guardian_findings`
- `guardian_degradations`
- `guardian_blocking_outcome`

Relationships:
- `CapabilityResolutionRecord` owns the loaded and skipped source lists.
- `GuardianExecutionRecord` references one `GuardianCapability` and zero or more `GuardianFinding` records.
- `GuidanceGuardianProjection` is derived from `CapabilityResolutionRecord`, `GuardianExecutionRecord`, and `GuardianFinding` data and must be persisted instead of recomputed opaquely.
