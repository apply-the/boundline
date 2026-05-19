use boundline::cli::output::{render_session_status, render_trace_summary};

use crate::assistant_delight_support::{
    DELIGHT_TIME_TO_FIRST_USEFUL_ANSWER_MS, active_session_status, active_trace_summary,
};

#[test]
fn delight_feedback_signals_are_projected_through_status_and_inspect() {
    let status_text = render_session_status(&active_session_status());
    let inspect_text =
        render_trace_summary(&active_trace_summary(), "latest-workspace-trace", "/boundline-next");

    for text in [&status_text, &inspect_text] {
        assert!(
            text.contains(&format!(
                "time_to_first_useful_answer_ms: {DELIGHT_TIME_TO_FIRST_USEFUL_ANSWER_MS}"
            )),
            "{text}"
        );
        assert!(text.contains("time_to_first_useful_answer_command: explain_plan"), "{text}");
        assert!(text.contains("explanation_attribution_rate: 1.00"), "{text}");
        assert!(text.contains("next_action_acceptance_rate: 1.00"), "{text}");
        assert!(text.contains("latest_next_action_outcome: accepted"), "{text}");
    }
}
