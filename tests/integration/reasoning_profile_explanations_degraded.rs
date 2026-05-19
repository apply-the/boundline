use boundline::cli::output::{render_session_status, render_trace_summary};

use crate::assistant_delight_support::{
    DEGRADED_CONFIDENCE_SUMMARY, DEGRADED_CONTRIBUTION, DEGRADED_FALLBACK_DISCLOSURE,
    DEGRADED_NEXT_ACTION, DEGRADED_SELECTION_REASON, degraded_session_status,
    degraded_trace_summary,
};

#[test]
fn degraded_reasoning_profile_explanations_surface_bounded_fallbacks() {
    let status_text = render_session_status(&degraded_session_status());
    let inspect_text = render_trace_summary(
        &degraded_trace_summary(),
        "latest-workspace-trace",
        "/boundline-next",
    );

    assert!(
        status_text.contains(&format!("why_summary: {DEGRADED_CONTRIBUTION}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("risk_summary: {DEGRADED_CONFIDENCE_SUMMARY}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("reasoning_selection_reason: {DEGRADED_SELECTION_REASON}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("reasoning_contribution: {DEGRADED_CONTRIBUTION}")),
        "{status_text}"
    );
    assert!(
        status_text
            .contains(&format!("reasoning_fallback_disclosure: {DEGRADED_FALLBACK_DISCLOSURE}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("next_best_action: {DEGRADED_NEXT_ACTION}")),
        "{status_text}"
    );
    assert!(
        status_text.contains(&format!("explain_plan_recovery: {DEGRADED_NEXT_ACTION}")),
        "{status_text}"
    );

    assert!(
        inspect_text.contains(&format!("why_summary: {DEGRADED_CONTRIBUTION}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("risk_summary: {DEGRADED_CONFIDENCE_SUMMARY}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("reasoning_selection_reason: {DEGRADED_SELECTION_REASON}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("reasoning_contribution: {DEGRADED_CONTRIBUTION}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text
            .contains(&format!("reasoning_fallback_disclosure: {DEGRADED_FALLBACK_DISCLOSURE}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("next_best_action: {DEGRADED_NEXT_ACTION}")),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(&format!("explain_plan_recovery: {DEGRADED_NEXT_ACTION}")),
        "{inspect_text}"
    );
}
