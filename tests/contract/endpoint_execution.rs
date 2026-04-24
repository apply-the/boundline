#![allow(dead_code)]

use serde_json::json;
use synod::domain::step::{
    ErrorInfo, Recoverability, StepExecutionResult, StepExecutionResultError,
};

fn retryable_failure_result() -> StepExecutionResult {
    StepExecutionResult::failure(
        ErrorInfo::new("tool_failed", "tool execution failed")
            .with_details(json!({"exit_code": 1})),
        Recoverability::Retryable,
    )
    .with_evidence(json!({"stderr": "transient crash"}))
}

fn successful_endpoint_result() -> StepExecutionResult {
    StepExecutionResult::success(json!({"tests_passed": true}))
        .with_evidence(json!({"stdout": "ok"}))
}

#[test]
fn retryable_failures_surface_error_details_and_recoverability() {
    let result = retryable_failure_result();

    result.validate().unwrap();
    assert_eq!(result.recoverability, Recoverability::Retryable);
    assert_eq!(result.error.unwrap().code, "tool_failed");
}

#[test]
fn invalid_results_without_output_or_error_fail_validation() {
    let result = StepExecutionResult { output: None, error: None, ..successful_endpoint_result() };

    let error = result.validate().unwrap_err();
    assert_eq!(error, StepExecutionResultError::MissingOutput);
}
