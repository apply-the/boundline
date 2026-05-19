use boundline::cli::output::{render_session_status, render_trace_summary};

use crate::assistant_delight_support::{
    ACTIVE_CONTRIBUTION, ACTIVE_SELECTION_REASON, DEGRADED_CONTRIBUTION,
    DEGRADED_FALLBACK_DISCLOSURE, DEGRADED_SELECTION_REASON, active_session_status,
    active_trace_summary, degraded_session_status, degraded_trace_summary,
};

#[test]
fn active_reasoning_profile_projection_contract_surfaces_selection_and_contribution() {
    let status_text = render_session_status(&active_session_status());
    let inspect_text =
        render_trace_summary(&active_trace_summary(), "latest-workspace-trace", "/boundline-next");

    for text in [&status_text, &inspect_text] {
        assert!(
            text.contains(&format!("reasoning_selection_reason: {ACTIVE_SELECTION_REASON}")),
            "{text}"
        );
        assert!(text.contains(&format!("reasoning_contribution: {ACTIVE_CONTRIBUTION}")), "{text}");
        assert!(!text.contains("reasoning_fallback_disclosure:"), "{text}");
        assert!(text.contains("why_summary:"), "{text}");
        assert!(text.contains("risk_summary:"), "{text}");
        assert!(text.contains("evidence_summary:"), "{text}");
        assert!(text.contains("challenge_required_review:"), "{text}");
        assert!(text.contains("explain_plan_governance:"), "{text}");
    }
}

#[test]
fn degraded_reasoning_profile_projection_contract_surfaces_explicit_fallback() {
    let status_text = render_session_status(&degraded_session_status());
    let inspect_text = render_trace_summary(
        &degraded_trace_summary(),
        "latest-workspace-trace",
        "/boundline-next",
    );

    for text in [&status_text, &inspect_text] {
        assert!(
            text.contains(&format!("reasoning_selection_reason: {DEGRADED_SELECTION_REASON}")),
            "{text}"
        );
        assert!(
            text.contains(&format!("reasoning_contribution: {DEGRADED_CONTRIBUTION}")),
            "{text}"
        );
        assert!(
            text.contains(&format!(
                "reasoning_fallback_disclosure: {DEGRADED_FALLBACK_DISCLOSURE}"
            )),
            "{text}"
        );
        assert!(text.contains("why_summary:"), "{text}");
        assert!(text.contains("risk_summary:"), "{text}");
        assert!(text.contains("challenge_required_review:"), "{text}");
        assert!(text.contains("explain_plan_recovery:"), "{text}");
    }
}
