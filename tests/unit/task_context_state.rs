use boundline::domain::cluster::{
    ClusterDeliveryStory, ClusterRouteOwner, ClusterSessionProjection, ClusteredExecutionCondition,
    ClusteredExecutionKind, WorkspaceParticipationKind, WorkspaceParticipationRecord,
};
use boundline::domain::governance::{
    ApprovalState, AutopilotAction, AutopilotDecisionRecord, CanonCapabilitySnapshot,
    CompactedCanonMemory, GovernanceLifecycleState, GovernanceRuntimeKind, GovernedStagePacket,
    GovernedStageRecord, MemoryCredibilityState, PacketReadiness, PacketReuseBinding,
};
use boundline::domain::limits::RunLimits;
use boundline::domain::step::{ErrorInfo, Recoverability, Step, StepResultSummary};
use boundline::domain::task::{
    ClarificationReasonKind, ClarificationRecord, ClarificationStatus, DerivedTaskDraft,
};
use boundline::domain::task_context::{
    LATEST_CANON_CAPABILITY_SNAPSHOT_KEY, LATEST_COMPACTED_CANON_MEMORY_KEY,
    LATEST_GOVERNANCE_STAGE_KEY, TaskContext, TaskContextError,
};
use serde_json::{Map, Value, json};

#[test]
fn task_context_merges_state_patches_and_repairs_nested_buckets() {
    let mut initial_state = Map::new();
    initial_state.insert("step_outputs".to_string(), Value::String("broken".to_string()));

    let mut context = TaskContext::new(
        "session-context",
        "/tmp/boundline-context",
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
        boundline::domain::step::ErrorInfo::new("retryable", "try again"),
        Recoverability::Retryable,
    );
    context.set_last_result(StepResultSummary::from_step(&failed_step));
    context.push_history_ref("attempt-1");

    assert_eq!(context.history_refs, vec!["attempt-1".to_string()]);
    assert_eq!(context.last_result.as_ref().unwrap().step_id, "verify");
}

#[test]
fn task_context_round_trips_governance_state_records() {
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/boundline-context",
        RunLimits::default(),
        Map::new(),
    );
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
        reason_code: None,
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
        "/tmp/boundline-context",
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
fn task_context_round_trips_canon_snapshot_and_compacted_memory() {
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/boundline-context",
        RunLimits::default(),
        Map::new(),
    );
    let snapshot = CanonCapabilitySnapshot {
        canon_version: "0.45.0".to_string(),
        supported_schema_versions: vec!["2026-02-01".to_string()],
        operations: vec!["start".to_string(), "refresh".to_string(), "capabilities".to_string()],
        supported_modes: Vec::new(),
        status_values: vec!["governed_ready".to_string()],
        approval_state_values: vec!["not_needed".to_string()],
        packet_readiness_values: vec!["reusable".to_string()],
        compatibility_notes: vec!["stable-json".to_string()],
    };
    let memory = CompactedCanonMemory {
        headline: "Canon verification packet is still credible".to_string(),
        credibility: MemoryCredibilityState::Credible,
        stage_key: Some("change:verify".to_string()),
        run_ref: Some("run-123".to_string()),
        packet_ref: Some(".canon/runs/run-123".to_string()),
        reason_code: None,
        artifact_refs: vec![".canon/runs/run-123/verification.md".to_string()],
        mode_summary: None,
        possible_actions: Vec::new(),
        recommended_next_action: None,
        evidence_summary: None,
    };

    context.set_latest_canon_capability_snapshot(&snapshot).unwrap();
    context.set_latest_compacted_canon_memory(&memory).unwrap();

    assert_eq!(context.latest_canon_capability_snapshot().unwrap(), Some(snapshot));
    assert_eq!(context.latest_compacted_canon_memory().unwrap(), Some(memory));
}

#[test]
fn task_context_reports_invalid_canon_memory_payloads() {
    let mut initial_state = Map::new();
    initial_state.insert(LATEST_CANON_CAPABILITY_SNAPSHOT_KEY.to_string(), json!("broken"));
    initial_state.insert(LATEST_COMPACTED_CANON_MEMORY_KEY.to_string(), json!("broken"));
    let context = TaskContext::new(
        "session-context",
        "/tmp/boundline-context",
        RunLimits::default(),
        initial_state,
    );

    let snapshot_error = context.latest_canon_capability_snapshot().unwrap_err();
    assert!(matches!(
        snapshot_error,
        TaskContextError::StateDeserializationFailed { ref key, .. }
            if key == LATEST_CANON_CAPABILITY_SNAPSHOT_KEY
    ));

    let memory_error = context.latest_compacted_canon_memory().unwrap_err();
    assert!(matches!(
        memory_error,
        TaskContextError::StateDeserializationFailed { ref key, .. }
            if key == LATEST_COMPACTED_CANON_MEMORY_KEY
    ));
}

#[test]
fn task_context_failure_helpers_cover_workspace_membership_and_error_buckets() {
    let mut initial_state = Map::new();
    initial_state.insert("step_errors".to_string(), Value::String("broken".to_string()));
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/boundline-context",
        RunLimits::default(),
        initial_state,
    );
    let error = ErrorInfo::new("terminal", "boom");

    assert!(context.belongs_to_workspace("/tmp/boundline-context"));
    assert!(!context.belongs_to_workspace("/tmp/other-workspace"));

    context.apply_failure_error("verify", &error);

    assert_eq!(context.state["last_step_id"], json!("verify"));
    assert_eq!(context.state["last_error"]["code"], json!("terminal"));
    assert_eq!(context.state["step_errors"]["verify"]["message"], json!("boom"));
}

#[test]
fn task_context_round_trips_clarification_and_derived_draft_records() {
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/boundline-context",
        RunLimits::default(),
        Map::new(),
    );
    let clarification = ClarificationRecord {
        clarification_id: "clarification-1".to_string(),
        reason_kind: ClarificationReasonKind::UnboundedRequest,
        prompt: "Narrow the request to one bounded outcome".to_string(),
        missing_fields: vec!["bounded_scope".to_string()],
        blocking_sources: vec!["source-1".to_string()],
        turn_index: 1,
        status: ClarificationStatus::Open,
    };
    let draft = DerivedTaskDraft {
        draft_id: "draft-1".to_string(),
        bundle_id: "bundle-1".to_string(),
        bounded_goal: "Improve the platform docs and fix whatever tests are broken".to_string(),
        flow_hint: Some("bug-fix".to_string()),
        planning_ready: false,
        validation_targets: vec!["docs/brief.md".to_string()],
        blocking_clarification_ref: Some(clarification.clarification_id.clone()),
    };

    context.set_latest_clarification(&clarification).unwrap();
    context.set_derived_task_draft(&draft).unwrap();

    assert_eq!(context.latest_clarification().unwrap(), Some(clarification));
    assert_eq!(context.derived_task_draft().unwrap(), Some(draft));
}

#[test]
fn task_context_round_trips_cluster_session_projection() {
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/cluster-primary",
        RunLimits::default(),
        Map::new(),
    );
    let projection = ClusterSessionProjection {
        cluster_id: "cluster-1".to_string(),
        primary_workspace_ref: "/tmp/cluster-primary".to_string(),
        member_workspace_refs: vec![
            "/tmp/cluster-primary".to_string(),
            "/tmp/cluster-member".to_string(),
        ],
        started_from_command: "run".to_string(),
        updated_at: 42,
    };

    context.set_cluster_session_projection(&projection).unwrap();

    assert_eq!(context.cluster_session_projection().unwrap(), Some(projection));
}

#[test]
fn task_context_round_trips_cluster_delivery_story() {
    let mut context = TaskContext::new(
        "session-context",
        "/tmp/cluster-primary",
        RunLimits::default(),
        Map::new(),
    );
    let story = ClusterDeliveryStory {
        cluster_id: "cluster-1".to_string(),
        primary_workspace_ref: "/tmp/cluster-primary".to_string(),
        authoritative_workspace_ref: "/tmp/cluster-member".to_string(),
        route_owner: ClusterRouteOwner::Native,
        member_workspace_refs: vec![
            "/tmp/cluster-primary".to_string(),
            "/tmp/cluster-member".to_string(),
        ],
        participating_workspaces: vec![WorkspaceParticipationRecord {
            workspace_ref: "/tmp/cluster-primary".to_string(),
            participation_kind: WorkspaceParticipationKind::Entry,
            order: 0,
            latest_trace_ref: Some("/tmp/cluster-primary/.boundline/traces/task.json".to_string()),
            latest_status: Some("succeeded".to_string()),
            headline: "primary workspace executed the entry slice".to_string(),
            terminal_reason: None,
        }],
        started_from_command: "run".to_string(),
        execution_condition: ClusteredExecutionCondition {
            kind: ClusteredExecutionKind::Paused,
            active_workspace_ref: Some("/tmp/cluster-member".to_string()),
            blocking_workspace_ref: None,
            summary: "handoff to the next workspace is ready".to_string(),
            recovery_allowed: true,
        },
        updated_at: 42,
    };

    context.set_cluster_delivery_story(&story).unwrap();

    assert_eq!(context.cluster_delivery_story().unwrap(), Some(story));
}
