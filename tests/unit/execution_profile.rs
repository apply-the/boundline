use boundline::domain::execution::{
    AdaptiveChangeKind, AdaptiveExecutionProfile, ExecutionAttemptDefinition, ExecutionCommand,
    ExecutionFailureMode, WorkspaceChange, WorkspaceExecutionProfile,
};
use boundline::domain::limits::TerminalCondition;
use boundline::domain::review::{
    ReviewProfile, ReviewScenario, ReviewTrigger, ReviewerDefinition, ReviewerDisposition,
    ReviewerFinding, VoteRuleDefinition,
};
use boundline::{Recoverability, RunLimits};

fn sample_profile() -> WorkspaceExecutionProfile {
    WorkspaceExecutionProfile {
        name: "red-to-green-execution".to_string(),
        read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
        validation_command: ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        },
        attempts: vec![ExecutionAttemptDefinition {
            attempt_id: "fix-add".to_string(),
            summary: "Replace subtraction with addition".to_string(),
            failure_mode: ExecutionFailureMode::Replan,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "left - right".to_string(),
                replace: "left + right".to_string(),
            }],
        }],
        adaptive: None,
        limits: RunLimits::default(),
        governance: None,
        review: None,
        legacy_source: None,
    }
}

#[test]
fn execution_profile_validation_accepts_relative_targets_and_attempts() {
    sample_profile().validate().unwrap();
}

#[test]
fn execution_profile_validation_rejects_paths_outside_the_workspace() {
    let mut profile = sample_profile();
    profile.read_targets = vec!["../outside.rs".to_string()];

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("workspace root"));
}

#[test]
fn execution_profile_validation_rejects_duplicate_attempt_ids() {
    let mut profile = sample_profile();
    profile.attempts.push(profile.attempts[0].clone());

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("duplicated"));
}

#[test]
fn failure_modes_map_to_existing_recoverability_classes() {
    assert_eq!(ExecutionFailureMode::Retry.recoverability(), Recoverability::Retryable);
    assert_eq!(ExecutionFailureMode::Replan.recoverability(), Recoverability::ReplanRequired);
    assert_eq!(ExecutionFailureMode::Terminal.recoverability(), Recoverability::Terminal);
}

#[test]
fn validation_command_renders_as_a_shell_like_string() {
    let command = sample_profile().validation_command;

    assert_eq!(command.rendered(), "cargo test --quiet");
}

#[test]
fn run_limits_accept_partial_json_overrides() {
    let limits: RunLimits = serde_json::from_value(serde_json::json!({
        "max_steps": 6,
        "max_replans": 1
    }))
    .unwrap();

    assert_eq!(limits.max_steps, 6);
    assert_eq!(limits.max_retries, RunLimits::default().max_retries);
    assert_eq!(limits.max_replans, 1);
    assert_eq!(limits.terminal_precedence, RunLimits::default().terminal_precedence);
    assert!(limits.terminal_precedence.contains(&TerminalCondition::GoalSatisfied));
}

#[test]
fn execution_profile_validation_accepts_optional_review_configuration() {
    let mut profile = sample_profile();
    profile.review = Some(ReviewProfile {
        triggers: vec![ReviewTrigger::PrReady],
        reviewers: vec![
            ReviewerDefinition {
                reviewer_id: "safety".to_string(),
                role: "Safety".to_string(),
                source: Some("gpt".to_string()),
                weight: 2,
            },
            ReviewerDefinition {
                reviewer_id: "maintainability".to_string(),
                role: "Maintainability".to_string(),
                source: Some("claude".to_string()),
                weight: 1,
            },
        ],
        vote_rule: VoteRuleDefinition::default(),
        adjudication: Default::default(),
        scenarios: vec![ReviewScenario {
            trigger: ReviewTrigger::PrReady,
            findings: vec![
                ReviewerFinding::new(
                    "safety".to_string(),
                    ReviewerDisposition::Approve,
                    "No blocking issues".to_string(),
                ),
                ReviewerFinding::new(
                    "maintainability".to_string(),
                    ReviewerDisposition::Concern,
                    "Minor cleanup".to_string(),
                ),
            ],
            adjudication_finding: None,
        }],
    });

    profile.validate().unwrap();
}

#[test]
fn execution_profile_validation_accepts_adaptive_only_configuration() {
    let mut profile = sample_profile();
    profile.attempts.clear();
    profile.adaptive = Some(AdaptiveExecutionProfile {
        max_selected_targets: 1,
        max_generated_attempts: 4,
        path_preferences: vec!["src/".to_string()],
        allowed_change_kinds: vec![AdaptiveChangeKind::ArithmeticSwap],
    });

    profile.validate().unwrap();
}

#[test]
fn execution_profile_validation_rejects_invalid_adaptive_configuration() {
    let mut profile = sample_profile();
    profile.attempts.clear();
    profile.read_targets.clear();
    profile.adaptive = Some(AdaptiveExecutionProfile {
        max_selected_targets: 1,
        max_generated_attempts: 4,
        path_preferences: vec![],
        allowed_change_kinds: vec![],
    });

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("read target"));

    let mut profile = sample_profile();
    profile.attempts.clear();
    profile.adaptive = Some(AdaptiveExecutionProfile {
        max_selected_targets: 0,
        max_generated_attempts: 4,
        path_preferences: vec![],
        allowed_change_kinds: vec![],
    });

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("max_selected_targets"));

    let mut profile = sample_profile();
    profile.attempts.clear();
    profile.adaptive = Some(AdaptiveExecutionProfile {
        max_selected_targets: 1,
        max_generated_attempts: 0,
        path_preferences: vec![],
        allowed_change_kinds: vec![],
    });

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("max_generated_attempts"));

    let mut profile = sample_profile();
    profile.attempts.clear();
    profile.adaptive = Some(AdaptiveExecutionProfile {
        max_selected_targets: 1,
        max_generated_attempts: 1,
        path_preferences: vec!["../outside".to_string()],
        allowed_change_kinds: vec![],
    });

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("path preference"));
}

#[test]
fn adaptive_execution_profile_defaults_builtin_change_kinds() {
    let adaptive = AdaptiveExecutionProfile {
        max_selected_targets: 1,
        max_generated_attempts: 4,
        path_preferences: vec![],
        allowed_change_kinds: vec![],
    };

    assert_eq!(
        adaptive.effective_change_kinds(),
        vec![
            AdaptiveChangeKind::ArithmeticSwap,
            AdaptiveChangeKind::ComparisonFlip,
            AdaptiveChangeKind::BooleanFlip,
            AdaptiveChangeKind::OrderingBoundaryFlip,
            AdaptiveChangeKind::ResultStatusFlip,
            AdaptiveChangeKind::NumericLiteralFlip,
        ]
    );
}
