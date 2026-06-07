//! Recursive stage refinement domain types.
//!
//! This module defines the core domain model for bounded, inspectable
//! stage-refinement loops. A refinement loop iteratively improves a stage
//! artifact (currently only the plan stage) through a
//! `planner → critic → planner → finalizer` pattern, producing compact
//! round packets that are trace-linked and schema-versioned.
//!
//! Key types:
//! - [`Confidence`] — four-level enum for assessing plan quality
//! - [`StopReason`] — closed vocabulary of 9 loop termination reasons
//! - [`RoundPacket`] — compact per-round record persisted in the trace store
//! - [`RefinementProfile`] — TOML-configurable profile enabling refinement
//! - [`ClosureCheck`] — ordered evaluation determining whether to stop or continue
//! - [`PlanStructureDigest`] — structural snapshot for material-delta detection

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

// ── Schema Version ────────────────────────────────────────────────────

/// Schema version for round packets emitted by this feature.
/// Bump when the packet shape changes in a backward-incompatible way.
pub const ROUND_PACKET_SCHEMA_VERSION: &str = "1.0";

/// Default maximum number of refinement rounds when not specified in config.
pub const DEFAULT_MAX_ROUNDS: u32 = 3;

/// Default maximum elapsed time in seconds when not specified in config.
pub const DEFAULT_MAX_ELAPSED_TIME_SECONDS: u64 = 300;

/// File name for the refinement profiles configuration file,
/// relative to the workspace `.boundline/` directory.
pub const REFINEMENT_CONFIG_FILE: &str = "refinement-profiles.toml";

// ── Confidence ─────────────────────────────────────────────────────────

/// Four-level enum for assessing the quality of a plan candidate.
///
/// The critic proposes a value; the runtime validates and may downgrade
/// but never silently upgrade. `High` is forbidden when blocking findings
/// are unresolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Plan is not ready; significant rework needed.
    Insufficient,
    /// Plan has major gaps; requires substantial revision.
    Low,
    /// Plan is adequate; may proceed with noted findings.
    Sufficient,
    /// Plan is thorough and complete; no material issues.
    High,
}

impl Confidence {
    /// Validate and potentially downgrade the critic's proposed confidence
    /// based on objective finding counts. The effective confidence never
    /// exceeds the critic's confidence.
    ///
    /// Returns the effective confidence and an optional adjustment reason
    /// when a downgrade occurred.
    pub fn validate_effective(
        critic: Confidence,
        has_blockers: bool,
        high_severity_count: usize,
        medium_severity_count: usize,
    ) -> (Confidence, Option<ConfidenceAdjustment>) {
        // Blocking findings always cap at Sufficient.
        if has_blockers {
            return Self::cap_at_sufficient(critic, ConfidenceAdjustment::BlockersUnresolved);
        }

        // High-severity findings cap at Sufficient.
        if high_severity_count > 0 {
            return Self::cap_at_sufficient(critic, ConfidenceAdjustment::HighSeverityFindings);
        }

        // Three or more medium-severity findings cap at Sufficient.
        if medium_severity_count >= 3 {
            return Self::cap_at_sufficient(critic, ConfidenceAdjustment::MultipleMediumFindings);
        }

        // No objective reason to downgrade.
        (critic, None)
    }

    /// Cap `critic` at `Sufficient` and return the effective confidence
    /// along with an adjustment reason when a downgrade actually occurred.
    fn cap_at_sufficient(
        critic: Confidence,
        reason: ConfidenceAdjustment,
    ) -> (Confidence, Option<ConfidenceAdjustment>) {
        let effective =
            if critic < Confidence::Sufficient { critic } else { Confidence::Sufficient };
        let adjustment = if effective < critic { Some(reason) } else { None };
        (effective, adjustment)
    }
}

// ── Confidence Adjustment ──────────────────────────────────────────────

/// Reason the runtime adjusted the critic's proposed confidence downward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceAdjustment {
    /// Blocking findings remain unresolved.
    BlockersUnresolved,
    /// One or more high-severity findings are present.
    HighSeverityFindings,
    /// Three or more medium-severity findings are present.
    MultipleMediumFindings,
}

// ── Stop Reason ────────────────────────────────────────────────────────

/// Closed vocabulary of reasons a refinement loop stopped.
///
/// The runtime must only emit values from this set. Unrecognized values
/// are invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Closure check found no structural or semantic change between rounds.
    NoMaterialDelta,
    /// The configured `max_rounds` limit was reached.
    RoundLimitExhausted,
    /// The configured `max_elapsed_time` limit was exceeded.
    TimeLimitExhausted,
    /// The provider returned an empty or missing candidate.
    EmptyCandidate,
    /// Blocking findings remain and the round budget was exhausted.
    UnresolvedBlocker,
    /// A provider failed mid-round.
    ProviderFailure,
    /// The round packet is missing required fields or structurally invalid.
    MalformedPacket,
    /// A requested or applied delta references a non-existent artifact.
    InvalidDelta,
    /// Configuration validation failed (zero limits, missing provider, etc.).
    InvalidConfiguration,
}

// ── Delta Kind ─────────────────────────────────────────────────────────

/// The type of structural change described by a revision delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaKind {
    /// Add a new task to the plan.
    AddTask,
    /// Remove a task from the plan.
    RemoveTask,
    /// Change task ordering.
    ReorderTask,
    /// Add or remove a dependency edge.
    UpdateDependency,
    /// Change scope boundary.
    UpdateScope,
    /// Change validation strategy.
    UpdateValidation,
    /// Change risk assessment or mitigation.
    UpdateRisk,
    /// Resolve a blocker.
    UpdateBlocker,
}

// ── Finding Identifier ─────────────────────────────────────────────────

/// A newtype wrapper for finding identifiers used in provenance tracking.
///
/// If the review domain already defines an equivalent type, this should
/// be replaced by a re-export.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FindingId(pub String);

// ── Revision Delta ─────────────────────────────────────────────────────

/// A structured description of a change to a stage artifact, requested
/// by the critic and applied by the planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionDelta {
    /// Trace artifact reference, e.g. `"trace://plan-candidate-2"`.
    pub artifact_ref: String,
    /// Type of change.
    pub kind: DeltaKind,
    /// Specific element being changed (task ID, section, dependency edge).
    pub target: String,
    /// Human-readable description of the change.
    pub description: String,
    /// Finding that motivated this delta.
    pub provenance: FindingId,
}

// ── Refinement Roles ───────────────────────────────────────────────────

/// Provider ID mapping for the three refinement roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementRoles {
    /// Provider ID for the planner role.
    pub planner_provider_id: String,
    /// Provider ID for the critic role.
    pub critic_provider_id: String,
    /// Provider ID for the finalizer role.
    pub finalizer_provider_id: String,
}

// ── Refinement Profile ─────────────────────────────────────────────────

/// A named, versioned configuration enabling a specific refinement
/// pattern for a specific stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementProfile {
    /// Profile name, e.g. `"plan_refinement"`.
    #[serde(default)]
    pub profile: String,
    /// Stage this profile applies to, e.g. `"plan"`.
    #[serde(default)]
    pub stage: String,
    /// Whether refinement is active for this stage.
    pub enabled: bool,
    /// Hard round limit; must be ≥ 1 after resolving config and CLI overrides.
    pub max_rounds: u32,
    /// Hard time limit in seconds; must be > 0.
    pub max_elapsed_time_seconds: u64,
    /// Provider ID mapping for planner, critic, and finalizer.
    pub roles: RefinementRoles,
}

// ── Refinement Outcome ─────────────────────────────────────────────────

/// The final result of a refinement loop.
///
/// The term `success` is not used as an outcome label. `Finalized` means
/// the artifact is ready for the next stage; `Incomplete` means it is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefinementOutcome {
    /// Artifact is ready for the next stage.
    Finalized,
    /// Artifact is not ready; stop reason and outstanding findings provided.
    Incomplete,
}

// ── Round Packet ───────────────────────────────────────────────────────

/// A compact structured record of one refinement round.
///
/// Persisted in the trace store and linked to the session. Artifacts are
/// referenced by trace identifier rather than copied inline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundPacket {
    /// Schema version, e.g. `"1.0"`.
    pub schema_version: String,
    /// Profile name, e.g. `"plan_refinement"`.
    pub profile: String,
    /// Stage name, e.g. `"plan"`.
    pub stage: String,
    /// 1-based round number within the loop.
    pub round: u32,
    /// Trace artifact reference, e.g. `"trace://plan-candidate-2"`.
    pub candidate_ref: String,
    /// Finding IDs from this round.
    pub findings: Vec<FindingId>,
    /// Revision deltas the critic requested for this round.
    pub requested_deltas: Vec<RevisionDelta>,
    /// Revision deltas the planner applied for this round.
    pub applied_deltas: Vec<RevisionDelta>,
    /// Confidence level proposed by the critic.
    pub critic_confidence: Confidence,
    /// Confidence level validated by the runtime.
    pub effective_confidence: Confidence,
    /// Reason for adjustment when critic and effective differ; `None` when they match.
    pub confidence_adjustment_reason: Option<ConfidenceAdjustment>,
    /// Reason the loop stopped; `None` when the loop continues.
    pub stop_reason: Option<StopReason>,
}

// ── Plan Structure Digest ──────────────────────────────────────────────

/// A structural snapshot of a plan candidate used for material-delta
/// detection. Two digests are equal when no structural or semantic
/// change exists between the corresponding plan candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStructureDigest {
    /// Number of tasks in the plan.
    pub task_count: usize,
    /// Task IDs in execution order.
    pub task_ids_ordered: Vec<String>,
    /// Dependency edges as (from_id, to_id) pairs.
    pub dependency_pairs: BTreeSet<(String, String)>,
    /// Hash of the scope boundary description.
    pub scope_boundary_hash: u64,
    /// Hash of the validation strategy description.
    pub validation_strategy_hash: u64,
    /// Number of identified risks.
    pub risk_count: usize,
    /// Number of identified blockers.
    pub blocker_count: usize,
    /// Bitmask of readiness indicators.
    pub readiness_flags: u8,
    /// IDs of findings that remain unresolved.
    pub unresolved_finding_ids: BTreeSet<String>,
}

impl PlanStructureDigest {
    /// Returns `true` when any structural dimension differs from `previous`.
    pub fn is_material_delta_from(&self, previous: &PlanStructureDigest) -> bool {
        self.task_count != previous.task_count
            || self.task_ids_ordered != previous.task_ids_ordered
            || self.dependency_pairs != previous.dependency_pairs
            || self.scope_boundary_hash != previous.scope_boundary_hash
            || self.validation_strategy_hash != previous.validation_strategy_hash
            || self.risk_count != previous.risk_count
            || self.blocker_count != previous.blocker_count
            || self.readiness_flags != previous.readiness_flags
            || self.unresolved_finding_ids != previous.unresolved_finding_ids
    }
}

// ── Refinement Loop State ──────────────────────────────────────────────

/// Explicit, testable state tracking for the refinement loop lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefinementLoopState {
    /// Loop has been configured but not yet started.
    Pending,
    /// Loop is actively executing rounds.
    Running,
    /// Loop has stopped with the given reason.
    Stopped(StopReason),
}

// ── Refinement Config Error ────────────────────────────────────────────

/// Errors that can occur during refinement configuration loading and validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefinementConfigError {
    /// The TOML config file could not be read or parsed.
    #[error("failed to read refinement config: {0}")]
    Io(String),
    /// The TOML config file has invalid syntax.
    #[error("invalid TOML in refinement config: {0}")]
    Parse(String),
    /// max_rounds must be ≥ 1; zero values are invalid.
    #[error("max_rounds must be >= 1, got {0}")]
    ZeroMaxRounds(u32),
    /// max_elapsed_time_seconds must be > 0.
    #[error("max_elapsed_time_seconds must be > 0, got {0}")]
    ZeroMaxElapsedTime(u64),
    /// A required provider ID is not registered.
    #[error("provider '{0}' not found in registry")]
    ProviderNotFound(String),
    /// A provider is registered but not active.
    #[error("provider '{0}' is registered but not active")]
    ProviderInactive(String),
    /// A provider failed permission admission.
    #[error("provider '{0}' failed permission admission")]
    ProviderUnauthorized(String),
}

// ── Refinement Error ───────────────────────────────────────────────────

/// Errors that can occur during refinement loop execution.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefinementError {
    /// A provider call failed.
    #[error("provider failure: {0}")]
    ProviderFailure(String),
    /// The provider returned an empty or missing candidate.
    #[error("empty candidate returned by provider")]
    EmptyCandidate,
    /// The round packet is malformed.
    #[error("round packet validation failed: {0}")]
    MalformedPacket(String),
    /// A delta references a non-existent artifact.
    #[error("invalid delta: {0}")]
    InvalidDelta(String),
    /// The loop timed out.
    #[error("refinement loop timed out after {0:?}")]
    Timeout(Duration),
    /// A generic execution failure.
    #[error("refinement execution error: {0}")]
    Execution(String),
}

// ── Round Packet Validation Error ──────────────────────────────────────

/// Errors that can occur during round packet validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RoundPacketValidationError {
    /// A required field is missing.
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    /// The schema version is unrecognized or invalid.
    #[error("invalid schema version: expected {expected}, got {got}")]
    InvalidSchemaVersion { expected: String, got: String },
    /// The candidate_ref does not use the `trace://` prefix.
    #[error("invalid candidate ref: must use trace:// prefix, got {0}")]
    InvalidCandidateRef(String),
    /// The round number is zero (must be ≥ 1).
    #[error("invalid round number: must be >= 1, got {0}")]
    InvalidRoundNumber(u32),
    /// The effective confidence exceeds the critic confidence (downgrade-only rule).
    #[error("confidence invariant violated: effective ({effective:?}) exceeds critic ({critic:?})")]
    ConfidenceUpgrade { critic: Confidence, effective: Confidence },
    /// High confidence cannot coexist with unresolved blocking findings.
    #[error("confidence invariant violated: high confidence with unresolved blockers")]
    HighConfidenceWithBlockers,
}

// ── Refinement Profile Validation ──────────────────────────────────────

impl RefinementProfile {
    /// Validate profile constraints.
    ///
    /// Checks:
    /// - `max_rounds` ≥ 1
    /// - `max_elapsed_time_seconds` > 0
    ///
    /// Provider resolution is handled separately by the registry.
    pub fn validate_limits(&self) -> Result<(), RefinementConfigError> {
        if self.max_rounds == 0 {
            return Err(RefinementConfigError::ZeroMaxRounds(self.max_rounds));
        }
        if self.max_elapsed_time_seconds == 0 {
            return Err(RefinementConfigError::ZeroMaxElapsedTime(self.max_elapsed_time_seconds));
        }
        Ok(())
    }
}

// ── Round Packet Validation ────────────────────────────────────────────

impl RoundPacket {
    /// Validate the round packet's structural invariants.
    ///
    /// Checks:
    /// - Required fields are present (enforced at type level via `Serialize`/`Deserialize`)
    /// - `schema_version` matches `ROUND_PACKET_SCHEMA_VERSION`
    /// - `candidate_ref` uses `trace://` prefix
    /// - `round` ≥ 1
    /// - `effective_confidence` never exceeds `critic_confidence` (downgrade-only)
    /// - `effective_confidence` is not `High` when findings contain blockers
    /// - When `round` > 1, `candidate_ref` must reference an updated candidate
    ///   from the prior round (cross-round continuity).
    pub fn validate(
        &self,
        previous_candidate_ref: Option<&str>,
    ) -> Result<(), RoundPacketValidationError> {
        if self.schema_version != ROUND_PACKET_SCHEMA_VERSION {
            return Err(RoundPacketValidationError::InvalidSchemaVersion {
                expected: ROUND_PACKET_SCHEMA_VERSION.to_string(),
                got: self.schema_version.clone(),
            });
        }

        if !self.candidate_ref.starts_with("trace://") {
            return Err(RoundPacketValidationError::InvalidCandidateRef(
                self.candidate_ref.clone(),
            ));
        }

        if self.round == 0 {
            return Err(RoundPacketValidationError::InvalidRoundNumber(self.round));
        }

        if self.effective_confidence > self.critic_confidence {
            return Err(RoundPacketValidationError::ConfidenceUpgrade {
                critic: self.critic_confidence,
                effective: self.effective_confidence,
            });
        }

        if self.effective_confidence == Confidence::High && !self.findings.is_empty() {
            return Err(RoundPacketValidationError::HighConfidenceWithBlockers);
        }

        // When round > 1, the candidate_ref must differ from the previous round's.
        if self.round > 1
            && let Some(prev_ref) = previous_candidate_ref
            && self.candidate_ref == prev_ref
        {
            return Err(RoundPacketValidationError::InvalidCandidateRef(format!(
                "round {} candidate_ref matches previous round ({})",
                self.round, prev_ref
            )));
        }

        Ok(())
    }
}

// ── Load Refinement Profile ────────────────────────────────────────────

/// Load a refinement profile from `.boundline/refinement-profiles.toml`.
///
/// Returns `Ok(None)` when the config file does not exist (meaning
/// profiles are not configured — the caller applies built-in defaults).
/// Returns an error when the file exists but cannot be parsed or the
/// requested profile is missing.
pub fn load_refinement_profile(
    workspace_root: &Path,
    profile_name: &str,
) -> Result<Option<RefinementProfile>, RefinementConfigError> {
    let config_path = workspace_root.join(".boundline").join(REFINEMENT_CONFIG_FILE);

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(RefinementConfigError::Io(e.to_string())),
    };

    // The config file uses a top-level [profiles.<name>] structure.
    let parsed: toml::Value =
        toml::from_str(&content).map_err(|e| RefinementConfigError::Parse(e.to_string()))?;

    let profile_table =
        parsed.get("profiles").and_then(|p| p.get(profile_name)).ok_or_else(|| {
            RefinementConfigError::Parse(format!(
                "profile '{}' not found in refinement config",
                profile_name
            ))
        })?;

    let mut profile: RefinementProfile = toml::Value::try_into(profile_table.clone())
        .map_err(|e| RefinementConfigError::Parse(e.to_string()))?;
    // The profile name comes from the TOML section key, not the table body.
    profile.profile = profile_name.to_string();

    Ok(Some(profile))
}

// ── Resolve Effective Profile ──────────────────────────────────────────

/// Merge config profile with CLI overrides and built-in defaults to
/// produce the effective [`RefinementProfile`].
pub fn resolve_effective_profile(
    config_profile: Option<RefinementProfile>,
    cli_refine: bool,
    cli_no_refine: bool,
    cli_max_rounds: Option<u32>,
    cli_max_elapsed_time: Option<u64>,
) -> Result<RefinementProfile, RefinementConfigError> {
    let base = config_profile.unwrap_or(RefinementProfile {
        profile: "plan_refinement".to_string(),
        stage: "plan".to_string(),
        enabled: false,
        max_rounds: DEFAULT_MAX_ROUNDS,
        max_elapsed_time_seconds: DEFAULT_MAX_ELAPSED_TIME_SECONDS,
        roles: RefinementRoles {
            planner_provider_id: String::new(),
            critic_provider_id: String::new(),
            finalizer_provider_id: String::new(),
        },
    });

    let enabled = if cli_no_refine {
        false
    } else if cli_refine {
        true
    } else {
        base.enabled
    };

    let max_rounds = cli_max_rounds.unwrap_or(base.max_rounds);
    let max_elapsed_time_seconds = cli_max_elapsed_time.unwrap_or(base.max_elapsed_time_seconds);

    let effective = RefinementProfile { enabled, max_rounds, max_elapsed_time_seconds, ..base };

    effective.validate_limits()?;

    Ok(effective)
}

// ── Closure Check ──────────────────────────────────────────────────────

/// Evaluates whether the refinement loop should stop or continue after
/// a round completes.
///
/// The stop reasons are evaluated in priority order: errors/budget
/// exhaustion before quality gates.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_closure(
    packet: &RoundPacket,
    digest: &PlanStructureDigest,
    previous_digest: Option<&PlanStructureDigest>,
    elapsed: Duration,
    max_elapsed: Duration,
    current_round: u32,
    max_rounds: u32,
    has_unresolved_blockers: bool,
) -> Option<StopReason> {
    // Error conditions (fail-fast).
    if packet.stop_reason == Some(StopReason::InvalidConfiguration) {
        return Some(StopReason::InvalidConfiguration);
    }
    if packet.stop_reason == Some(StopReason::MalformedPacket) {
        return Some(StopReason::MalformedPacket);
    }
    if packet.stop_reason == Some(StopReason::InvalidDelta) {
        return Some(StopReason::InvalidDelta);
    }
    if packet.stop_reason == Some(StopReason::ProviderFailure) {
        return Some(StopReason::ProviderFailure);
    }
    if packet.stop_reason == Some(StopReason::EmptyCandidate) {
        return Some(StopReason::EmptyCandidate);
    }

    // Budget exhaustion.
    if elapsed >= max_elapsed {
        return Some(StopReason::TimeLimitExhausted);
    }

    // Quality gate: no material improvement.
    if let Some(prev) = previous_digest
        && !digest.is_material_delta_from(prev)
    {
        return Some(StopReason::NoMaterialDelta);
    }

    // Budget exhaustion after quality check.
    if current_round >= max_rounds {
        if has_unresolved_blockers {
            return Some(StopReason::UnresolvedBlocker);
        }
        return Some(StopReason::RoundLimitExhausted);
    }

    // Continue.
    None
}

// ── Round Packet Serialization ─────────────────────────────────────────

impl RoundPacket {
    /// Serialize the packet as a JSON value with all required fields.
    pub fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Deserialize and validate a round packet from a JSON value.
    pub fn from_json_value(
        value: &serde_json::Value,
        previous_candidate_ref: Option<&str>,
    ) -> Result<Self, RoundPacketValidationError> {
        let packet: RoundPacket = serde_json::from_value(value.clone()).map_err(|e| {
            RoundPacketValidationError::MissingField(Box::leak(e.to_string().into_boxed_str()))
        })?;
        packet.validate(previous_candidate_ref)?;
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Confidence Tests ───────────────────────────────────────────

    #[test]
    fn confidence_variants_serialize_correctly() {
        let json = serde_json::to_string(&Confidence::High).unwrap();
        assert_eq!(json, "\"high\"");
        let json = serde_json::to_string(&Confidence::Insufficient).unwrap();
        assert_eq!(json, "\"insufficient\"");
    }

    #[test]
    fn confidence_validate_no_downgrade_when_clean() {
        let (eff, reason) = Confidence::validate_effective(Confidence::High, false, 0, 0);
        assert_eq!(eff, Confidence::High);
        assert!(reason.is_none());
    }

    #[test]
    fn confidence_downgrade_for_blockers() {
        let (eff, reason) = Confidence::validate_effective(Confidence::High, true, 0, 0);
        assert_eq!(eff, Confidence::Sufficient);
        assert_eq!(reason, Some(ConfidenceAdjustment::BlockersUnresolved));
    }

    #[test]
    fn confidence_downgrade_for_high_severity() {
        let (eff, reason) = Confidence::validate_effective(Confidence::High, false, 1, 0);
        assert_eq!(eff, Confidence::Sufficient);
        assert_eq!(reason, Some(ConfidenceAdjustment::HighSeverityFindings));
    }

    #[test]
    fn confidence_downgrade_for_medium_severity() {
        let (eff, reason) = Confidence::validate_effective(Confidence::High, false, 0, 3);
        assert_eq!(eff, Confidence::Sufficient);
        assert_eq!(reason, Some(ConfidenceAdjustment::MultipleMediumFindings));
    }

    #[test]
    fn confidence_no_downgrade_when_already_low() {
        let (eff, reason) = Confidence::validate_effective(Confidence::Low, true, 0, 0);
        assert_eq!(eff, Confidence::Low);
        assert!(reason.is_none()); // Already below Sufficient.
    }

    #[test]
    fn confidence_ord_respects_severity() {
        assert!(Confidence::High > Confidence::Sufficient);
        assert!(Confidence::Sufficient > Confidence::Low);
        assert!(Confidence::Low > Confidence::Insufficient);
    }

    // ── StopReason Tests ───────────────────────────────────────────

    #[test]
    fn stop_reason_all_variants_serialize() {
        let variants = [
            StopReason::NoMaterialDelta,
            StopReason::RoundLimitExhausted,
            StopReason::TimeLimitExhausted,
            StopReason::EmptyCandidate,
            StopReason::UnresolvedBlocker,
            StopReason::ProviderFailure,
            StopReason::MalformedPacket,
            StopReason::InvalidDelta,
            StopReason::InvalidConfiguration,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: StopReason = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back);
        }
    }

    // ── DeltaKind Tests ────────────────────────────────────────────

    #[test]
    fn delta_kind_all_variants_serialize() {
        let variants = [
            DeltaKind::AddTask,
            DeltaKind::RemoveTask,
            DeltaKind::ReorderTask,
            DeltaKind::UpdateDependency,
            DeltaKind::UpdateScope,
            DeltaKind::UpdateValidation,
            DeltaKind::UpdateRisk,
            DeltaKind::UpdateBlocker,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: DeltaKind = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back);
        }
    }

    // ── RevisionDelta Tests ────────────────────────────────────────

    #[test]
    fn revision_delta_serialization_roundtrip() {
        let delta = RevisionDelta {
            artifact_ref: "trace://plan-candidate-1".to_string(),
            kind: DeltaKind::AddTask,
            target: "task-3".to_string(),
            description: "Add validation step".to_string(),
            provenance: FindingId("f-001".to_string()),
        };
        let json = serde_json::to_string(&delta).unwrap();
        let back: RevisionDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(delta, back);
    }

    // ── RefinementRoles Tests ──────────────────────────────────────

    #[test]
    fn refinement_roles_serialization_roundtrip() {
        let roles = RefinementRoles {
            planner_provider_id: "openai-gpt-5".to_string(),
            critic_provider_id: "openai-gpt-5".to_string(),
            finalizer_provider_id: "openai-gpt-5".to_string(),
        };
        let json = serde_json::to_string(&roles).unwrap();
        let back: RefinementRoles = serde_json::from_str(&json).unwrap();
        assert_eq!(roles, back);
    }

    // ── RoundPacket Tests ──────────────────────────────────────────

    fn make_valid_packet(round: u32, candidate_ref: &str) -> RoundPacket {
        RoundPacket {
            schema_version: ROUND_PACKET_SCHEMA_VERSION.to_string(),
            profile: "plan_refinement".to_string(),
            stage: "plan".to_string(),
            round,
            candidate_ref: candidate_ref.to_string(),
            findings: vec![],
            requested_deltas: vec![],
            applied_deltas: vec![],
            critic_confidence: Confidence::Sufficient,
            effective_confidence: Confidence::Sufficient,
            confidence_adjustment_reason: None,
            stop_reason: None,
        }
    }

    #[test]
    fn round_packet_valid_passes_validation() {
        let packet = make_valid_packet(1, "trace://plan-candidate-1");
        assert!(packet.validate(None).is_ok());
    }

    #[test]
    fn round_packet_invalid_schema_version_fails() {
        let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
        packet.schema_version = "2.0".to_string();
        assert!(packet.validate(None).is_err());
    }

    #[test]
    fn round_packet_invalid_candidate_ref_fails() {
        let packet = make_valid_packet(1, "plan-candidate-1");
        assert!(packet.validate(None).is_err());
    }

    #[test]
    fn round_packet_zero_round_fails() {
        let packet = make_valid_packet(0, "trace://plan-candidate-0");
        assert!(packet.validate(None).is_err());
    }

    #[test]
    fn round_packet_confidence_upgrade_fails() {
        let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
        packet.critic_confidence = Confidence::Low;
        packet.effective_confidence = Confidence::High;
        assert!(packet.validate(None).is_err());
    }

    #[test]
    fn round_packet_high_confidence_with_findings_fails() {
        let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
        packet.critic_confidence = Confidence::High;
        packet.effective_confidence = Confidence::High;
        packet.findings = vec![FindingId("f-001".to_string())];
        assert!(packet.validate(None).is_err());
    }

    #[test]
    fn round_packet_cross_round_same_candidate_fails() {
        let packet = make_valid_packet(2, "trace://plan-candidate-1");
        assert!(packet.validate(Some("trace://plan-candidate-1")).is_err());
    }

    #[test]
    fn round_packet_cross_round_different_candidate_passes() {
        let packet = make_valid_packet(2, "trace://plan-candidate-2");
        assert!(packet.validate(Some("trace://plan-candidate-1")).is_ok());
    }

    // ── PlanStructureDigest Tests ──────────────────────────────────

    fn make_digest(task_count: usize) -> PlanStructureDigest {
        PlanStructureDigest {
            task_count,
            task_ids_ordered: (0..task_count).map(|i| format!("t-{i}")).collect(),
            dependency_pairs: BTreeSet::new(),
            scope_boundary_hash: 0,
            validation_strategy_hash: 0,
            risk_count: 0,
            blocker_count: 0,
            readiness_flags: 0,
            unresolved_finding_ids: BTreeSet::new(),
        }
    }

    #[test]
    fn digest_identical_plans_not_material() {
        let d1 = make_digest(3);
        let d2 = make_digest(3);
        assert!(!d1.is_material_delta_from(&d2));
    }

    #[test]
    fn digest_task_count_change_is_material() {
        let d1 = make_digest(3);
        let d2 = make_digest(4);
        assert!(d1.is_material_delta_from(&d2));
    }

    #[test]
    fn digest_ordering_change_is_material() {
        let d1 = make_digest(3);
        let mut d2 = make_digest(3);
        d2.task_ids_ordered = vec!["t-0".to_string(), "t-2".to_string(), "t-1".to_string()];
        assert!(d1.is_material_delta_from(&d2));
    }

    #[test]
    fn digest_dependency_change_is_material() {
        let d1 = make_digest(3);
        let mut d2 = make_digest(3);
        d2.dependency_pairs.insert(("t-0".to_string(), "t-1".to_string()));
        assert!(d1.is_material_delta_from(&d2));
    }

    #[test]
    fn digest_wording_only_not_material() {
        // Wording changes don't affect the digest — identical digests.
        let d1 = make_digest(3);
        let d2 = make_digest(3);
        assert!(!d1.is_material_delta_from(&d2));
    }

    // ── ConfidenceAdjustment Tests ─────────────────────────────────

    #[test]
    fn confidence_adjustment_variants_serialize() {
        let variants = [
            ConfidenceAdjustment::BlockersUnresolved,
            ConfidenceAdjustment::HighSeverityFindings,
            ConfidenceAdjustment::MultipleMediumFindings,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: ConfidenceAdjustment = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back);
        }
    }

    // ── RefinementOutcome Tests ────────────────────────────────────

    #[test]
    fn refinement_outcome_variants_serialize() {
        let finalized = RefinementOutcome::Finalized;
        let json = serde_json::to_string(&finalized).unwrap();
        assert_eq!(json, "\"finalized\"");
        let back: RefinementOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(finalized, back);

        let incomplete = RefinementOutcome::Incomplete;
        let json = serde_json::to_string(&incomplete).unwrap();
        assert_eq!(json, "\"incomplete\"");
        let back: RefinementOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(incomplete, back);
    }

    // ── evaluate_closure Tests ─────────────────────────────────────

    fn closure_digest() -> PlanStructureDigest {
        PlanStructureDigest {
            task_count: 1,
            task_ids_ordered: vec!["t-0".into()],
            dependency_pairs: BTreeSet::new(),
            scope_boundary_hash: 0,
            validation_strategy_hash: 0,
            risk_count: 0,
            blocker_count: 0,
            readiness_flags: 0,
            unresolved_finding_ids: BTreeSet::new(),
        }
    }

    fn closure_prev_digest() -> PlanStructureDigest {
        PlanStructureDigest {
            task_count: 2,
            task_ids_ordered: vec!["t-0".into(), "t-1".into()],
            dependency_pairs: BTreeSet::new(),
            scope_boundary_hash: 0,
            validation_strategy_hash: 0,
            risk_count: 0,
            blocker_count: 0,
            readiness_flags: 0,
            unresolved_finding_ids: BTreeSet::new(),
        }
    }

    #[test]
    fn closure_provider_failure_bubbles() {
        let mut p = make_valid_packet(1, "trace://c1");
        p.stop_reason = Some(StopReason::ProviderFailure);
        let d = closure_digest();
        assert_eq!(
            evaluate_closure(&p, &d, None, Duration::ZERO, Duration::from_secs(300), 1, 3, false),
            Some(StopReason::ProviderFailure)
        );
    }

    #[test]
    fn closure_empty_candidate_bubbles() {
        let mut p = make_valid_packet(1, "trace://c1");
        p.stop_reason = Some(StopReason::EmptyCandidate);
        let d = closure_digest();
        assert_eq!(
            evaluate_closure(&p, &d, None, Duration::ZERO, Duration::from_secs(300), 1, 3, false),
            Some(StopReason::EmptyCandidate)
        );
    }

    #[test]
    fn closure_invalid_config_bubbles() {
        let mut p = make_valid_packet(1, "trace://c1");
        p.stop_reason = Some(StopReason::InvalidConfiguration);
        let d = closure_digest();
        assert_eq!(
            evaluate_closure(&p, &d, None, Duration::ZERO, Duration::from_secs(300), 1, 3, false),
            Some(StopReason::InvalidConfiguration)
        );
    }

    #[test]
    fn closure_malformed_packet_bubbles() {
        let mut p = make_valid_packet(1, "trace://c1");
        p.stop_reason = Some(StopReason::MalformedPacket);
        let d = closure_digest();
        assert_eq!(
            evaluate_closure(&p, &d, None, Duration::ZERO, Duration::from_secs(300), 1, 3, false),
            Some(StopReason::MalformedPacket)
        );
    }

    #[test]
    fn closure_invalid_delta_bubbles() {
        let mut p = make_valid_packet(1, "trace://c1");
        p.stop_reason = Some(StopReason::InvalidDelta);
        let d = closure_digest();
        assert_eq!(
            evaluate_closure(&p, &d, None, Duration::ZERO, Duration::from_secs(300), 1, 3, false),
            Some(StopReason::InvalidDelta)
        );
    }

    #[test]
    fn closure_time_exhausted() {
        let p = make_valid_packet(1, "trace://c1");
        let d = closure_digest();
        assert_eq!(
            evaluate_closure(
                &p,
                &d,
                None,
                Duration::from_secs(10),
                Duration::from_secs(5),
                1,
                3,
                false
            ),
            Some(StopReason::TimeLimitExhausted)
        );
    }

    #[test]
    fn closure_no_material_delta() {
        let p = make_valid_packet(1, "trace://c1");
        let d = closure_digest();
        let prev = closure_digest(); // Same digest — no delta
        assert_eq!(
            evaluate_closure(
                &p,
                &d,
                Some(&prev),
                Duration::ZERO,
                Duration::from_secs(300),
                1,
                3,
                false
            ),
            Some(StopReason::NoMaterialDelta)
        );
    }

    #[test]
    fn closure_round_limit_exhausted() {
        let p = make_valid_packet(3, "trace://c3");
        let d = closure_digest();
        let prev = closure_prev_digest();
        assert_eq!(
            evaluate_closure(
                &p,
                &d,
                Some(&prev),
                Duration::ZERO,
                Duration::from_secs(300),
                3,
                3,
                false
            ),
            Some(StopReason::RoundLimitExhausted)
        );
    }

    #[test]
    fn closure_unresolved_blocker() {
        let p = make_valid_packet(3, "trace://c3");
        let d = closure_digest();
        let prev = closure_prev_digest();
        assert_eq!(
            evaluate_closure(
                &p,
                &d,
                Some(&prev),
                Duration::ZERO,
                Duration::from_secs(300),
                3,
                3,
                true
            ),
            Some(StopReason::UnresolvedBlocker)
        );
    }

    #[test]
    fn closure_continues() {
        let p = make_valid_packet(1, "trace://c1");
        let d = closure_digest();
        let prev = closure_prev_digest();
        assert_eq!(
            evaluate_closure(
                &p,
                &d,
                Some(&prev),
                Duration::ZERO,
                Duration::from_secs(300),
                1,
                3,
                false
            ),
            None
        );
    }

    #[test]
    fn from_json_value_invalid_json_fails() {
        let bad_json = serde_json::json!({"not": "a packet"});
        assert!(RoundPacket::from_json_value(&bad_json, None).is_err());
    }

    #[test]
    fn from_json_value_valid_json_succeeds() {
        let p = make_valid_packet(1, "trace://plan-candidate-1");
        let json = p.to_json_value().unwrap();
        let parsed = RoundPacket::from_json_value(&json, None).unwrap();
        assert_eq!(parsed.candidate_ref, "trace://plan-candidate-1");
    }

    #[test]
    fn load_refinement_profile_missing_profile_name_returns_error() {
        let dir = std::env::temp_dir().join(format!("bl-test-load-{}", std::process::id()));
        let boundline = dir.join(".boundline");
        std::fs::create_dir_all(&boundline).unwrap();
        std::fs::write(
            boundline.join("refinement-profiles.toml"),
            "[profiles.other]\nenabled = true\n",
        )
        .unwrap();
        let result = load_refinement_profile(&dir, "plan_refinement");
        assert!(result.is_err());
    }
}
