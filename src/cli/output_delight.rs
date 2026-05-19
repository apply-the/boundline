use super::task_status_text;
use crate::domain::session::DelightFeedbackSignal;
use crate::domain::trace::InspectClosureView;

const INSPECT_CLOSURE_HEADLINE_SUFFIX: &str = "headline";
const INSPECT_CLOSURE_LINE_SUFFIX: &str = "line";
const INSPECT_CLOSURE_MISSING_INPUT_SUFFIX: &str = "missing_input";
const INSPECT_CLOSURE_NEXT_ACTION_SUFFIX: &str = "next_action";
const INSPECT_CLOSURE_SOURCE_ATTRIBUTION_SUFFIX: &str = "source_attribution";
const INSPECT_CLOSURE_TERMINAL_REASON_SUFFIX: &str = "terminal_reason";
const INSPECT_CLOSURE_TERMINAL_STATUS_SUFFIX: &str = "terminal_status";
const DELIGHT_LABEL_EXPLANATION_ATTRIBUTION_COUNTS: &str = "explanation_attribution_counts";
const DELIGHT_LABEL_EXPLANATION_ATTRIBUTION_RATE: &str = "explanation_attribution_rate";
const DELIGHT_LABEL_LATEST_NEXT_ACTION_OUTCOME: &str = "latest_next_action_outcome";
const DELIGHT_LABEL_LATEST_NEXT_ACTION_OVERRIDE_REASON: &str = "latest_next_action_override_reason";
const DELIGHT_LABEL_NEXT_ACTION_ACCEPTANCE_COUNTS: &str = "next_action_acceptance_counts";
const DELIGHT_LABEL_NEXT_ACTION_ACCEPTANCE_RATE: &str = "next_action_acceptance_rate";
const DELIGHT_LABEL_TIME_TO_FIRST_USEFUL_ANSWER_COMMAND: &str =
    "time_to_first_useful_answer_command";
const DELIGHT_LABEL_TIME_TO_FIRST_USEFUL_ANSWER_MS: &str = "time_to_first_useful_answer_ms";
const DELIGHT_NOT_YET_RECORDED: &str = "not-yet-recorded";
const DELIGHT_UNKNOWN_OUTCOME: &str = "unknown";

fn inspect_closure_label(view: &InspectClosureView, suffix: &str) -> String {
    format!("inspect_{}_{}", view.view_kind.as_str(), suffix)
}

pub(crate) fn inspect_closure_lines(view: &InspectClosureView) -> Vec<String> {
    let mut lines = vec![format!(
        "{}: {}",
        inspect_closure_label(view, INSPECT_CLOSURE_HEADLINE_SUFFIX),
        view.headline
    )];

    for narrative_line in &view.narrative_lines {
        lines.push(format!(
            "{}: {narrative_line}",
            inspect_closure_label(view, INSPECT_CLOSURE_LINE_SUFFIX)
        ));
    }
    for source_attribution in &view.source_attribution {
        lines.push(format!(
            "{}: {source_attribution}",
            inspect_closure_label(view, INSPECT_CLOSURE_SOURCE_ATTRIBUTION_SUFFIX)
        ));
    }
    for missing_input in &view.missing_inputs {
        lines.push(format!(
            "{}: {missing_input}",
            inspect_closure_label(view, INSPECT_CLOSURE_MISSING_INPUT_SUFFIX)
        ));
    }

    lines.push(format!(
        "{}: {}",
        inspect_closure_label(view, INSPECT_CLOSURE_TERMINAL_STATUS_SUFFIX),
        task_status_text(view.terminal_status)
    ));
    lines.push(format!(
        "{}: {}",
        inspect_closure_label(view, INSPECT_CLOSURE_TERMINAL_REASON_SUFFIX),
        view.terminal_reason
    ));

    if let Some(next_action) = &view.next_action {
        lines.push(format!(
            "{}: {next_action}",
            inspect_closure_label(view, INSPECT_CLOSURE_NEXT_ACTION_SUFFIX)
        ));
    }

    lines
}

fn delight_rate_text(rate: Option<f64>) -> String {
    rate.map(|value| format!("{value:.2}")).unwrap_or_else(|| DELIGHT_NOT_YET_RECORDED.to_string())
}

fn delight_time_to_first_useful_answer_ms(
    delight_feedback: Option<&DelightFeedbackSignal>,
    started_at: Option<u64>,
) -> Option<u64> {
    let first_useful_answer_at =
        delight_feedback.and_then(|feedback| feedback.first_useful_answer_at)?;
    let started_at = started_at?;
    first_useful_answer_at.checked_sub(started_at)
}

pub(crate) fn append_delight_feedback_lines(
    lines: &mut Vec<String>,
    delight_feedback: Option<&DelightFeedbackSignal>,
    started_at: Option<u64>,
) {
    let time_to_first_useful_answer_ms =
        delight_time_to_first_useful_answer_ms(delight_feedback, started_at)
            .map(|value| value.to_string())
            .unwrap_or_else(|| DELIGHT_NOT_YET_RECORDED.to_string());
    let time_to_first_useful_answer_command = delight_feedback
        .and_then(|feedback| feedback.first_useful_answer_command)
        .map(|surface| surface.as_str().to_string())
        .unwrap_or_else(|| DELIGHT_NOT_YET_RECORDED.to_string());
    let explanation_attribution_rate = delight_rate_text(
        delight_feedback.and_then(DelightFeedbackSignal::explanation_attribution_rate),
    );
    let next_action_acceptance_rate = delight_rate_text(
        delight_feedback.and_then(DelightFeedbackSignal::next_action_acceptance_rate),
    );
    let attributed_explanations =
        delight_feedback.map_or(0, |feedback| feedback.attributed_explanations);
    let total_explanations = delight_feedback.map_or(0, |feedback| feedback.total_explanations);
    let accepted_next_actions =
        delight_feedback.map_or(0, |feedback| feedback.accepted_next_actions);
    let overridden_next_actions =
        delight_feedback.map_or(0, |feedback| feedback.overridden_next_actions);
    let latest_next_action_outcome = delight_feedback
        .map(|feedback| feedback.next_action_outcome.as_str())
        .unwrap_or(DELIGHT_UNKNOWN_OUTCOME);

    lines.push(format!(
        "{DELIGHT_LABEL_TIME_TO_FIRST_USEFUL_ANSWER_MS}: {time_to_first_useful_answer_ms}"
    ));
    lines.push(format!(
        "{DELIGHT_LABEL_TIME_TO_FIRST_USEFUL_ANSWER_COMMAND}: {time_to_first_useful_answer_command}"
    ));
    lines.push(format!(
        "{DELIGHT_LABEL_EXPLANATION_ATTRIBUTION_RATE}: {explanation_attribution_rate}"
    ));
    lines.push(format!(
        "{DELIGHT_LABEL_EXPLANATION_ATTRIBUTION_COUNTS}: attributed={attributed_explanations} total={total_explanations}"
    ));
    lines.push(format!(
        "{DELIGHT_LABEL_NEXT_ACTION_ACCEPTANCE_RATE}: {next_action_acceptance_rate}"
    ));
    lines.push(format!(
        "{DELIGHT_LABEL_NEXT_ACTION_ACCEPTANCE_COUNTS}: accepted={accepted_next_actions} overridden={overridden_next_actions}"
    ));
    lines.push(format!("{DELIGHT_LABEL_LATEST_NEXT_ACTION_OUTCOME}: {latest_next_action_outcome}"));

    if let Some(override_reason) =
        delight_feedback.and_then(|feedback| feedback.override_reason.as_deref())
    {
        lines
            .push(format!("{DELIGHT_LABEL_LATEST_NEXT_ACTION_OVERRIDE_REASON}: {override_reason}"));
    }
}
