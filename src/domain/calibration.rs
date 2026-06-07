//! Adaptive governance calibration domain types.
//!
//! This module defines graduated control levels (advisory → catch → rule → hook),
//! the calibration policy that maps guardian rules to enforcement levels, trust
//! metric accumulation and evaluation, override records, degradation rules, and
//! escalation triggers.

use serde::{Deserialize, Serialize};

// ── Control Levels ────────────────────────────────────────────────────

/// The enforcement level of a guardian finding.
///
/// Levels graduate from advisory (visible only) to hook (unconditional block).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevel {
    /// Finding is visible but does not block execution.
    Advisory,
    /// Finding needs attention; operator can bypass with an override.
    Catch,
    /// Finding blocks execution unless a satisfying override is provided.
    Rule,
    /// Finding blocks unconditionally; only a privileged process can bypass.
    Hook,
}

impl ControlLevel {
    /// Returns the next stricter level, or `None` if already at Hook.
    #[must_use]
    pub fn promote(&self) -> Option<ControlLevel> {
        match self {
            ControlLevel::Advisory => Some(ControlLevel::Catch),
            ControlLevel::Catch => Some(ControlLevel::Rule),
            ControlLevel::Rule => Some(ControlLevel::Hook),
            ControlLevel::Hook => None,
        }
    }

    /// Returns the next less-strict level, or `None` if already at Advisory.
    #[must_use]
    pub fn demote(&self) -> Option<ControlLevel> {
        match self {
            ControlLevel::Advisory => None,
            ControlLevel::Catch => Some(ControlLevel::Advisory),
            ControlLevel::Rule => Some(ControlLevel::Catch),
            ControlLevel::Hook => Some(ControlLevel::Rule),
        }
    }

    /// Whether this level blocks execution without an override.
    #[must_use]
    pub fn blocks_execution(&self) -> bool {
        matches!(self, ControlLevel::Rule | ControlLevel::Hook)
    }

    /// Whether this level can be overridden by an operator.
    #[must_use]
    pub fn is_overridable(&self) -> bool {
        matches!(self, ControlLevel::Catch | ControlLevel::Rule)
    }

    /// Human-readable display name.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ControlLevel::Advisory => "advisory",
            ControlLevel::Catch => "catch",
            ControlLevel::Rule => "rule",
            ControlLevel::Hook => "hook",
        }
    }
}

/// Authority zone classification for risk-based calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityZone {
    /// Low-risk, routine work.
    Green,
    /// Medium-risk, needs attention.
    Yellow,
    /// High-risk, strict enforcement.
    Red,
}

/// Risk level within an authority zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

// ── Override Policy ────────────────────────────────────────────────────

/// Policy governing who can override a guardian finding and what evidence is required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OverridePolicy {
    /// Roles authorized to override this guardian's findings.
    #[serde(default)]
    pub allowed_roles: Vec<String>,
    /// Evidence types that must accompany an override.
    #[serde(default)]
    pub required_evidence: Vec<String>,
    /// Whether the override expires.
    #[serde(default)]
    pub time_limited: bool,
    /// Expiry duration in hours, if time-limited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration_hours: Option<u32>,
}

// ── Calibration Policy ─────────────────────────────────────────────────

/// A single entry in the calibration policy mapping a guardian rule to
/// control levels by authority zone and risk level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlLevelEntry {
    /// Matches a guardian rule id in `guardian-rules.toml`.
    pub rule_id: String,
    /// Authority zone this entry applies to.
    pub authority_zone: AuthorityZone,
    /// Risk level this entry applies to.
    pub risk_level: RiskLevel,
    /// Level when no trust data exists.
    pub default_level: ControlLevel,
    /// Level in green zone with sufficient trust.
    pub green_level: ControlLevel,
    /// Level in yellow zone with sufficient trust.
    pub yellow_level: ControlLevel,
    /// Level in red zone.
    pub red_level: ControlLevel,
    /// Minimum calibrated confidence (0.0–1.0) for promotion eligibility.
    pub confidence_threshold: f64,
    /// Override authorization rules for this guardian.
    pub override_policy: OverridePolicy,
}

/// The versioned calibration policy stored in `.boundline/calibration-policy.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationPolicy {
    /// Policy schema version.
    pub schema_version: String,
    /// Minimum adjudicated sessions before trust evaluation.
    #[serde(default = "default_evidence_window")]
    pub evidence_window: u32,
    /// Minimum adjudicated sample size for TPR/FPR computation.
    #[serde(default = "default_minimum_evidence_threshold")]
    pub minimum_evidence_threshold: u32,
    /// Per-rule calibration entries.
    #[serde(default)]
    pub entries: Vec<ControlLevelEntry>,
}

pub const fn default_evidence_window() -> u32 {
    5
}
pub const fn default_minimum_evidence_threshold() -> u32 {
    3
}

/// Error type for calibration policy validation failures.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CalibrationPolicyError {
    #[error(
        "contradictory entries for rule '{rule_id}' at zone={zone:?} risk={risk:?}: levels {level_a:?} vs {level_b:?}"
    )]
    Contradiction {
        rule_id: String,
        zone: AuthorityZone,
        risk: RiskLevel,
        level_a: ControlLevel,
        level_b: ControlLevel,
    },
    #[error("red-zone entry for rule '{rule_id}' cannot default to advisory")]
    RedZoneAdvisory { rule_id: String },
    #[error("confidence threshold for rule '{rule_id}' is out of range (0.0–1.0): {value}")]
    ConfidenceOutOfRange { rule_id: String, value: f64 },
    #[error(
        "calibration policy is empty; at least one entry is required when a policy file exists"
    )]
    EmptyPolicy,
}

impl CalibrationPolicy {
    /// Validate the calibration policy for structural and logical correctness.
    pub fn validate(&self) -> Result<(), CalibrationPolicyError> {
        if self.entries.is_empty() {
            return Err(CalibrationPolicyError::EmptyPolicy);
        }

        for (i, entry) in self.entries.iter().enumerate() {
            check_red_zone_entry(entry)?;
            check_confidence_in_range(entry)?;

            for other in &self.entries[i + 1..] {
                check_contradiction(entry, other)?;
            }
        }

        Ok(())
    }

    /// Find the calibration entry matching a rule_id, authority zone, and risk level.
    pub fn find_entry(
        &self,
        rule_id: &str,
        zone: AuthorityZone,
        risk: RiskLevel,
    ) -> Option<&ControlLevelEntry> {
        self.entries
            .iter()
            .find(|e| e.rule_id == rule_id && e.authority_zone == zone && e.risk_level == risk)
    }

    /// Resolve the control level for a guardian rule given the current context
    /// and optional trust record.
    pub fn resolve_level(
        &self,
        rule_id: &str,
        zone: AuthorityZone,
        risk: RiskLevel,
        trust: Option<&GuardianTrustRecord>,
    ) -> ControlLevelAssignment {
        let entry = self.find_entry(rule_id, zone, risk);
        let default_level = entry.map(|e| e.default_level).unwrap_or(ControlLevel::Advisory);
        let confidence_threshold = entry.map(|e| e.confidence_threshold).unwrap_or(0.85_f64);

        let (assigned_level, reason, calibrated_confidence) = match trust {
            Some(t) if t.adjudicated_count() >= self.minimum_evidence_threshold as u64 => {
                resolve_trust_based_level(
                    t,
                    default_level,
                    confidence_threshold,
                    self.evidence_window,
                )
            }
            _ => {
                let guard_confidence =
                    trust.and_then(|t| t.eval_pass_rate).unwrap_or(confidence_threshold);
                (
                    default_level,
                    format!(
                        "cold start or insufficient trust data; using default level {default_level:?}"
                    ),
                    guard_confidence,
                )
            }
        };

        ControlLevelAssignment {
            rule_id: rule_id.to_string(),
            assigned_level,
            guardian_confidence: trust
                .and_then(|t| t.eval_pass_rate)
                .unwrap_or(confidence_threshold),
            calibrated_confidence,
            authority_zone: zone,
            risk_level: risk,
            reason,
            degraded_from: None,
            degradation_reason: None,
        }
    }
}

/// Check that a red-zone entry does not default or enforce at `advisory`.
fn check_red_zone_entry(entry: &ControlLevelEntry) -> Result<(), CalibrationPolicyError> {
    if entry.authority_zone != AuthorityZone::Red {
        return Ok(());
    }
    if entry.default_level == ControlLevel::Advisory || entry.red_level == ControlLevel::Advisory {
        return Err(CalibrationPolicyError::RedZoneAdvisory { rule_id: entry.rule_id.clone() });
    }
    Ok(())
}

/// Check that a confidence threshold is within 0.0–1.0.
fn check_confidence_in_range(entry: &ControlLevelEntry) -> Result<(), CalibrationPolicyError> {
    if !(0.0..=1.0).contains(&entry.confidence_threshold) {
        return Err(CalibrationPolicyError::ConfidenceOutOfRange {
            rule_id: entry.rule_id.clone(),
            value: entry.confidence_threshold,
        });
    }
    Ok(())
}

/// Check two entries for contradictions on the same rule_id/zone/risk tuple.
fn check_contradiction(
    a: &ControlLevelEntry,
    b: &ControlLevelEntry,
) -> Result<(), CalibrationPolicyError> {
    if a.rule_id != b.rule_id
        || a.authority_zone != b.authority_zone
        || a.risk_level != b.risk_level
    {
        return Ok(());
    }
    if a.default_level != b.default_level
        || a.green_level != b.green_level
        || a.yellow_level != b.yellow_level
        || a.red_level != b.red_level
    {
        return Err(CalibrationPolicyError::Contradiction {
            rule_id: a.rule_id.clone(),
            zone: a.authority_zone,
            risk: a.risk_level,
            level_a: a.default_level,
            level_b: b.default_level,
        });
    }
    Ok(())
}

/// Resolve a guardian's control level based on accumulated trust data.
fn resolve_trust_based_level(
    trust: &GuardianTrustRecord,
    default_level: ControlLevel,
    confidence_threshold: f64,
    evidence_window: u32,
) -> (ControlLevel, String, f64) {
    let tpr = trust.true_positive_rate();
    let calibrated = trust.calibrated_confidence(confidence_threshold);
    let adjudicated = trust.adjudicated_count();

    if trust.incident_correlation {
        let locked = incident_lock_level(default_level);
        return (
            locked,
            format!("incident correlation: guardian locked at {locked:?}"),
            calibrated,
        );
    }

    if trust.eval_pass_rate.is_some_and(|r| r < confidence_threshold) {
        return (
            default_level,
            format!(
                "eval pass rate below confidence threshold {confidence_threshold}; staying at {default_level:?}"
            ),
            calibrated,
        );
    }

    if adjudicated < evidence_window as u64 {
        return (
            default_level,
            format!(
                "insufficient evidence ({adjudicated}/{evidence_window} sessions); staying at {default_level:?}"
            ),
            calibrated,
        );
    }

    match tpr {
        Some(rate) if rate >= 0.90 && trust.false_positive_count == 0 => {
            let promoted = default_level.promote().unwrap_or(default_level);
            (
                promoted,
                format!("TPR {rate:.2} >= 0.90, zero false positives; promoted to {promoted:?}"),
                calibrated,
            )
        }
        Some(rate) if rate < 0.80 => {
            let demoted = default_level.demote().unwrap_or(ControlLevel::Advisory);
            let fpr = 1.0 - rate;
            (demoted, format!("FPR {fpr:.2} > 0.20; demoted to {demoted:?}"), calibrated)
        }
        Some(rate) => (
            default_level,
            format!("TPR {rate:.2} within acceptable range; staying at {default_level:?}"),
            calibrated,
        ),
        None => (
            default_level,
            format!("insufficient sample for TPR; staying at {default_level:?}"),
            calibrated,
        ),
    }
}

/// Clamp a control level to advisory or catch when an incident correlation is active.
fn incident_lock_level(level: ControlLevel) -> ControlLevel {
    match level {
        ControlLevel::Advisory | ControlLevel::Catch => level,
        ControlLevel::Rule | ControlLevel::Hook => ControlLevel::Catch,
    }
}

// ── Guardian Trust Record ──────────────────────────────────────────────

/// Accumulated trust metrics for a guardian across adjudicated sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardianTrustRecord {
    /// Guardian rule id this record tracks.
    pub rule_id: String,
    /// Findings upheld by council as valid, actionable, and correctly classified.
    #[serde(default)]
    pub true_positive_count: u64,
    /// Findings rejected by council as invalid, not applicable, or incorrectly blocking.
    #[serde(default)]
    pub false_positive_count: u64,
    /// Findings pending resolution (not yet adjudicated).
    #[serde(default)]
    pub deferred_count: u64,
    /// Override records accepted by council.
    #[serde(default)]
    pub accepted_override_count: u64,
    /// Same finding reappearing across sessions.
    #[serde(default)]
    pub repeated_violation_count: u64,
    /// Whether the guardian is correlated with a past incident.
    #[serde(default)]
    pub incident_correlation: bool,
    /// Latest evaluation pass rate (0.0–1.0), if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval_pass_rate: Option<f64>,
    /// ISO 8601 timestamp of the last trust evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_evaluated_at: Option<String>,
}

impl GuardianTrustRecord {
    /// Create a new trust record for a guardian with zero counts.
    #[must_use]
    pub fn new(rule_id: &str) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            true_positive_count: 0,
            false_positive_count: 0,
            deferred_count: 0,
            accepted_override_count: 0,
            repeated_violation_count: 0,
            incident_correlation: false,
            eval_pass_rate: None,
            last_evaluated_at: None,
        }
    }

    /// Total number of adjudicated findings (TP + FP, excluding deferred).
    #[must_use]
    pub fn adjudicated_count(&self) -> u64 {
        self.true_positive_count + self.false_positive_count
    }

    /// Compute the true positive rate.
    ///
    /// Returns `None` when the adjudicated sample size is zero (no rate computable).
    /// TPR = TP / (TP + FP).
    #[must_use]
    pub fn true_positive_rate(&self) -> Option<f64> {
        let total = self.adjudicated_count();
        if total == 0 { None } else { Some(self.true_positive_count as f64 / total as f64) }
    }

    /// Compute the false positive rate.
    ///
    /// Returns `None` when the adjudicated sample size is zero.
    #[must_use]
    pub fn false_positive_rate(&self) -> Option<f64> {
        self.true_positive_rate().map(|tpr| 1.0 - tpr)
    }

    /// Compute the effective calibrated confidence after trust adjustment.
    ///
    /// Starts from the eval pass rate (or confidence_threshold if not available),
    /// then adjusts downward based on false positive rate and incident correlation.
    #[must_use]
    pub fn calibrated_confidence(&self, confidence_threshold: f64) -> f64 {
        let base = self.eval_pass_rate.unwrap_or(confidence_threshold);
        let fpr_penalty = self.false_positive_rate().unwrap_or(0.0) * 0.5;
        let incident_penalty = if self.incident_correlation { 0.3 } else { 0.0 };
        (base - fpr_penalty - incident_penalty).clamp(0.0, 1.0)
    }

    /// Record a council adjudication outcome against this guardian.
    pub fn record_adjudication(&mut self, upheld: bool, deferred: bool) {
        if deferred {
            self.deferred_count += 1;
        } else if upheld {
            self.true_positive_count += 1;
        } else {
            self.false_positive_count += 1;
        }
    }
}

// ── Control Level Assignment ───────────────────────────────────────────

/// The current control level assignment for a guardian in a workspace context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlLevelAssignment {
    /// Guardian rule id.
    pub rule_id: String,
    /// Current enforcement level.
    pub assigned_level: ControlLevel,
    /// Raw confidence from the guardian (0.0–1.0).
    pub guardian_confidence: f64,
    /// Confidence after trust-metric adjustment (0.0–1.0).
    pub calibrated_confidence: f64,
    /// Authority zone at assignment time.
    pub authority_zone: AuthorityZone,
    /// Risk level at assignment time.
    pub risk_level: RiskLevel,
    /// Human-readable reason for the level assignment.
    pub reason: String,
    /// Original level before degradation, if degraded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degraded_from: Option<ControlLevel>,
    /// Why degradation occurred, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degradation_reason: Option<String>,
}

// ── Override Record ────────────────────────────────────────────────────

/// A trace-visible record of an operator override of a blocked finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverrideRecord {
    /// Identifier of the blocked finding.
    pub finding_id: String,
    /// Identifier of the control being overridden.
    pub control_id: String,
    /// Guardian that produced the finding.
    pub guardian_id: String,
    /// Level the operator is requesting (cannot be Hook).
    pub requested_level: ControlLevel,
    /// Operator's justification.
    pub reason: String,
    /// Who performed the override (when available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_identity: Option<String>,
    /// ISO 8601 timestamp when the override was written.
    pub timestamp: String,
    /// ISO 8601 expiry if time-limited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    /// Whether the override meets the configured override policy.
    #[serde(default)]
    pub satisfies_policy: bool,
}

// ── Degradation & Escalation ───────────────────────────────────────────

/// Trigger for control degradation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradationTrigger {
    /// AI provider or tool not reachable.
    ProviderUnavailable,
    /// Specific model not available.
    ModelUnavailable,
    /// Required tool not present.
    ToolUnavailable,
}

/// A trace event recording that a control was downgraded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DegradationEvent {
    /// Affected guardian rule.
    pub rule_id: String,
    /// Level before degradation.
    pub original_level: ControlLevel,
    /// Level after degradation.
    pub degraded_level: ControlLevel,
    /// What caused the degradation.
    pub degradation_trigger: DegradationTrigger,
    /// Whether the degraded path is safe.
    pub safe: bool,
    /// Whether human intervention is required.
    pub requires_human_gate: bool,
}

/// Trigger for finding escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationTrigger {
    /// Finding unresolved across multiple sessions.
    RepeatedUnresolved,
    /// Workspace entered the red zone.
    RedZone,
    /// Low confidence but high potential impact.
    LowConfidenceHighImpact,
    /// Mandatory evidence cannot be produced.
    MissingEvidence,
    /// Security, domain, or contract boundary at risk.
    BoundaryRisk,
}

/// A trace event recording that a finding was escalated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationEvent {
    /// Escalated guardian rule.
    pub rule_id: String,
    /// What triggered the escalation.
    pub escalation_trigger: EscalationTrigger,
    /// Level at time of escalation.
    pub current_level: ControlLevel,
    /// Recommended level after escalation.
    pub recommended_level: ControlLevel,
}

// ── Built-in Default ───────────────────────────────────────────────────

/// Error type for policy loading failures.
#[derive(Debug, thiserror::Error)]
pub enum PolicyLoadError {
    #[error("failed to read calibration policy: {0}")]
    ReadError(String),
    #[error("calibration policy is invalid: {0}")]
    InvalidPolicy(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Load the calibration policy from a workspace directory.
///
/// Reads `.boundline/calibration-policy.toml`, deserializes, validates,
/// and returns the policy. Falls back to a built-in all-advisory policy
/// if the file does not exist.
///
/// # Errors
///
/// Returns `PolicyLoadError` if the file exists but cannot be read or is invalid.
pub fn load_calibration_policy(
    workspace_root: &std::path::Path,
) -> Result<CalibrationPolicy, PolicyLoadError> {
    let policy_path = workspace_root.join(".boundline").join("calibration-policy.toml");

    if !policy_path.exists() {
        return Ok(builtin_calibration_policy());
    }

    let content = std::fs::read_to_string(&policy_path)
        .map_err(|e| PolicyLoadError::ReadError(e.to_string()))?;
    let policy: CalibrationPolicy = toml::from_str(&content)
        .map_err(|e| PolicyLoadError::InvalidPolicy(format!("TOML parse error: {e}")))?;
    policy.validate().map_err(|e| PolicyLoadError::InvalidPolicy(e.to_string()))?;
    Ok(policy)
}

/// Return a built-in calibration policy that defaults all guardians to advisory.
///
/// This is the fail-safe default when no `.boundline/calibration-policy.toml`
/// exists. All guardians are advisory (visible but do not block).
#[must_use]
pub fn builtin_calibration_policy() -> CalibrationPolicy {
    CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 5,
        minimum_evidence_threshold: 3,
        entries: Vec::new(),
    }
}

// ── Trust Record Persistence ──────────────────────────────────────────

/// Load guardian trust records from `.boundline/trust-records.json`.
///
/// Returns an empty Vec if the file does not exist or cannot be parsed.
pub fn load_trust_records(workspace_root: &std::path::Path) -> Vec<GuardianTrustRecord> {
    let path = workspace_root.join(".boundline").join("trust-records.json");
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// Save guardian trust records to `.boundline/trust-records.json`.
///
/// Creates the `.boundline/` directory if it does not exist.
pub fn save_trust_records(
    workspace_root: &std::path::Path,
    records: &[GuardianTrustRecord],
) -> Result<(), std::io::Error> {
    let boundline_dir = workspace_root.join(".boundline");
    std::fs::create_dir_all(&boundline_dir)?;
    let path = boundline_dir.join("trust-records.json");
    let json = serde_json::to_string_pretty(records).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Find or create a trust record for the given guardian.
pub fn get_or_create_trust_record<'a>(
    rule_id: &str,
    records: &'a mut Vec<GuardianTrustRecord>,
) -> &'a mut GuardianTrustRecord {
    let pos = records.iter().position(|r| r.rule_id == rule_id);
    match pos {
        Some(idx) => &mut records[idx],
        None => {
            records.push(GuardianTrustRecord::new(rule_id));
            let last = records.len() - 1;
            &mut records[last]
        }
    }
}
