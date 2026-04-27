use serde_json::{Map, Value, json};
use synod::domain::governance::{
    ApprovalState, AutopilotAction, AutopilotDecisionRecord, GovernanceLifecycleState,
    GovernanceRuntimeKind, GovernedStagePacket, GovernedStageRecord, PacketReadiness,
    PacketReuseBinding,
};
use synod::domain::limits::RunLimits;
use synod::domain::step::{ErrorInfo, Recoverability, Step, StepResultSummary};
use synod::domain::task_context::{LATEST_GOVERNANCE_STAGE_KEY, TaskContext, TaskContextError};

#[test]
fn task_context_merges_state_patches_and_repairs_nested_buckets() {
    let mut initial_state = Map::new();
    initial_state.insert("step_outputs".to_string(), Value::String("broken".to_string()));

    let mut context = TaskContext::new(
        "session-context",
        "/tmp/synod-context",
        RunLimits::default(),
        initial_state,
    );

    let mut patch = Map::new();
    patch.insert("goal_satisfied".to_string(), json!(true));
    patch.insert("verified".to_string(), json!(true));

    context.apply_success_output("analyze", &json!({"analysis": "complete"}), Some(&patch));

    assert_eq!(context.state["analysis"], json!("complete"));
    assert_eq!(context.state["goal_satisfied"], json!(true));
    assert_eq!(context.state["verified"], json!(true));
    assert_eq!(context.state["step_outputs"]["analyze"]["analysis"], json!("complete"));

    let mut failed_step = Step::decision("verify", json!({})).unwrap();
    failed_step.mark_failed(
        synod::domain::step::ErrorInfo::new("retryable", "try again"),
        Recoverability::Retryable,
    );
    context.set_last_result(StepResultSummary::from_step(&failed_step));
    context.push_history_ref("attempt-1");

    assert_eq!(context.history_refs, vec!["attempt-1".to_string()]);
    assert_eq!(context.last_result.as_ref().unwrap().step_id, "verify");
}

#[test]
fn task_context_round_trips_governance_state_records() {
    let mut context =
        TaskContext::new("session-context", "/tmp/synod-context", RunLimits::default(), Map::new());
    let record = GovernedStageRecord {
        stage_key: "bug-fix:investigate".to_string(),
        runtime: GovernanceRuntimeKind::Local,
        lifecycle_state: GovernanceLifecycleState::GovernedReady,
        required: false,
        autopilot_enabled: false,
        approval_state: ApprovalState::NotNeeded,
        canon_run_ref: None,
        governance_attempt_id: "attempt-1".to_string(),
        previous_governance_attempt_id: None,
        packet_ref: Some("packet-1".to_string()),
        decision_ref: Some("decision-1".to_string()),
        blocked_reason: None,
    };
    let packet = GovernedStagePacket {
        packet_ref: "packet-1".to_string(),
        runtime: GovernanceRuntimeKind::Local,
        canon_mode: None,
        expected_document_refs: vec!["packet-1/brief.md".to_string()],
        document_refs: vec!["packet-1/brief.md".to_string()],
        readiness: PacketReadiness::Reusable,
        missing_sections: Vec::new(),
        headline: "local packet".to_string(),
    };
    let reuse = PacketReuseBinding {
        upstream_stage_key: "bug-fix:investigate".to_string(),
        downstream_stage_key: "bug-fix:implement".to_string(),
        packet_ref: packet.packet_ref.clone(),
        binding_reason: "reuse immediate upstream packet".to_string(),
    };
    let decision = AutopilotDecisionRecord {
        decision_id: "decision-1".to_string(),
        stage_key: "bug-fix:investigate".to_string(),
        candidate_actions: vec![AutopilotAction::SelectMode],
        candidate_modes: Vec::new(),
        selected_action: Some(AutopilotAction::SelectMode),
        selected_mode: None,
        selected_target_stage_key: None,
        rationale: "local runtime already selected".to_string(),
        blocked_reason: None,
    };

    context.set_latest_governance_stage(&record).unwrap();
    context.set_latest_governance_packet(&packet).unwrap();
    context.set_latest_governance_packet_reuse(&reuse).unwrap();
    context.set_latest_governance_decision(&decision).unwrap();

    assert_eq!(context.latest_governance_stage().unwrap(), Some(record));
    assert_eq!(context.latest_governance_packet().unwrap(), Some(packet));
    assert_eq!(context.latest_governance_packet_reuse().unwrap(), Some(reuse));
    assert_eq!(context.latest_governance_decision().unwrap(), Some(decision));
}

#[test]
fn task_context_reports_invalid_governance_state_payloads() {
    let mut initial_state = Map::new();
    initial_state.insert(LATEST_GOVERNANCE_STAGE_KEY.to_string(), json!("broken"));
    let context = TaskContext::new(
        "session-context",
        "/tmp/synod-context",
        RunLimits::default(),
        initial_state,
    );

    let error = context.latest_governance_stage().unwrap_err();
    assert!(matches!(
        error,
        TaskContextError::StateDeserializationFailed { ref key, .. }
            if key == LATEST_GOVERNANCE_STAGE_KEY
    ));
}

#[test]
fn task_context_failure_helpers_cover_workspace_membership_and_error_buckets() {
    let mut initial_state = Map::new();
    initial_state.insert("step_errors".to_string(), Value::String("broken".to_string()));
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/synod-context",
        RunLimits::default(),
        initial_state,
    );
    let error = ErrorInfo::new("terminal", "boom");

    assert!(context.belongs_to_workspace("/tmp/synod-context"));
    assert!(!context.belongs_to_workspace("/tmp/other-workspace"));

    context.apply_failure_error("verify", &error);

    assert_eq!(context.state["last_step_id"], json!("verify"));
    assert_eq!(context.state["last_error"]["code"], json!("terminal"));
    assert_eq!(context.state["step_errors"]["verify"]["message"], json!("boom"));
}
