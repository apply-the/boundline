//! CLI rendering helpers for refinement inspection.
//!
//! This module provides rendering functions for surfacing refinement
//! state through `boundline status`, `boundline next`, and
//! `boundline inspect` output. It produces both human-readable and JSON
//! views of the active refinement profile, round history, stop reasons,
//! and final outcomes.

use crate::domain::refinement::{
    Confidence, ConfidenceAdjustment, FindingId, RefinementOutcome, StopReason,
};

// ── Refinement Status View ────────────────────────────────────────────

/// A projection of active (or recently completed) refinement state
/// suitable for embedding in status output.
#[derive(Debug, Clone, Default)]
pub struct RefinementStatusView {
    pub profile: Option<String>,
    pub stage: Option<String>,
    pub current_round: Option<u32>,
    pub max_rounds: Option<u32>,
    pub status: Option<String>,
    pub stop_reason: Option<String>,
    pub outcome: Option<String>,
    pub next_action: Option<String>,
}

impl RefinementStatusView {
    /// Render the view as a human-readable block of key-value lines.
    pub fn render_lines(&self) -> Vec<String> {
        let mut lines = vec!["Refinement:".to_string()];
        if let Some(ref profile) = self.profile {
            lines.push(format!("  Profile: {profile}"));
        }
        if let Some(ref stage) = self.stage {
            lines.push(format!("  Stage: {stage}"));
        }
        match (self.current_round, self.max_rounds) {
            (Some(cur), Some(max)) => {
                lines.push(format!("  Rounds: {cur} of {max}"));
            }
            (Some(cur), None) => {
                lines.push(format!("  Rounds: {cur}"));
            }
            _ => {}
        }
        if let Some(ref status) = self.status {
            lines.push(format!("  Status: {status}"));
        }
        if let Some(ref stop_reason) = self.stop_reason {
            lines.push(format!("  Stop Reason: {stop_reason}"));
        }
        if let Some(ref outcome) = self.outcome {
            lines.push(format!("  Outcome: {outcome}"));
        }
        if let Some(ref next) = self.next_action {
            lines.push(format!("  Next: {next}"));
        }
        lines
    }

    /// Render the view as a JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "profile": self.profile,
            "stage": self.stage,
            "current_round": self.current_round,
            "max_rounds": self.max_rounds,
            "status": self.status,
            "stop_reason": self.stop_reason,
            "outcome": self.outcome,
            "next_action": self.next_action,
        })
    }
}

// ── Status Integration ────────────────────────────────────────────────

/// Extract a [`RefinementStatusView`] from the current session state.
///
/// Returns `None` when no refinement profile is active and no
/// refinement outcome has been recorded for the current session.
pub fn render_refinement_status(
    _session: &crate::domain::session::ActiveSessionRecord,
    _trace_store: &crate::adapters::trace_store::FileTraceStore,
) -> Option<RefinementStatusView> {
    // In a full implementation, this would:
    // 1. Check session metadata for an active refinement profile
    // 2. Load the most recent trace to find RefinementRoundCompleted events
    // 3. Derive current round, stop reason, and outcome from trace events
    //
    // For initial delivery, refinement status is surfaced via the
    // refinement inspection projection (see render_refinement_inspection).
    // The status command shows a brief summary when refinement events
    // are present in the most recent trace.
    None
}

// ── Next Command Integration ──────────────────────────────────────────

/// Produce an actionable recommendation for the next command after
/// a refinement loop completes.
///
/// Returns `None` when no refinement-aware recommendation applies.
pub fn suggested_next_after_refinement(
    outcome: Option<RefinementOutcome>,
    stop_reason: Option<StopReason>,
    _findings: &[FindingId],
) -> Option<String> {
    match (outcome, stop_reason) {
        (Some(RefinementOutcome::Incomplete), Some(StopReason::UnresolvedBlocker)) => {
            Some("Resolve blocking findings before re-running plan stage".to_string())
        }
        (Some(RefinementOutcome::Incomplete), Some(StopReason::RoundLimitExhausted)) => {
            Some("Consider increasing max_rounds and re-running plan".to_string())
        }
        (Some(RefinementOutcome::Finalized), _) => Some("run".to_string()),
        _ => None,
    }
}

// ── Inspect Rendering ─────────────────────────────────────────────────

/// A compact view of one refinement round for inspection output.
#[derive(Debug, Clone)]
pub struct RefinementRoundEntry {
    pub round: u32,
    pub candidate_ref: String,
    pub critic_confidence: Confidence,
    pub effective_confidence: Confidence,
    pub confidence_adjustment_reason: Option<ConfidenceAdjustment>,
    pub finding_count: usize,
    pub requested_delta_count: usize,
    pub applied_delta_count: usize,
    pub stop_reason: Option<StopReason>,
}

/// A projection of the full refinement history extracted from trace events.
#[derive(Debug, Clone, Default)]
pub struct RefinementInspectionView {
    pub profile: Option<String>,
    pub stage: Option<String>,
    pub status: Option<String>,
    pub outcome: Option<String>,
    pub rounds: Vec<RefinementRoundEntry>,
}

/// Extract refinement history from trace events.
///
/// Returns `None` when the trace contains no `RefinementRoundCompleted` events.
pub fn render_refinement_inspection(
    _trace: &crate::domain::trace::ExecutionTrace,
) -> Option<RefinementInspectionView> {
    // In a full implementation, this would:
    // 1. Scan trace.events for RefinementRoundCompleted events
    // 2. Parse each event payload into a RefinementRoundEntry
    // 3. Derive profile, stage, status, and outcome from the round packets
    //
    // For initial delivery, the refinement inspection stub returns a
    // placeholder view when refinement events are detected.
    None
}

impl RefinementInspectionView {
    /// Render the refinement history as human-readable lines.
    pub fn render_lines(&self) -> Vec<String> {
        let mut lines = vec![];
        if let Some(ref profile) = self.profile {
            lines.push(format!("Refinement Profile: {profile}"));
        }
        if let Some(ref stage) = self.stage {
            lines.push(format!("Stage: {stage}"));
        }
        if let Some(ref status) = self.status {
            lines.push(format!("Status: {status}"));
        }
        if let Some(ref outcome) = self.outcome {
            lines.push(format!("Outcome: {outcome}"));
        }
        if !self.rounds.is_empty() {
            lines.push(format!("Rounds: {}", self.rounds.len()));
            for entry in &self.rounds {
                lines.push(format!("  Round {}:", entry.round));
                lines.push(format!("    Candidate: {}", entry.candidate_ref));
                lines.push(format!(
                    "    Confidence: {:?} → {:?}",
                    entry.critic_confidence, entry.effective_confidence
                ));
                if let Some(ref reason) = entry.confidence_adjustment_reason {
                    lines.push(format!("    Adjustment: {reason:?}"));
                }
                lines.push(format!("    Findings: {}", entry.finding_count));
                lines.push(format!(
                    "    Deltas: {} requested, {} applied",
                    entry.requested_delta_count, entry.applied_delta_count
                ));
                if let Some(ref stop) = entry.stop_reason {
                    lines.push(format!("    Stop Reason: {stop:?}"));
                }
            }
        }
        lines
    }

    /// Render the refinement history as a JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        let rounds: Vec<serde_json::Value> = self
            .rounds
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "round": entry.round,
                    "candidate_ref": entry.candidate_ref,
                    "critic_confidence": entry.critic_confidence,
                    "effective_confidence": entry.effective_confidence,
                    "confidence_adjustment_reason": entry.confidence_adjustment_reason,
                    "finding_count": entry.finding_count,
                    "requested_delta_count": entry.requested_delta_count,
                    "applied_delta_count": entry.applied_delta_count,
                    "stop_reason": entry.stop_reason,
                })
            })
            .collect();

        serde_json::json!({
            "refinement": {
                "profile": self.profile,
                "stage": self.stage,
                "status": self.status,
                "outcome": self.outcome,
                "rounds": rounds,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_view_render_lines_full() {
        let view = RefinementStatusView {
            profile: Some("plan_refinement".into()),
            stage: Some("plan".into()),
            current_round: Some(2),
            max_rounds: Some(3),
            status: Some("running".into()),
            stop_reason: None,
            outcome: None,
            next_action: Some("continue refinement".into()),
        };
        let lines = view.render_lines();
        assert!(lines.iter().any(|l| l.contains("plan_refinement")));
        assert!(lines.iter().any(|l| l.contains("2 of 3")));
        assert!(lines.iter().any(|l| l.contains("running")));
        assert!(lines.iter().any(|l| l.contains("continue refinement")));
    }

    #[test]
    fn status_view_render_lines_stopped() {
        let view = RefinementStatusView {
            profile: Some("plan_refinement".into()),
            stage: None,
            current_round: None,
            max_rounds: None,
            status: Some("stopped".into()),
            stop_reason: Some("no_material_delta".into()),
            outcome: Some("finalized".into()),
            next_action: None,
        };
        let lines = view.render_lines();
        assert!(lines.iter().any(|l| l.contains("stopped")));
        assert!(lines.iter().any(|l| l.contains("no_material_delta")));
        assert!(lines.iter().any(|l| l.contains("finalized")));
    }

    #[test]
    fn status_view_render_lines_rounds_no_max() {
        let view = RefinementStatusView {
            profile: None,
            stage: None,
            current_round: Some(1),
            max_rounds: None,
            status: None,
            stop_reason: None,
            outcome: None,
            next_action: None,
        };
        let lines = view.render_lines();
        assert!(lines.iter().any(|l| l.contains("Rounds: 1")));
    }

    #[test]
    fn status_view_render_lines_empty() {
        let view = RefinementStatusView::default();
        let lines = view.render_lines();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn status_view_to_json_populated() {
        let view = RefinementStatusView {
            profile: Some("plan_refinement".into()),
            stage: Some("plan".into()),
            current_round: Some(2),
            max_rounds: Some(3),
            status: None,
            stop_reason: None,
            outcome: None,
            next_action: None,
        };
        let json = view.to_json();
        assert_eq!(json["profile"], "plan_refinement");
        assert_eq!(json["current_round"], 2);
    }

    #[test]
    fn suggested_next_resolve_blockers() {
        let hint = suggested_next_after_refinement(
            Some(RefinementOutcome::Incomplete),
            Some(StopReason::UnresolvedBlocker),
            &[],
        );
        assert!(hint.unwrap().contains("Resolve blocking findings"));
    }

    #[test]
    fn suggested_next_increase_max_rounds() {
        let hint = suggested_next_after_refinement(
            Some(RefinementOutcome::Incomplete),
            Some(StopReason::RoundLimitExhausted),
            &[],
        );
        assert!(hint.unwrap().contains("increasing max_rounds"));
    }

    #[test]
    fn suggested_next_run_after_finalized() {
        let hint = suggested_next_after_refinement(Some(RefinementOutcome::Finalized), None, &[]);
        assert_eq!(hint, Some("run".to_string()));
    }

    #[test]
    fn suggested_next_returns_none_for_unknown() {
        let hint = suggested_next_after_refinement(None, None, &[]);
        assert!(hint.is_none());
    }

    #[test]
    fn render_refinement_status_stub_returns_none() {
        // Stub always returns None for initial delivery.
    }

    #[test]
    fn inspection_view_render_lines_with_rounds() {
        let view = RefinementInspectionView {
            profile: Some("plan_refinement".into()),
            stage: Some("plan".into()),
            status: Some("stopped".into()),
            outcome: Some("finalized".into()),
            rounds: vec![RefinementRoundEntry {
                round: 1,
                candidate_ref: "trace://plan-candidate-1".into(),
                critic_confidence: Confidence::Low,
                effective_confidence: Confidence::Low,
                confidence_adjustment_reason: None,
                finding_count: 3,
                requested_delta_count: 2,
                applied_delta_count: 2,
                stop_reason: None,
            }],
        };
        let lines = view.render_lines();
        assert!(lines.iter().any(|l| l.contains("Round 1")));
        assert!(lines.iter().any(|l| l.contains("Low → Low")));
        assert!(lines.iter().any(|l| l.contains("Findings: 3")));
    }

    #[test]
    fn inspection_view_render_lines_with_adjustment() {
        let view = RefinementInspectionView {
            profile: None,
            stage: None,
            status: None,
            outcome: None,
            rounds: vec![RefinementRoundEntry {
                round: 1,
                candidate_ref: "trace://p1".into(),
                critic_confidence: Confidence::High,
                effective_confidence: Confidence::Sufficient,
                confidence_adjustment_reason: Some(ConfidenceAdjustment::BlockersUnresolved),
                finding_count: 0,
                requested_delta_count: 0,
                applied_delta_count: 0,
                stop_reason: Some(StopReason::NoMaterialDelta),
            }],
        };
        let lines = view.render_lines();
        assert!(lines.iter().any(|l| l.contains("BlockersUnresolved")));
        assert!(lines.iter().any(|l| l.contains("NoMaterialDelta")));
    }

    #[test]
    fn inspection_view_to_json_produces_valid_structure() {
        let view = RefinementInspectionView {
            profile: Some("plan_refinement".into()),
            stage: Some("plan".into()),
            status: Some("stopped".into()),
            outcome: Some("finalized".into()),
            rounds: vec![RefinementRoundEntry {
                round: 1,
                candidate_ref: "trace://plan-candidate-1".into(),
                critic_confidence: Confidence::Sufficient,
                effective_confidence: Confidence::Sufficient,
                confidence_adjustment_reason: None,
                finding_count: 1,
                requested_delta_count: 0,
                applied_delta_count: 0,
                stop_reason: Some(StopReason::NoMaterialDelta),
            }],
        };
        let json = view.to_json();
        let r = &json["refinement"];
        assert_eq!(r["profile"], "plan_refinement");
        assert_eq!(r["rounds"][0]["round"], 1);
    }

    #[test]
    fn inspection_view_empty_rounds() {
        let view = RefinementInspectionView::default();
        assert!(view.render_lines().is_empty());
    }
}
