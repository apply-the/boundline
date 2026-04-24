use synod::domain::limits::{RunLimits, TerminalCondition};
use synod::domain::task::TaskStatus;
use synod::orchestrator::terminal::{select_terminal_condition, task_status_for_condition};

#[test]
fn terminal_precedence_prefers_the_earliest_configured_condition() {
    let limits = RunLimits::default();
    let selected = select_terminal_condition(
        &limits.terminal_precedence,
        &[TerminalCondition::StepLimitExceeded, TerminalCondition::GoalSatisfied],
    );

    assert_eq!(selected, Some(TerminalCondition::GoalSatisfied));
}

#[test]
fn terminal_condition_mapping_distinguishes_failed_and_exhausted_states() {
    assert_eq!(
        task_status_for_condition(TerminalCondition::RetryBudgetExhausted),
        TaskStatus::Exhausted
    );
    assert_eq!(
        task_status_for_condition(TerminalCondition::NoCredibleNextStep),
        TaskStatus::Failed
    );
}
