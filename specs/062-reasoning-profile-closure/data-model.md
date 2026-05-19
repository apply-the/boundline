# Data Model: Reasoning Profile Closure

## Existing Runtime Baseline

`062` reuses the typed runtime entities introduced in `061`, especially
`ReasoningProfileDefinition`, `ProfileActivationRecord`,
`IndependenceAssessment`, `ReasoningOutcome`, and
`ReasoningConfidenceContribution`. This document adds the closure-specific
entities needed to make shipped claims explicit and release-ready.

## ProfileClosureClassification

- Purpose: States the final shipped status of each audited S6 capability.
- Fields:
  - `capability_key`: Stable identifier such as `bounded_self_consistency`,
    `independent_pair_review`, `heterogeneous_security_review`,
    `bounded_reflexion`, `debate`, or `adjudication`.
  - `classification`: One of `shipped_profile`, `bounded_substrate`,
    `shared_primitive`, or `deferred`.
  - `operator_claim`: Human-facing summary that may appear in roadmap,
    validation, and release docs.
  - `runtime_required`: Boolean indicating whether runtime activation evidence is
    mandatory.
  - `trace_required`: Boolean indicating whether additive trace vocabulary must
    remain visible.
  - `confidence_required`: Boolean indicating whether confidence contribution is
    required for the claim.
- Validation rules:
  - `shipped_profile` requires concrete runtime evidence.
  - `bounded_substrate` must not be presented as a standalone shipped profile
    but may be described as bounded support used by concrete profiles.
  - `shared_primitive` must not be presented as a standalone shipped profile.
  - `deferred` must not be claimed as currently shipped.

## ProfileExecutionEvidence

- Purpose: Captures the evidence needed to prove a concrete shipped profile is
  real and bounded.
- Fields:
  - `profile_id`: Concrete shipped profile id.
  - `scenario_kind`: `positive_path`, `degraded`, `blocked`, or `interrupted`.
  - `stage_key`: Governed stage where the scenario runs.
  - `terminal_status`: Final profile state.
  - `status_projection`: Operator-visible summary projected through `status`.
  - `inspect_projection`: Operator-visible summary projected through `inspect`.
  - `trace_events`: Ordered set of expected reasoning trace events.
  - `confidence_summary`: Optional confidence-handoff summary.
- Validation rules:
  - Each shipped concrete residual profile must have at least one
    `positive_path` record.
  - At least one bounded non-success record is required when failure handling is
    materially part of the profile claim.
  - `status_projection`, `inspect_projection`, and `trace_events` must agree on
    the terminal status.

## NonProfileClosureEvidence

- Purpose: Captures what must stay visible when a capability is classified as
  bounded substrate or a shared primitive rather than a standalone profile.
- Fields:
  - `capability_key`: `debate` or `adjudication`.
  - `classification`: `bounded_substrate` or `shared_primitive`.
  - `supporting_profiles`: Concrete profiles that may invoke the primitive.
  - `visible_surfaces`: Runtime, trace, and docs surfaces that may mention the
    primitive.
  - `forbidden_claims`: Statements that would incorrectly imply standalone
    profile shipment.
- Validation rules:
  - `bounded_substrate` surfaces may describe bounded supporting debate
    behavior but must not expose a standalone profile id.
  - `shared_primitive` surfaces may describe shared disagreement-resolution
    behavior but must not expose a standalone profile id.

## CompatibilityValidationSource

- Purpose: Defines which artifact Boundline uses to validate the Canon
  compatibility story in a given environment.
- Fields:
  - `source_kind`: `sibling_repo` or `local_snapshot`.
  - `contract_line`: Supported contract line.
  - `boundline_window`: Supported Boundline compatibility window.
  - `canon_window`: Supported Canon compatibility window.
  - `version_anchor`: Concrete producer version reference such as manifest or
    snapshot `canon_min`.
- Validation rules:
  - At least one source must always be available in Boundline.
  - `local_snapshot` must preserve the published compatibility story when the
    sibling Canon repository is unavailable.

## ReleaseAlignmentRecord

- Purpose: Keeps all release-facing statements aligned with the final closure
  decision.
- Fields:
  - `boundline_version`: Target Boundline release version for this closure.
  - `canon_version`: Target Canon version for the required companion
    publication update.
  - `roadmap_refs`: Files that describe the shipped closure state.
  - `validation_refs`: Validation artifacts that prove the closure state.
  - `changelog_refs`: Release-note files that must be updated.
  - `docs_refs`: Operator-facing docs that must remain aligned.
- Validation rules:
  - Every release-facing file in the record must use the same profile
    classification language.
  - Canon version and published compatibility material must match the same
    released pair that Boundline validates locally.

## MaintainabilityGateRecord

- Purpose: Tracks quality-gate obligations tied to the closure slice.
- Fields:
  - `surface_key`: Named implementation hotspot such as
    `session_validate_governance` or `reasoning_independence_assessment`.
  - `current_gate`: The repository maintainability threshold that must pass.
  - `refactor_strategy`: The planned behavior-preserving decomposition approach.
  - `validation_command`: The local or CI command that proves the gate passes,
    such as the existing SonarCloud quality workflow in `.github/workflows/quality.yml`
    after refreshed `lcov.info` upload plus the local clippy closeout.
- Validation rules:
  - The record may not be satisfied by suppressing the rule.
  - Touched functions must clear the current release-blocking cognitive-complexity
    threshold reported by the repository quality gate.
  - Refactors must preserve existing behavior and test coverage.