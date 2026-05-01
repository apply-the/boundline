use synod::cli::output::{render_compatibility_follow_up_status, render_session_status};
use synod::domain::session::{
    CompatibilityFollowUpMode, CompatibilityFollowUpView, ContinuityAuthority, SessionStatus,
    SessionStatusView,
};
use synod::domain::task::TaskStatus;

fn compatibility_follow_up() -> CompatibilityFollowUpView {
    CompatibilityFollowUpView {
        follow_up_mode: CompatibilityFollowUpMode::InspectOnly,
        trace_ref: "/tmp/workspace/.synod/traces/compat.json".to_string(),
        routing_summary:
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
                .to_string(),
        execution_condition: "terminal - work completed successfully".to_string(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: "work completed successfully".to_string(),
        next_command: "synod inspect --workspace /tmp/workspace".to_string(),
    }
}

#[test]
fn render_compatibility_follow_up_status_surfaces_authority_and_inspect_command() {
    let rendered = render_compatibility_follow_up_status(
        "/tmp/workspace",
        ContinuityAuthority::CompatibilityTrace,
        &compatibility_follow_up(),
        "latest compatibility trace is authoritative",
    );

    assert!(rendered.contains("continuity_authority: compatibility_trace"), "{rendered}");
    assert!(rendered.contains("route_owner: compatibility"), "{rendered}");
    assert!(
        rendered.contains(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("compatibility_follow_up: inspect_only"), "{rendered}");
    assert!(
        rendered.contains("next_command: synod inspect --workspace /tmp/workspace"),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_compatibility_follow_up_without_replacing_native_routing() {
    let rendered = render_session_status(&SessionStatusView {
        session_id: "session-1".to_string(),
        workspace_ref: "/tmp/workspace".to_string(),
        goal: Some("Fix the failing add test".to_string()),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        authored_input_summary: None,
        authored_input_sources: None,
        authored_input_deduplicated_sources: None,
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: None,
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: Some(
            "confirmed (bug-fix) - operator confirmed flow during planning".to_string(),
        ),
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: Some(ContinuityAuthority::NativeSession),
        compatibility_follow_up: Some(compatibility_follow_up()),
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: None,
        current_step_id: None,
        current_step_index: None,
        latest_status: SessionStatus::Planned,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: None,
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
        latest_exhaustion_reason: None,
        latest_review_trigger: None,
        latest_review_vote: None,
        latest_review_outcome: None,
        latest_review_headline: None,
        latest_governance_stage: None,
        latest_governance_runtime: None,
        latest_governance_mode: None,
        latest_governance_run_ref: None,
        latest_governance_state: None,
        latest_governance_blocked_reason: None,
        latest_governance_packet_ref: None,
        latest_governance_packet_source_stage: None,
        latest_governance_packet_binding_reason: None,
        latest_governance_approval: None,
        latest_governance_decision: None,
        latest_governance_candidates: None,
        governance_next_action: None,
        next_command: Some("synod run".to_string()),
        explanation: "native session remains authoritative".to_string(),
    });

    assert!(rendered.contains("routing: native (goal_plan)"), "{rendered}");
    assert!(rendered.contains("route_owner: native"), "{rendered}");
    assert!(rendered.contains("continuity_authority: native_session"), "{rendered}");
    assert!(
        rendered.contains(
            "route_config_projection: flow_state=confirmed (bug-fix) - operator confirmed flow during planning"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("compatibility_follow_up: inspect_only"), "{rendered}");
    assert!(
        rendered
            .contains("compatibility_follow_up_command: synod inspect --workspace /tmp/workspace"),
        "{rendered}"
    );
    assert!(rendered.contains("next_command: synod run"), "{rendered}");
}
