use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::cli::{CommandExitStatus, workspace as cli_workspace};
use crate::domain::distribution::SUPPORTED_CANON_VERSION;
use crate::domain::governance::{
    ApprovalState, CanonCapabilitySnapshot, CanonMode, CanonModeSelectionPreference,
    GovernanceLifecycleState, GovernanceRuntimeKind, GovernedSessionLifecycle, GovernedStageRecord,
    governed_stage_catalog, validate_canon_capabilities_for_mode,
};
use crate::domain::review::{
    VotingBoundaryDecision, VotingBoundaryInput, VotingBoundaryTrigger, VotingStageRisk,
    voting_boundary_decision,
};
use crate::domain::session::VotingSessionState;

#[derive(Debug, Clone)]
pub struct GovernRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub mode: Option<CanonMode>,
    pub goal: Option<&'a str>,
    pub brief: &'a [PathBuf],
    pub base: Option<&'a str>,
    pub head: Option<&'a str>,
    pub risk: Option<&'a str>,
    pub structural_impact: bool,
    pub public_contract_change: bool,
    pub validation_exhausted: bool,
    pub pr_ready: bool,
    pub preserved_behavior_evidence: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

pub fn execute_govern(request: GovernRequest<'_>) -> Result<GovernCommandReport, GovernError> {
    let workspace = cli_workspace::resolve_workspace(request.workspace)
        .map_err(|error| GovernError::WorkspaceResolution(error.to_string()))?;

    let Some(mode) = request.mode else {
        return Ok(GovernCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: render_mode_choices(),
        });
    };

    let snapshot = current_canon_capability_snapshot();
    validate_canon_capabilities_for_mode(&snapshot, mode).map_err(GovernError::UnsupportedMode)?;

    let store = FileSessionStore::for_workspace(&workspace);
    let mut record = store.load().map_err(GovernError::SessionStore)?.ok_or_else(|| {
        GovernError::MissingSession(format!(
            ".boundline/session.json is missing; run `boundline start --workspace {}` before governed stage work",
            workspace.display()
        ))
    })?;

    if let Some(goal) = request.goal.map(str::trim).filter(|goal| !goal.is_empty()) {
        record.goal = Some(goal.to_string());
        if matches!(record.latest_status, crate::domain::session::SessionStatus::Initialized) {
            record.latest_status = crate::domain::session::SessionStatus::GoalCaptured;
        }
    }

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        && request.brief.is_empty()
        && request.base.is_none()
        && request.head.is_none()
    {
        return Err(GovernError::MissingInput(format!(
            "mode `{}` requires a captured goal, --goal, --brief, --base/--head, or equivalent authored input",
            mode.as_str()
        )));
    }

    let stage_ref = format!("govern:{}", mode.as_str());
    let packet_ref = format!("canon:{}:pending", mode.as_str());
    record.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: Some(mode),
        selected_mode_sequence: vec![mode],
        latest_reasoning_profile: None,
        current_stage_index: 0,
        stage_records: vec![GovernedStageRecord {
            stage_key: stage_ref.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::Incomplete,
            required: true,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: format!("govern-{}", Uuid::new_v4()),
            previous_governance_attempt_id: None,
            packet_ref: Some(packet_ref.clone()),
            decision_ref: None,
            blocked_reason: Some(
                "Canon execution is staged through Boundline; run the reported next command after supplying required inputs or approvals"
                    .to_string(),
            ),
        }],
        accumulated_context: Vec::new(),
        terminal_reason: None,
    });
    record.latest_voting = voting_state_for_request(mode, &request);
    record.updated_at = crate::domain::trace::current_timestamp_millis();
    store.persist(&record).map_err(GovernError::SessionStore)?;

    Ok(GovernCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!(
            concat!(
                "govern: staged\n",
                "workspace: {}\n",
                "mode: {}\n",
                "canon_capabilities: {}\n",
                "governed_stage_ref: {}\n",
                "governed_packet_ref: {}\n",
                "next_command: boundline status --workspace {}\n"
            ),
            workspace.display(),
            mode.as_str(),
            snapshot.summary_text(),
            stage_ref,
            packet_ref,
            workspace.display(),
        ),
    })
}

fn voting_state_for_request(
    mode: CanonMode,
    request: &GovernRequest<'_>,
) -> Option<VotingSessionState> {
    let stage = voting_stage_for_mode(mode)?;
    let has_voting_signal = request.risk.is_some()
        || request.structural_impact
        || request.public_contract_change
        || request.validation_exhausted
        || request.pr_ready
        || request.preserved_behavior_evidence;
    if !has_voting_signal {
        return None;
    }

    let risk = voting_risk(request.risk);
    let decision = voting_boundary_decision(VotingBoundaryInput {
        stage,
        risk,
        structural_impact: request.structural_impact,
        public_contract_change: request.public_contract_change,
        validation_exhausted: request.validation_exhausted,
        pr_ready: request.pr_ready,
        material_security_finding: mode == CanonMode::SecurityAssessment
            && matches!(risk, VotingStageRisk::High | VotingStageRisk::Critical),
        critical_supply_chain_finding: mode == CanonMode::SupplyChainAnalysis
            && matches!(risk, VotingStageRisk::Critical),
        migration_cutover: mode == CanonMode::Migration
            && matches!(risk, VotingStageRisk::High | VotingStageRisk::Critical),
        incident_high_blast_radius: mode == CanonMode::Incident
            && matches!(risk, VotingStageRisk::High | VotingStageRisk::Critical),
        preserved_behavior_evidence: request.preserved_behavior_evidence,
        explicitly_requested: false,
    });

    Some(voting_session_state(stage, decision, mode))
}

fn voting_stage_for_mode(mode: CanonMode) -> Option<VotingBoundaryTrigger> {
    match mode {
        CanonMode::Architecture => Some(VotingBoundaryTrigger::Architecture),
        CanonMode::Change => Some(VotingBoundaryTrigger::Change),
        CanonMode::Implementation => Some(VotingBoundaryTrigger::Implementation),
        CanonMode::Verification => Some(VotingBoundaryTrigger::Verification),
        CanonMode::PrReview => Some(VotingBoundaryTrigger::PrReview),
        CanonMode::Refactor => Some(VotingBoundaryTrigger::Refactor),
        CanonMode::SecurityAssessment => Some(VotingBoundaryTrigger::SecurityAssessment),
        CanonMode::SupplyChainAnalysis => Some(VotingBoundaryTrigger::SupplyChainAnalysis),
        CanonMode::Migration => Some(VotingBoundaryTrigger::Migration),
        CanonMode::Incident => Some(VotingBoundaryTrigger::Incident),
        _ => None,
    }
}

fn voting_risk(raw: Option<&str>) -> VotingStageRisk {
    match raw.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("critical") => VotingStageRisk::Critical,
        Some("high") => VotingStageRisk::High,
        Some("low") => VotingStageRisk::Low,
        _ => VotingStageRisk::Medium,
    }
}

fn voting_session_state(
    stage: VotingBoundaryTrigger,
    decision: VotingBoundaryDecision,
    mode: CanonMode,
) -> VotingSessionState {
    let trigger = decision
        .trigger
        .clone()
        .or(decision.skip_reason.clone())
        .unwrap_or_else(|| "risk_policy_not_triggered".to_string());
    let required = decision.required;
    VotingSessionState {
        trigger,
        reviewed_evidence_ref: Some(format!("govern:{}", mode.as_str())),
        result: if required { "pending".to_string() } else { "skipped".to_string() },
        reviewer_findings: Vec::new(),
        adjudication_result: None,
        blocking: decision.blocks_continuation_until_resolved,
        next_action: if required {
            "resolve_voting_boundary".to_string()
        } else {
            format!("continue_{}_stage", format!("{stage:?}").to_ascii_lowercase())
        },
    }
}

fn render_mode_choices() -> String {
    let mut lines = vec![
        "govern: mode required".to_string(),
        "summary: choose one Canon mode for a Boundline-governed stage boundary".to_string(),
        "mode_choices:".to_string(),
    ];
    lines.extend(governed_stage_catalog().iter().map(|entry| {
        format!(
            "- {} ({:?}; recommendation_only={})",
            entry.mode.as_str(),
            entry.category,
            entry.recommendation_only
        )
    }));
    lines.push("next_command: boundline govern --mode <mode> --workspace <workspace>".to_string());
    lines.join("\n")
}

fn current_canon_capability_snapshot() -> CanonCapabilitySnapshot {
    CanonCapabilitySnapshot {
        canon_version: SUPPORTED_CANON_VERSION.to_string(),
        supported_schema_versions: vec!["2026-02-01".to_string()],
        operations: vec!["capabilities".to_string(), "start".to_string(), "refresh".to_string()],
        supported_modes: governed_stage_catalog().iter().map(|entry| entry.mode).collect(),
        status_values: vec![
            "governed_ready".to_string(),
            "awaiting_approval".to_string(),
            "blocked".to_string(),
        ],
        approval_state_values: vec!["not_needed".to_string(), "requested".to_string()],
        packet_readiness_values: vec!["reusable".to_string(), "incomplete".to_string()],
        compatibility_notes: vec!["boundline-stage-boundary-routing".to_string()],
    }
}

#[derive(Debug, Error)]
pub enum GovernError {
    #[error("workspace resolution failed: {0}")]
    WorkspaceResolution(String),
    #[error("{0}")]
    MissingSession(String),
    #[error("{0}")]
    MissingInput(String),
    #[error("{0}")]
    UnsupportedMode(String),
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use crate::cli::session;
    use crate::domain::session::SessionStatus;

    fn temp_workspace(label: &str) -> PathBuf {
        let workspace =
            std::env::temp_dir().join(format!("boundline-govern-{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn request_for_mode<'a>(
        workspace: Option<&'a Path>,
        mode: CanonMode,
        goal: Option<&'a str>,
        risk: Option<&'a str>,
    ) -> GovernRequest<'a> {
        GovernRequest {
            workspace,
            mode: Some(mode),
            goal,
            brief: &[],
            base: None,
            head: None,
            risk,
            structural_impact: false,
            public_contract_change: false,
            validation_exhausted: false,
            pr_ready: false,
            preserved_behavior_evidence: false,
        }
    }

    #[test]
    fn execute_govern_requires_authored_input_when_session_goal_is_empty() {
        let workspace = temp_workspace("missing-input");
        session::execute_start_with_target(Some(workspace.as_path()), None).unwrap();

        let error = execute_govern(request_for_mode(
            Some(workspace.as_path()),
            CanonMode::Architecture,
            None,
            None,
        ))
        .unwrap_err();

        assert!(matches!(error, GovernError::MissingInput(_)), "{error}");
        assert!(error.to_string().contains("requires a captured goal"), "{error}");
    }

    #[test]
    fn execute_govern_updates_initialized_session_with_trimmed_goal_and_vote_state() {
        let workspace = temp_workspace("goal-persist");
        session::execute_start_with_target(Some(workspace.as_path()), None).unwrap();

        let mut request = request_for_mode(
            Some(workspace.as_path()),
            CanonMode::Architecture,
            Some("  Choose onboarding architecture  "),
            Some("high"),
        );
        request.structural_impact = true;

        let report = execute_govern(request).unwrap();
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("govern: staged"), "{}", report.terminal_output);

        let store = FileSessionStore::for_workspace(&workspace);
        let record = store.load().unwrap().unwrap();
        assert_eq!(record.goal.as_deref(), Some("Choose onboarding architecture"));
        assert_eq!(record.latest_status, SessionStatus::GoalCaptured);

        let vote = record.latest_voting.expect("voting state should be persisted");
        assert_eq!(vote.trigger, "high_impact_architecture");
        assert_eq!(vote.result, "pending");
        assert!(vote.blocking);
    }

    #[test]
    fn voting_state_tracks_mode_specific_risk_triggers_and_low_risk_skip() {
        for (mode, risk, expected_trigger) in [
            (CanonMode::SecurityAssessment, Some("high"), "material_security_finding"),
            (CanonMode::SupplyChainAnalysis, Some("critical"), "critical_supply_chain_finding"),
            (CanonMode::Migration, Some("high"), "migration_cutover"),
            (CanonMode::Incident, Some("high"), "incident_high_blast_radius"),
        ] {
            let vote =
                voting_state_for_request(mode, &request_for_mode(None, mode, Some("goal"), risk))
                    .expect("voting state should be created for risky governed modes");
            assert_eq!(vote.trigger, expected_trigger);
            assert_eq!(vote.result, "pending");
            assert!(vote.blocking);
        }

        let mut refactor_request =
            request_for_mode(None, CanonMode::Refactor, Some("goal"), Some("low"));
        refactor_request.preserved_behavior_evidence = true;
        let skipped = voting_state_for_request(CanonMode::Refactor, &refactor_request)
            .expect("refactor request should still project a voting state");
        assert_eq!(skipped.trigger, "low_risk_preserved_behavior");
        assert_eq!(skipped.result, "skipped");
        assert!(!skipped.blocking);
    }

    #[test]
    fn voting_stage_for_mode_maps_delivery_and_operational_modes() {
        assert_eq!(voting_stage_for_mode(CanonMode::Change), Some(VotingBoundaryTrigger::Change));
        assert_eq!(
            voting_stage_for_mode(CanonMode::Verification),
            Some(VotingBoundaryTrigger::Verification)
        );
        assert_eq!(
            voting_stage_for_mode(CanonMode::Incident),
            Some(VotingBoundaryTrigger::Incident)
        );
        assert_eq!(voting_stage_for_mode(CanonMode::Backlog), None);
    }
}
