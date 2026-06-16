use std::fs;
use std::path::PathBuf;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::execute_run;
use boundline::domain::goal_plan::{
    GoalPlan, PlannedTask, PlanningAnalysisCoverage, PlanningAnalysisFinding,
    PlanningAnalysisProjection, PlanningAnalysisSeverity, PlanningAnalysisSource,
    PlanningAnalysisSourceRef, PlanningAnalysisState,
};
use boundline::domain::governance::{
    CanonMode, CanonModeSelectionPreference, GovernanceRuntimeKind, GovernedSessionLifecycle,
};
use boundline::domain::limits::RunLimits;
use boundline::domain::plan::Plan;
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::step::Step;
use boundline::domain::task::{Task, TaskRunRequest};
use serde_json::json;
use uuid::Uuid;

fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

fn build_request(workspace_ref: &str) -> TaskRunRequest {
    TaskRunRequest {
        goal: "Deliver a bounded change".to_string(),
        input: json!({"ticket": "PIPELINE-1"}),
        session_id: "session-pipeline".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    }
}

fn build_task(workspace_ref: &str) -> Task {
    let plan =
        Plan::new(vec![Step::decision("analyze", json!({"phase": "pipeline"})).unwrap()]).unwrap();
    Task::new("task-pipeline", &build_request(workspace_ref), plan).unwrap()
}

fn build_planned_record(workspace_ref: &str) -> ActiveSessionRecord {
    ActiveSessionRecord {
        session_id: "session-pipeline".to_string(),
        workspace_ref: workspace_ref.to_string(),
        goal: Some("Deliver a bounded change".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(build_task(workspace_ref)),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(format!("{workspace_ref}/.boundline/traces/task.json")),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    }
}

fn build_ready_goal_plan() -> Result<GoalPlan, Box<dyn std::error::Error>> {
    Ok(GoalPlan::new(
        "Deliver a bounded change",
        vec![PlannedTask {
            task_id: "T001".to_string(),
            description: "Update the bounded implementation".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("bounded change delivered".to_string()),
            decision_type_hint: None,
            depends_on: None,
        }],
    )?
    .with_planning_rationale("workspace evidence supports this bounded change")
    .with_verification_strategy("run the focused regression checks after editing"))
}

fn blocked_planning_analysis() -> PlanningAnalysisProjection {
    PlanningAnalysisProjection {
        state: PlanningAnalysisState::Blocked,
        findings: vec![PlanningAnalysisFinding {
            severity: PlanningAnalysisSeverity::Critical,
            source: PlanningAnalysisSource::Goal,
            code: "success_criterion_uncovered".to_string(),
            message: "acceptance target is not covered by the active plan".to_string(),
            source_refs: vec![PlanningAnalysisSourceRef {
                artifact_kind: "goal_plan".to_string(),
                artifact_ref: "T001".to_string(),
                anchor: Some("acceptance target".to_string()),
            }],
        }],
        coverage: Some(PlanningAnalysisCoverage {
            success_criteria_total: 1,
            success_criteria_covered: 0,
            backlog_slice_total: Some(1),
            backlog_slice_covered: Some(0),
            validation_anchor_total: None,
            validation_anchor_covered: None,
            risk_total: None,
            risk_covered: None,
            constraint_total: None,
            constraint_covered: None,
            governed_evidence_ready: false,
        }),
    }
}

fn pending_backlog_lifecycle() -> GovernedSessionLifecycle {
    GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: None,
        selected_mode_sequence: vec![
            CanonMode::Discovery,
            CanonMode::Architecture,
            CanonMode::Backlog,
        ],
        latest_reasoning_profile: None,
        current_stage_index: 2,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
        planning_input_fingerprint: None,
    }
}

#[test]
fn planning_gate_pipeline_contract_prioritizes_plan_quality()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-contract-pipeline-plan-quality");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    let mut goal_plan = build_ready_goal_plan()?.with_verification_strategy(" ");
    goal_plan.planning_analysis = Some(blocked_planning_analysis());
    record.goal_plan = Some(goal_plan);
    record.governance_lifecycle = Some(pending_backlog_lifecycle());
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert!(
        report.terminal_output.contains("current goal plan is not ready for execution"),
        "{}",
        report.terminal_output
    );
    assert_eq!(
        report.session_status.as_ref().and_then(|status| status.plan_quality_state.as_deref()),
        Some("clarification_required")
    );

    Ok(())
}

#[test]
fn planning_gate_pipeline_contract_prioritizes_backlog_quality()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-contract-pipeline-backlog-quality");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    let mut goal_plan = build_ready_goal_plan()?;
    goal_plan.planning_analysis = Some(blocked_planning_analysis());
    record.goal_plan = Some(goal_plan);
    record.governance_lifecycle = Some(pending_backlog_lifecycle());
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert!(
        report.terminal_output.contains("governed backlog packet is not ready for execution"),
        "{}",
        report.terminal_output
    );
    assert_eq!(
        report.session_status.as_ref().and_then(|status| status.backlog_quality_state.as_deref()),
        Some("clarification_required")
    );

    Ok(())
}

#[test]
fn planning_gate_pipeline_contract_blocks_execution_on_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-contract-pipeline-analysis");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    let mut goal_plan = build_ready_goal_plan()?;
    goal_plan.planning_analysis = Some(blocked_planning_analysis());
    record.goal_plan = Some(goal_plan);
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert!(
        report.terminal_output.contains("planning analysis found a blocking execution gap"),
        "{}",
        report.terminal_output
    );
    assert_eq!(
        report.session_status.as_ref().and_then(|status| status.planning_analysis_state.as_deref()),
        Some("blocked")
    );

    Ok(())
}
