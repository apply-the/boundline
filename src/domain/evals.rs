//! Eval runner domain types.
//!
//! This module defines the fixture model, result aggregation, and status
//! types used by the `boundline evals run` command. Every type is
//! serializable for JSON export to CI pipelines.

use serde::{Deserialize, Serialize};

/// The quality dimension an eval tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalDimension {
    PlanningQuality,
    ContextSelectionQuality,
    CriticalContextOmission,
    GuardianFindingQuality,
    CouncilRejectionBehavior,
    ProviderCallFailureHandling,
    CompactionSurvivalDecisions,
    CompactionSurvivalRejections,
}

impl EvalDimension {
    /// All supported eval dimensions.
    pub const fn all() -> [Self; 8] {
        [
            Self::PlanningQuality,
            Self::ContextSelectionQuality,
            Self::CriticalContextOmission,
            Self::GuardianFindingQuality,
            Self::CouncilRejectionBehavior,
            Self::ProviderCallFailureHandling,
            Self::CompactionSurvivalDecisions,
            Self::CompactionSurvivalRejections,
        ]
    }

    /// Human-readable dimension name.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::PlanningQuality => "planning-quality",
            Self::ContextSelectionQuality => "context-selection",
            Self::CriticalContextOmission => "critical-omission",
            Self::GuardianFindingQuality => "guardian-finding",
            Self::CouncilRejectionBehavior => "council-rejection",
            Self::ProviderCallFailureHandling => "provider-failure",
            Self::CompactionSurvivalDecisions => "compaction-decisions",
            Self::CompactionSurvivalRejections => "compaction-rejections",
        }
    }
}

/// The outcome of a single eval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalStatus {
    Pass,
    Fail,
}

/// Path prefix under which eval fixtures are stored, relative to the
/// Boundline workspace-local `.boundline/` directory.
pub const EVAL_FIXTURES_DIR: &str = "evals/fixtures";

/// A test case that validates a specific quality dimension against a known
/// session state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalFixture {
    /// Unique identifier for this eval (e.g., `"planning-quality-01"`).
    pub eval_id: String,
    /// Human-readable name.
    pub eval_name: String,
    /// Quality dimension being tested.
    pub dimension: EvalDimension,
    /// Path to the session fixture or trace, relative to
    /// `.boundline/evals/fixtures/`. No absolute paths in committed
    /// fixtures.
    pub fixture_ref: String,
    /// What the eval expects to find.
    pub expected_outcome: String,
}

/// The result of a single eval run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalResult {
    /// Matches the fixture's `eval_id`.
    pub eval_id: String,
    /// From the fixture.
    pub eval_name: String,
    /// Whether the eval passed or failed.
    pub status: EvalStatus,
    /// Explanation when `status` is `Fail`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Fixture and trace references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
    /// From the fixture.
    pub expected_outcome: String,
    /// What was observed during evaluation.
    pub actual_outcome: String,
    /// Execution time in milliseconds.
    pub duration_ms: u64,
}

/// The aggregate result of an eval suite run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalSummary {
    /// AND of all required eval results (`Pass` only when all required
    /// evals pass).
    pub suite_status: EvalStatus,
    /// Per-eval results in execution order.
    pub results: Vec<EvalResult>,
    /// Total number of evals run.
    pub total_count: u64,
    /// Number of evals that passed.
    pub pass_count: u64,
    /// Number of evals that failed.
    pub fail_count: u64,
    /// Total suite execution time in milliseconds.
    pub duration_ms: u64,
}

impl EvalSummary {
    /// Compute the aggregate from a list of individual results.
    #[must_use]
    pub fn from_results(results: Vec<EvalResult>) -> Self {
        let total_count = results.len() as u64;
        let pass_count = results.iter().filter(|r| r.status == EvalStatus::Pass).count() as u64;
        let fail_count = total_count - pass_count;
        let suite_status = if fail_count == 0 { EvalStatus::Pass } else { EvalStatus::Fail };
        let duration_ms = results.iter().map(|r| r.duration_ms).sum();
        Self { suite_status, results, total_count, pass_count, fail_count, duration_ms }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_dimension_all_returns_eight_dimensions() {
        assert_eq!(EvalDimension::all().len(), 8);
    }

    #[test]
    fn eval_dimension_display_name_non_empty() {
        for d in EvalDimension::all() {
            assert!(!d.display_name().is_empty(), "empty name for {d:?}");
        }
    }

    #[test]
    fn eval_summary_all_pass_yields_suite_pass() {
        let results = vec![EvalResult {
            eval_id: "e1".into(),
            eval_name: "test".into(),
            status: EvalStatus::Pass,
            failure_reason: None,
            source_refs: vec![],
            expected_outcome: "blocked".into(),
            actual_outcome: "blocked".into(),
            duration_ms: 100,
        }];
        let summary = EvalSummary::from_results(results);
        assert_eq!(summary.suite_status, EvalStatus::Pass);
        assert_eq!(summary.pass_count, 1);
        assert_eq!(summary.fail_count, 0);
    }

    #[test]
    fn eval_summary_one_fail_yields_suite_fail() {
        let results = vec![
            EvalResult {
                eval_id: "e1".into(),
                eval_name: "pass".into(),
                status: EvalStatus::Pass,
                failure_reason: None,
                source_refs: vec![],
                expected_outcome: "blocked".into(),
                actual_outcome: "blocked".into(),
                duration_ms: 100,
            },
            EvalResult {
                eval_id: "e2".into(),
                eval_name: "fail".into(),
                status: EvalStatus::Fail,
                failure_reason: Some("expected blocked, got clean".into()),
                source_refs: vec!["fixture-2".into()],
                expected_outcome: "blocked".into(),
                actual_outcome: "clean".into(),
                duration_ms: 50,
            },
        ];
        let summary = EvalSummary::from_results(results);
        assert_eq!(summary.suite_status, EvalStatus::Fail);
        assert_eq!(summary.pass_count, 1);
        assert_eq!(summary.fail_count, 1);
    }

    #[test]
    fn eval_summary_duration_is_sum_of_results() {
        let results = vec![
            EvalResult {
                eval_id: "e1".into(),
                eval_name: "a".into(),
                status: EvalStatus::Pass,
                failure_reason: None,
                source_refs: vec![],
                expected_outcome: "ok".into(),
                actual_outcome: "ok".into(),
                duration_ms: 300,
            },
            EvalResult {
                eval_id: "e2".into(),
                eval_name: "b".into(),
                status: EvalStatus::Pass,
                failure_reason: None,
                source_refs: vec![],
                expected_outcome: "ok".into(),
                actual_outcome: "ok".into(),
                duration_ms: 200,
            },
        ];
        let summary = EvalSummary::from_results(results);
        assert_eq!(summary.duration_ms, 500);
    }

    #[test]
    fn eval_status_serialization_roundtrip() {
        for status in [EvalStatus::Pass, EvalStatus::Fail] {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: EvalStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, parsed);
        }
    }

    #[test]
    fn eval_fixture_serialization_roundtrip() {
        let fixture = EvalFixture {
            eval_id: "test-01".into(),
            eval_name: "Test Eval".into(),
            dimension: EvalDimension::PlanningQuality,
            fixture_ref: "sessions/blocked.json".into(),
            expected_outcome: "execution handoff withheld".into(),
        };
        let json = serde_json::to_string(&fixture).unwrap();
        let parsed: EvalFixture = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.eval_id, fixture.eval_id);
        assert_eq!(parsed.dimension, EvalDimension::PlanningQuality);
        assert!(parsed.fixture_ref.contains("sessions/"));
        assert!(!parsed.fixture_ref.starts_with('/'));
    }
}
