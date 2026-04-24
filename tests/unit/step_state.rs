use serde_json::json;
use synod::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, Step, StepAttempt, StepExecutionResult,
    StepExecutionResultError, StepResultSummary, StepStatus,
};

#[test]
fn step_lifecycle_tracks_attempts_and_failure_state() {
    let mut step = Step::agent("analyze", "analyzer", json!({"ticket": "BUG-4"})).unwrap();
    assert_eq!(step.status, StepStatus::Pending);
    assert_eq!(step.attempt_count, 0);

    step.mark_running();
    assert_eq!(step.status, StepStatus::Running);
    assert_eq!(step.attempt_count, 1);

    step.mark_failed(ErrorInfo::new("transient", "temporary failure"), Recoverability::Retryable);
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.recoverability, Some(Recoverability::Retryable));
}

#[test]
fn step_execution_result_validation_rejects_conflicting_payloads() {
    let result = StepExecutionResult {
        output: Some(json!({"ok": true})),
        error: Some(ErrorInfo::new("conflict", "both output and error present")),
        ..StepExecutionResult::success(json!({"ok": true}))
    };

    let error = result.validate().unwrap_err();
    assert_eq!(error, StepExecutionResultError::ConflictingOutputAndError);
}

#[test]
fn step_helpers_cover_decision_results_and_attempt_summaries() {
    let mut step = Step::decision("decide", json!({"ticket": "BUG-11"})).unwrap();
    step.mark_running();

    let mut patch = serde_json::Map::new();
    patch.insert("verified".to_string(), json!(true));
    let success = StepExecutionResult::success_with_patch(json!({"ok": true}), patch)
        .with_evidence(json!({"source": "unit"}));
    assert_eq!(success.status, ExecutionStatus::Succeeded);
    assert_eq!(success.state_patch.as_ref().unwrap()["verified"], json!(true));
    assert_eq!(success.evidence.as_ref().unwrap()["source"], json!("unit"));

    step.mark_succeeded(success.output.clone().unwrap());
    let summary = StepResultSummary::from_step(&step);
    assert_eq!(summary.step_id, "decide");
    assert_eq!(summary.status, StepStatus::Succeeded);

    let mut attempt = StepAttempt::new(step.id.clone(), json!({"input": true}), 10);
    let failure = StepExecutionResult::failure(
        ErrorInfo::new("retryable", "temporary issue"),
        Recoverability::Retryable,
    );
    attempt.complete(&failure, 12);
    assert_eq!(attempt.ended_at, Some(12));
    assert_eq!(attempt.failure_kind, Some(Recoverability::Retryable));
    assert_eq!(attempt.result_snapshot.unwrap()["code"], json!("retryable"));
}
