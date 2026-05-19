use boundline::cli::output::{render_session_status, render_trace_summary};

use crate::assistant_delight_support::{
    ACTIVE_CONFIDENCE_SUMMARY, ACTIVE_CONTRIBUTION, ACTIVE_SELECTION_REASON, active_session_status,
    active_trace_summary,
};

#[test]
fn active_reasoning_profile_explanations_stay_profile_aware_across_status_and_inspect() {
    let status_text = render_session_status(&active_session_status());
    let inspect_text =
        render_trace_summary(&active_trace_summary(), "latest-workspace-trace", "/boundline-next");

    assert!(status_text.contains(&format!("why_summary: {ACTIVE_CONTRIBUTION}")), "{status_text}");
    assert!(
        status_text.contains(&format!("risk_summary: {ACTIVE_CONFIDENCE_SUMMARY}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("reasoning_selection_reason: {ACTIVE_SELECTION_REASON}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("reasoning_contribution: {ACTIVE_CONTRIBUTION}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("explain_plan_summary: goal=Explain the bounded runtime change; stages=bug-fix/verify; reasoning={ACTIVE_CONTRIBUTION}")),
        "{status_text}"
    );

    assert!(
        inspect_text.contains(&format!("why_summary: {ACTIVE_CONTRIBUTION}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("risk_summary: {ACTIVE_CONFIDENCE_SUMMARY}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("reasoning_selection_reason: {ACTIVE_SELECTION_REASON}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("reasoning_contribution: {ACTIVE_CONTRIBUTION}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("explain_plan_summary: goal=Explain the bounded runtime change; stages=trace_inspect; reasoning={ACTIVE_CONTRIBUTION}")),
        "{inspect_text}"
    );
}
