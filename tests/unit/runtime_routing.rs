use std::error::Error;
use std::fs;

use serde_json::json;

use boundline::adapters::audit_store::{FileSessionAuditStore, FrameworkAdapterHookAuditStore};
use boundline::domain::brief::{
    AuthoredBriefBundle, AuthoredBriefResolutionState, GoalQualityAssessment, InputSourceKind,
    InputSourceReference,
};
use boundline::domain::flow_policy::FlowPolicy;
use boundline::domain::framework_adapter::{
    AdapterExecutionSource, AdapterFailureClass, AdapterHookKey, AdapterLifecycleStageKey,
    HookDispatchStatus, LifecycleStageExecutionStatus, StageClaimState, StageRoutingDecisionReason,
};
use boundline::domain::goal_plan::{GoalPlan, GoalPlanFlowMode, InferredFlow, PlannedTask};
use boundline::domain::limits::RunLimits;
use boundline::domain::negotiation::{
    NegotiatedDeliveryPacket, NegotiationConstraint, NegotiationConstraintKind,
    NegotiationConstraintSource, NegotiationConstraintState, NegotiationResolutionState,
};
use boundline::domain::plan::Plan;
use boundline::domain::session::{ActiveSessionRecord, RoutingMode, RoutingSource, SessionStatus};
use boundline::domain::step::Step;
use boundline::domain::task::{
    ClarificationReasonKind, ClarificationRecord, ClarificationStatus, Task, TaskRunRequest,
};
use boundline::domain::trace::HookEventDispatchRecord;
use boundline::normalize_brief_inputs;
use boundline::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};
use uuid::Uuid;

fn build_task(workspace_ref: &str) -> Task {
    let request = TaskRunRequest {
        goal: "Deliver runtime refoundation".to_string(),
        input: json!({"ticket": "RUNTIME-15"}),
        session_id: "session-1".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    };
    let plan =
        Plan::new(vec![Step::decision("analyze", json!({"phase": "bootstrap"})).unwrap()]).unwrap();

    Task::new("task-1", &request, plan).unwrap()
}

fn build_goal_plan(confirmed: bool) -> GoalPlan {
    let mut goal_plan = GoalPlan::new(
        "Fix the failing add test",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Fix the broken arithmetic path".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: None,
            depends_on: None,
        }],
    )
    .unwrap();
    goal_plan.flow = Some(InferredFlow {
        flow_name: "bug-fix".to_string(),
        confidence_reason: "goal contains keyword 'fix'".to_string(),
        confirmed,
    });
    if confirmed {
        goal_plan.confirm().unwrap();
    }
    goal_plan
}

#[test]
fn goal_plan_flow_state_reports_proposed_and_confirmed_modes() {
    let proposed = build_goal_plan(false);
    assert_eq!(proposed.flow_state().mode, GoalPlanFlowMode::Proposed);
    assert_eq!(proposed.flow_state().flow_name.as_deref(), Some("bug-fix"));

    let confirmed = build_goal_plan(true);
    assert_eq!(confirmed.flow_state().mode, GoalPlanFlowMode::Confirmed);
    assert_eq!(confirmed.flow_state().flow_name.as_deref(), Some("bug-fix"));

    let unconstrained = GoalPlan::new(
        "Implement a workspace summary",
        vec![PlannedTask {
            task_id: "planned-task-2".to_string(),
            description: "Implement the summary".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("summary added".to_string()),
            decision_type_hint: None,
            depends_on: None,
        }],
    )
    .unwrap();
    assert_eq!(unconstrained.flow_state().mode, GoalPlanFlowMode::Absent);
}

#[test]
fn framework_adapter_hook_audit_store_round_trips_sidecar_records() -> Result<(), Box<dyn Error>> {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-runtime-routing-hook-audit-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)?;

    let store = FileSessionAuditStore::for_session(&workspace, "session-1");
    let record = HookEventDispatchRecord {
        run_id: "run-1".to_string(),
        hook_key: AdapterHookKey::StageCompleted,
        stage_key: AdapterLifecycleStageKey::Plan,
        adapter_id: "speckit".to_string(),
        stage_claimed: true,
        payload_ref: ".boundline/traces/trace-1.json".to_string(),
        dispatch_status: HookDispatchStatus::Delivered,
        summary: "hook delivered".to_string(),
        recorded_at: 42,
    };

    let path = store.append_hook_dispatch(&record)?;
    assert!(path.ends_with("framework-adapter-hooks.jsonl"));

    let loaded = store.load_hook_dispatches()?;
    assert_eq!(loaded, vec![record]);

    Ok(())
}

#[test]
fn framework_adapter_domain_strings_cover_remaining_variant_helpers() {
    assert_eq!(AdapterLifecycleStageKey::Goal.as_str(), "goal");
    assert_eq!(AdapterLifecycleStageKey::Plan.as_str(), "plan");
    assert_eq!(AdapterLifecycleStageKey::Run.as_str(), "run");
    assert_eq!(AdapterLifecycleStageKey::Review.as_str(), "review");

    assert_eq!(AdapterHookKey::StageCompleted.as_str(), "stage_completed");
    assert_eq!(AdapterHookKey::StageFailed.as_str(), "stage_failed");

    assert_eq!(AdapterExecutionSource::BuiltIn.as_str(), "built_in");
    assert_eq!(AdapterExecutionSource::Adapter.as_str(), "adapter");

    assert_eq!(StageRoutingDecisionReason::NoAdapterSelected.as_str(), "no_adapter_selected");
    assert_eq!(StageRoutingDecisionReason::UndeclaredStage.as_str(), "undeclared_stage");
    assert_eq!(StageRoutingDecisionReason::DeclaredOverride.as_str(), "declared_override");
    assert_eq!(StageRoutingDecisionReason::PreflightBlocked.as_str(), "preflight_blocked");
    assert_eq!(StageRoutingDecisionReason::InvalidManifest.as_str(), "invalid_manifest");
    assert_eq!(StageRoutingDecisionReason::CompatibilityBlocked.as_str(), "compatibility_blocked");

    assert_eq!(StageClaimState::NotClaimed.as_str(), "not_claimed");
    assert_eq!(StageClaimState::Claimed.as_str(), "claimed");
    assert_eq!(StageClaimState::Completed.as_str(), "completed");
    assert_eq!(StageClaimState::FailedAfterClaim.as_str(), "failed_after_claim");

    assert_eq!(AdapterFailureClass::Preflight.as_str(), "preflight");
    assert_eq!(AdapterFailureClass::Manifest.as_str(), "manifest");
    assert_eq!(AdapterFailureClass::MissingConfig.as_str(), "missing_config");
    assert_eq!(AdapterFailureClass::AdapterRuntime.as_str(), "adapter_runtime");
    assert_eq!(AdapterFailureClass::Compatibility.as_str(), "compatibility");
    assert_eq!(AdapterFailureClass::ProtocolError.as_str(), "protocol_error");
    assert_eq!(AdapterFailureClass::TransportFailure.as_str(), "transport_failure");
    assert_eq!(AdapterFailureClass::HookWarningOnly.as_str(), "hook_warning_only");

    assert_eq!(LifecycleStageExecutionStatus::Succeeded.as_str(), "succeeded");
    assert_eq!(LifecycleStageExecutionStatus::Failed.as_str(), "failed");
    assert_eq!(LifecycleStageExecutionStatus::Blocked.as_str(), "blocked");
    assert_eq!(LifecycleStageExecutionStatus::Skipped.as_str(), "skipped");

    assert_eq!(HookDispatchStatus::Delivered.as_str(), "delivered");
    assert_eq!(HookDispatchStatus::Ignored.as_str(), "ignored");
    assert_eq!(HookDispatchStatus::Warning.as_str(), "warning");
    assert_eq!(HookDispatchStatus::Failed.as_str(), "failed");
}

#[test]
fn flow_policy_helpers_report_stage_id_and_progress() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert_eq!(policy.current_stage_id(), Some("investigate"));
    assert_eq!(policy.stage_progress(), Some((1, 3)));

    policy.advance_stage().unwrap();
    assert_eq!(policy.current_stage_id(), Some("implement"));
    assert_eq!(policy.stage_progress(), Some((2, 3)));
}

#[test]
fn session_runtime_resolve_routing_outcome_routes_native_for_proposed_plan() {
    let workspace = std::env::temp_dir().join("boundline-runtime-routing-pending");
    std::fs::create_dir_all(&workspace).unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let record = ActiveSessionRecord {
        session_id: "session-native".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Fix the failing add test".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(build_goal_plan(false)),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let outcome = runtime.resolve_routing_outcome(&record).unwrap();
    assert_eq!(outcome.mode, RoutingMode::Native);
    assert_eq!(outcome.source, RoutingSource::GoalPlan);
    assert!(outcome.reason.contains("native execution"));
}

#[test]
fn session_runtime_resolve_routing_outcome_uses_compatibility_when_only_task_exists() {
    let workspace = std::env::temp_dir().join("boundline-runtime-routing-compat");
    std::fs::create_dir_all(&workspace).unwrap();
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let record = ActiveSessionRecord {
        session_id: "session-fixture".to_string(),
        workspace_ref: workspace_ref.clone(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(build_task(&workspace_ref)),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Running,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let outcome = runtime.resolve_routing_outcome(&record).unwrap();
    assert_eq!(outcome.mode, RoutingMode::Compatibility);
    assert_eq!(outcome.source, RoutingSource::ExecutionProfile);
    assert!(outcome.reason.contains("compatibility"));
}

#[test]
fn plan_task_blocks_when_context_pack_is_not_credible() {
    let workspace = std::env::temp_dir().join("boundline-runtime-routing-blocked-context");
    std::fs::create_dir_all(&workspace).unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = ActiveSessionRecord {
        session_id: "session-blocked-context".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("investigate a thing".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let err = runtime.plan_task(&mut record, None, false).unwrap_err();

    assert!(matches!(err, SessionRuntimeError::ClarificationRequired { .. }));
    assert_eq!(record.latest_status, SessionStatus::Blocked);
    let goal_plan = record.goal_plan.as_ref().unwrap();
    assert_eq!(goal_plan.status, boundline::domain::goal_plan::GoalPlanStatus::Draft);
    assert_eq!(goal_plan.context_credibility().as_deref(), Some("insufficient"));
    assert!(
        goal_plan.context_summary().as_deref().unwrap().contains("no credible bounded context")
    );
    record.validate().unwrap();
}

#[test]
fn plan_task_records_authored_brief_context_on_empty_workspace() {
    let workspace = std::env::temp_dir().join("boundline-runtime-routing-authored-context");
    std::fs::create_dir_all(&workspace).unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let brief =
        normalize_brief_inputs(&workspace, Some("Document the runtime routing contract"), &[])
            .unwrap();
    let mut record = ActiveSessionRecord {
        session_id: "session-authored-context".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some(brief.render_goal_text()),
        authored_brief: Some(brief.clone()),
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let err = runtime.plan_task(&mut record, None, false).unwrap_err();

    let goal_plan = record.goal_plan.as_ref().unwrap();
    assert!(matches!(err, SessionRuntimeError::ClarificationRequired { .. }));
    assert_eq!(goal_plan.context_credibility().as_deref(), Some("insufficient"));
    assert!(goal_plan.context_primary_inputs().contains(&brief.summary_text()));
    assert!(
        goal_plan.context_provenance_lines().iter().any(|line| line.contains("authored_brief"))
    );
    assert_eq!(record.latest_status, SessionStatus::Blocked);
}

#[test]
fn repeated_plan_task_revises_goal_plan_when_workspace_evidence_changes() {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-runtime-routing-replan-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"boundline_runtime_routing_replan\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        workspace.join("src/dashboard.rs"),
        "pub fn render_dashboard() {}\npub struct DashboardState;",
    )
    .unwrap();
    std::fs::write(
        workspace.join("brief.md"),
        "Focus on src/dashboard.rs for the bounded dashboard surface.\n",
    )
    .unwrap();
    let brief = normalize_brief_inputs(
        &workspace,
        Some("shape dashboard surface"),
        &[std::path::PathBuf::from("brief.md")],
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = ActiveSessionRecord {
        session_id: "session-runtime-replan".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("shape dashboard surface".to_string()),
        authored_brief: Some(brief),
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    runtime.plan_task(&mut record, None, false).unwrap();
    let initial_plan = record.goal_plan.as_ref().unwrap();
    assert_eq!(initial_plan.proposal_revision, 1);
    assert_eq!(initial_plan.flow.as_ref().map(|flow| flow.flow_name.as_str()), Some("change"));

    std::fs::create_dir_all(workspace.join("tests")).unwrap();
    std::fs::write(
        workspace.join("tests/dashboard.rs"),
        "use boundline_runtime_routing_replan::dashboard::render_dashboard;\n#[test]\nfn dashboard_regression() { render_dashboard(); }",
    )
    .unwrap();

    runtime.plan_task(&mut record, None, false).unwrap();

    let revised_plan = record.goal_plan.as_ref().unwrap();
    assert_eq!(revised_plan.proposal_revision, 2);
    assert_eq!(revised_plan.flow.as_ref().map(|flow| flow.flow_name.as_str()), Some("bug-fix"));
    assert!(
        revised_plan
            .planning_rationale
            .as_deref()
            .unwrap()
            .contains("supersedes revision 1 because")
    );
}

#[test]
fn plan_task_blocks_on_negotiation_and_authored_brief_clarifications() {
    let workspace = std::env::temp_dir().join("boundline-runtime-routing-clarifications");
    std::fs::create_dir_all(&workspace).unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);

    let mut packet = NegotiatedDeliveryPacket::from_goal(
        "session-negotiation-context",
        &workspace.to_string_lossy(),
        "ship a big thing",
    );
    packet.resolution_state = NegotiationResolutionState::PendingClarification;
    packet.clarification_headline = Some("clarification required: narrow the request".to_string());
    packet.constraints.push(NegotiationConstraint {
        constraint_id: "constraint-1".to_string(),
        kind: NegotiationConstraintKind::Scope,
        summary: "choose one bounded outcome".to_string(),
        source: NegotiationConstraintSource::Goal,
        state: NegotiationConstraintState::Conflicting,
        blocks_planning: true,
    });

    let mut negotiation_record = ActiveSessionRecord {
        session_id: "session-negotiation-context".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("ship a big thing".to_string()),
        authored_brief: None,
        negotiation_packet: Some(packet),
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let negotiation_error = runtime.plan_task(&mut negotiation_record, None, false).unwrap_err();
    assert!(matches!(
        negotiation_error,
        SessionRuntimeError::ClarificationRequired { headline, prompt }
            if headline == "clarification required: narrow the request"
                && prompt == "choose one bounded outcome"
    ));

    let brief = AuthoredBriefBundle {
        bundle_id: "bundle-1".to_string(),
        primary_goal_text: Some("ship everything".to_string()),
        sources: vec![InputSourceReference {
            source_id: "source-1".to_string(),
            kind: InputSourceKind::DirectText,
            display_name: "goal".to_string(),
            workspace_path: None,
            precedence: 0,
            content: "ship everything".to_string(),
        }],
        deduplicated_sources: Vec::new(),
        governance_intent: None,
        resolution_state: AuthoredBriefResolutionState::ClarificationRequired,
        goal_quality: GoalQualityAssessment::default(),
        clarification: Some(ClarificationRecord {
            clarification_id: "clar-1".to_string(),
            reason_kind: ClarificationReasonKind::UnboundedRequest,
            prompt: "narrow the request to one bounded outcome".to_string(),
            missing_fields: vec!["bounded_outcome".to_string()],
            questions: vec![
                "What single bounded outcome should Boundline address first?".to_string(),
            ],
            blocking_sources: Vec::new(),
            turn_index: 0,
            status: ClarificationStatus::Open,
        }),
        derived_task_draft: None,
        captured_at: 1,
    };

    let mut authored_brief_record = ActiveSessionRecord {
        session_id: "session-authored-clarification".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some(brief.render_goal_text()),
        authored_brief: Some(brief),
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let authored_brief_error =
        runtime.plan_task(&mut authored_brief_record, None, false).unwrap_err();
    assert!(matches!(
        authored_brief_error,
        SessionRuntimeError::ClarificationRequired { headline, prompt }
            if headline.contains("clarification required")
                && prompt == "narrow the request to one bounded outcome"
    ));
}
