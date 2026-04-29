use synod::domain::decision::{
    Decision, DecisionError, DecisionStatus, DecisionType, EvidenceKind, EvidenceRef,
};
use synod::domain::tool_result::ToolResult;

#[test]
fn new_decision_has_pending_status_and_generated_id() {
    let decision = Decision::new(
        DecisionType::Analyze,
        "src/main.rs",
        "need to understand current code",
        "file contents collected",
        vec![EvidenceRef::file("src/main.rs")],
    );
    assert!(!decision.id.is_empty());
    assert_eq!(decision.status, DecisionStatus::Pending);
    assert_eq!(decision.decision_type, DecisionType::Analyze);
    assert_eq!(decision.target, "src/main.rs");
    assert!(decision.tool_result.is_none());
    assert!(decision.completed_at.is_none());
}

#[test]
fn validation_rejects_empty_target_rationale_and_expected_outcome() {
    let mut d = Decision::new(DecisionType::Code, "t", "r", "e", vec![]);
    d.target = String::new();
    assert!(matches!(d.validate(), Err(DecisionError::MissingTarget)));

    d.target = "t".to_string();
    d.rationale = String::new();
    assert!(matches!(d.validate(), Err(DecisionError::MissingRationale)));

    d.rationale = "r".to_string();
    d.expected_outcome = String::new();
    assert!(matches!(d.validate(), Err(DecisionError::MissingExpectedOutcome)));
}

#[test]
fn validation_rejects_empty_id() {
    let mut d = Decision::new(DecisionType::Code, "t", "r", "e", vec![]);
    d.id = String::new();
    assert!(matches!(d.validate(), Err(DecisionError::MissingId)));
}

#[test]
fn mark_dispatched_transitions_from_pending() {
    let mut d = Decision::new(DecisionType::Test, "tests/", "verify", "pass", vec![]);
    assert!(d.mark_dispatched().is_ok());
    assert_eq!(d.status, DecisionStatus::Dispatched);
}

#[test]
fn mark_dispatched_rejects_non_pending() {
    let mut d = Decision::new(DecisionType::Test, "tests/", "verify", "pass", vec![]);
    d.mark_dispatched().unwrap();
    let err = d.mark_dispatched().unwrap_err();
    assert!(matches!(
        err,
        DecisionError::InvalidTransition { from: DecisionStatus::Dispatched, .. }
    ));
}

#[test]
fn mark_verified_transitions_from_dispatched_with_tool_result() {
    let mut d = Decision::new(DecisionType::Test, "t", "r", "e", vec![]);
    d.mark_dispatched().unwrap();
    let result = ToolResult::new("cargo", "cargo test", true, 100);
    assert!(d.mark_verified(result).is_ok());
    assert_eq!(d.status, DecisionStatus::Verified);
    assert!(d.tool_result.is_some());
    assert!(d.completed_at.is_some());
}

#[test]
fn mark_verified_rejects_non_dispatched() {
    let mut d = Decision::new(DecisionType::Test, "t", "r", "e", vec![]);
    let result = ToolResult::new("cargo", "cargo test", true, 100);
    let err = d.mark_verified(result).unwrap_err();
    assert!(matches!(err, DecisionError::InvalidTransition { from: DecisionStatus::Pending, .. }));
}

#[test]
fn mark_failed_transitions_from_dispatched_with_tool_result() {
    let mut d = Decision::new(DecisionType::Code, "t", "r", "e", vec![]);
    d.mark_dispatched().unwrap();
    let result = ToolResult::new("cargo", "cargo check", false, 50);
    assert!(d.mark_failed(result).is_ok());
    assert_eq!(d.status, DecisionStatus::Failed);
    assert!(d.tool_result.is_some());
    assert!(d.completed_at.is_some());
}

#[test]
fn mark_recovered_transitions_from_failed() {
    let mut d = Decision::new(DecisionType::Code, "t", "r", "e", vec![]);
    d.mark_dispatched().unwrap();
    d.mark_failed(ToolResult::new("cargo", "cargo check", false, 50)).unwrap();
    assert!(d.mark_recovered().is_ok());
    assert_eq!(d.status, DecisionStatus::Recovered);
}

#[test]
fn mark_recovered_rejects_non_failed() {
    let mut d = Decision::new(DecisionType::Code, "t", "r", "e", vec![]);
    let err = d.mark_recovered().unwrap_err();
    assert!(matches!(err, DecisionError::InvalidTransition { from: DecisionStatus::Pending, .. }));
}

#[test]
fn decision_status_terminal_check() {
    assert!(!DecisionStatus::Pending.is_terminal());
    assert!(!DecisionStatus::Dispatched.is_terminal());
    assert!(DecisionStatus::Verified.is_terminal());
    assert!(DecisionStatus::Failed.is_terminal());
    assert!(DecisionStatus::Recovered.is_terminal());
}

#[test]
fn evidence_ref_constructors_set_kind() {
    let trace = EvidenceRef::trace("trace-ref");
    assert_eq!(trace.kind, EvidenceKind::Trace);
    assert_eq!(trace.reference, "trace-ref");

    let file = EvidenceRef::file("src/lib.rs");
    assert_eq!(file.kind, EvidenceKind::File);

    let canon = EvidenceRef::canon(".canon/artifact");
    assert_eq!(canon.kind, EvidenceKind::Canon);

    let tool = EvidenceRef::tool_output("decision-id");
    assert_eq!(tool.kind, EvidenceKind::ToolOutput);
}

#[test]
fn as_tool_output_evidence_uses_decision_id() {
    let d = Decision::new(DecisionType::Analyze, "t", "r", "e", vec![]);
    let evidence = d.as_tool_output_evidence();
    assert_eq!(evidence.kind, EvidenceKind::ToolOutput);
    assert_eq!(evidence.reference, d.id);
}

#[test]
fn decision_round_trips_through_json() {
    let d = Decision::new(
        DecisionType::Replan,
        "src/lib.rs",
        "needs replanning",
        "new plan generated",
        vec![EvidenceRef::trace("prev-trace")],
    );
    let json = serde_json::to_string(&d).unwrap();
    let parsed: Decision = serde_json::from_str(&json).unwrap();
    assert_eq!(d.id, parsed.id);
    assert_eq!(d.decision_type, parsed.decision_type);
    assert_eq!(d.target, parsed.target);
}
