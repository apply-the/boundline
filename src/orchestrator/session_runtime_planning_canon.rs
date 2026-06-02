use uuid::Uuid;

use crate::adapters::config_store::FileConfigStore;
use crate::adapters::governance_runtime::{GovernanceRuntime, GovernanceRuntimeResponse};
use crate::domain::governance::{CanonRuntimeConfig, planning_stage_key_for_mode};

use super::{
    ActiveSessionRecord, ApprovalState, CanonAuthorityZone, CanonCliRuntime, CanonIntendedPersona,
    CanonMode, CanonRiskClass, GoalPlan, GovernanceLifecycleState, GovernanceRequestKind,
    GovernanceRuntimeKind, GovernanceRuntimeRequest, GovernedStageRecord, PacketReadiness,
    PlanningContextSources, PreparedPlanningGovernanceRequest, ResolvedPlanningGovernanceDefaults,
    SessionRuntime, SessionRuntimeError, StageCouncilOutcome, StageCouncilStatus,
    append_governed_document_to_lifecycle, canon_workspace_scope_mismatch_reason,
    compacted_canon_memory_for_block, compacted_canon_memory_from_response,
    default_planning_system_context, discovery_stage_council_request,
    enrich_bounded_context_with_accumulated, governed_document_ref_from_response,
    load_workspace_execution_profile, missing_planning_governance_field,
    parse_planning_system_context, planning_brief_has_sufficient_content,
    planning_brief_insufficiency_reason, planning_canon_mode_for_stage_key,
    planning_canon_mode_sequence, planning_governance_input_documents,
    planning_stage_council_block_reason, runtime_command_available,
    stage_council_voting_session_state,
};

impl SessionRuntime {
    pub(super) fn prepare_planning_governance_requests(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
    ) -> Result<Vec<PreparedPlanningGovernanceRequest>, SessionRuntimeError> {
        let Some(lifecycle) = session.governance_lifecycle.as_ref() else {
            return Ok(Vec::new());
        };
        if lifecycle.governance_runtime != GovernanceRuntimeKind::Canon
            || lifecycle.explicit_opt_out
        {
            return Ok(Vec::new());
        }

        planning_canon_mode_sequence(&lifecycle.selected_mode_sequence)
            .into_iter()
            .map(|mode| {
                self.build_planning_governance_request(session, goal_plan, context_sources, mode)
            })
            .collect()
    }

    pub(super) fn execute_planning_governance_requests(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        requests: Vec<PreparedPlanningGovernanceRequest>,
        context_sources: &PlanningContextSources,
    ) -> Result<(), SessionRuntimeError> {
        if requests.is_empty() {
            return Ok(());
        }

        let canon = self.resolve_planning_canon_runtime();

        for (stage_index, prepared) in requests.into_iter().enumerate() {
            let mut request = prepared.request;
            if self.planning_stage_already_ready(session, &request.stage_key) {
                self.set_planning_stage_progress(session, stage_index + 1, None);
                continue;
            }

            if self.planning_stage_has_unresolved_gate(session, &request.stage_key) {
                self.set_planning_stage_progress(
                    session,
                    stage_index,
                    self.latest_planning_stage_reason(session),
                );
                break;
            }

            if !planning_brief_has_sufficient_content(context_sources, request.mode) {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    planning_brief_insufficiency_reason(request.mode),
                    prepared.stage_council.clone(),
                );
                break;
            }

            self.set_planning_stage_progress(session, stage_index, None);

            if let Some(outcome) = prepared.stage_council.as_ref()
                && outcome.status == StageCouncilStatus::Blocked
            {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    planning_stage_council_block_reason(&request.stage_key, outcome),
                    prepared.stage_council.clone(),
                );
                break;
            }

            let Some(canon) = canon.as_ref() else {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    missing_planning_canon_runtime_reason(),
                    prepared.stage_council.clone(),
                );
                break;
            };

            enrich_bounded_context_with_accumulated(
                &mut request.bounded_context,
                &self.planning_accumulated_context(session),
            );

            if self.record_planning_preflight_block(
                session,
                goal_plan,
                &request,
                stage_index,
                canon,
                prepared.stage_council.clone(),
            ) {
                break;
            }

            let response = CanonCliRuntime::new(canon.command.clone())
                .with_working_directory(&self.workspace_ref)
                .execute(&request)
                .map_err(|error| SessionRuntimeError::GovernanceRuntime(error.to_string()))?;

            let should_halt = self.record_planning_governance_response(
                session,
                goal_plan,
                &request,
                response,
                stage_index,
                prepared.stage_council,
            )?;
            if should_halt {
                break;
            }
        }

        Ok(())
    }

    fn record_planning_preflight_block(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        request: &GovernanceRuntimeRequest,
        stage_index: usize,
        canon: &CanonRuntimeConfig,
        stage_council: Option<StageCouncilOutcome>,
    ) -> bool {
        if let Some(outcome) = stage_council.as_ref()
            && outcome.status == StageCouncilStatus::Blocked
        {
            self.record_planning_governance_block(
                session,
                goal_plan,
                request,
                stage_index,
                planning_stage_council_block_reason(&request.stage_key, outcome),
                stage_council,
            );
            return true;
        }

        if !runtime_command_available(&canon.command) {
            self.record_planning_governance_block(
                session,
                goal_plan,
                request,
                stage_index,
                unavailable_planning_canon_command_reason(&canon.command),
                stage_council,
            );
            return true;
        }

        if let Some(reason) = canon_workspace_scope_mismatch_reason(&self.workspace_ref) {
            self.record_planning_governance_block(
                session,
                goal_plan,
                request,
                stage_index,
                reason,
                stage_council,
            );
            return true;
        }

        false
    }

    fn resolve_planning_canon_runtime(
        &self,
    ) -> Option<crate::domain::governance::CanonRuntimeConfig> {
        load_workspace_execution_profile(&self.workspace_ref)
            .ok()
            .and_then(|profile| profile.governance.and_then(|governance| governance.canon))
    }

    fn planning_stage_already_ready(&self, session: &ActiveSessionRecord, stage_key: &str) -> bool {
        self.latest_planning_stage_record(session, stage_key).is_some_and(|record| {
            matches!(
                record.lifecycle_state,
                GovernanceLifecycleState::GovernedReady | GovernanceLifecycleState::Completed
            )
        })
    }

    fn planning_stage_has_unresolved_gate(
        &self,
        session: &ActiveSessionRecord,
        stage_key: &str,
    ) -> bool {
        self.latest_planning_stage_record(session, stage_key).is_some_and(|record| {
            matches!(
                record.lifecycle_state,
                GovernanceLifecycleState::AwaitingApproval
                    | GovernanceLifecycleState::Blocked
                    | GovernanceLifecycleState::Failed
            )
        })
    }

    fn latest_planning_stage_reason(&self, session: &ActiveSessionRecord) -> Option<String> {
        session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle
                .stage_records
                .iter()
                .rev()
                .find(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_some())
                .and_then(|record| record.blocked_reason.clone())
                .or_else(|| lifecycle.terminal_reason.clone())
        })
    }

    fn planning_accumulated_context(
        &self,
        session: &ActiveSessionRecord,
    ) -> Vec<crate::domain::governance::GovernedDocumentRef> {
        session
            .governance_lifecycle
            .as_ref()
            .map(|lifecycle| lifecycle.accumulated_context.clone())
            .unwrap_or_default()
    }

    fn set_planning_stage_progress(
        &self,
        session: &mut ActiveSessionRecord,
        stage_index: usize,
        terminal_reason: Option<String>,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        lifecycle.current_stage_index = stage_index;
        lifecycle.terminal_reason = terminal_reason;
    }

    fn record_planning_governance_block(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        request: &GovernanceRuntimeRequest,
        stage_index: usize,
        reason: String,
        stage_council: Option<StageCouncilOutcome>,
    ) {
        let record = GovernedStageRecord {
            stage_key: request.stage_key.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::Blocked,
            required: true,
            autopilot_enabled: request.autopilot,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: request.governance_attempt_id.clone(),
            previous_governance_attempt_id: self
                .latest_planning_stage_record(session, &request.stage_key)
                .map(|record| record.governance_attempt_id.clone()),
            packet_ref: None,
            decision_ref: None,
            stage_council,
            blocked_reason: Some(reason.clone()),
        };

        self.upsert_planning_stage_record(session, record, stage_index, Some(reason.clone()));
        goal_plan.compacted_canon_memory = compacted_canon_memory_for_block(
            &request.stage_key,
            GovernanceRuntimeKind::Canon,
            &reason,
        );
    }

    fn record_planning_governance_response(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        request: &GovernanceRuntimeRequest,
        response: GovernanceRuntimeResponse,
        stage_index: usize,
        stage_council: Option<StageCouncilOutcome>,
    ) -> Result<bool, SessionRuntimeError> {
        let packet_rejected = response.packet.as_ref().is_some_and(|packet| {
            matches!(packet.readiness, PacketReadiness::Incomplete | PacketReadiness::Rejected)
        });
        let effective_status =
            if packet_rejected { GovernanceLifecycleState::Blocked } else { response.status };
        let blocked_reason = if packet_rejected {
            Some(
                response
                    .packet
                    .as_ref()
                    .map(|packet| {
                        let detail = if !packet.missing_sections.is_empty() {
                            format!(": missing sections {}", packet.missing_sections.join(", "))
                        } else if !response.message.trim().is_empty() {
                            format!(": {}", response.message)
                        } else {
                            String::new()
                        };
                        format!(
                            "governance packet was {:?} for planning stage {}{}",
                            packet.readiness, request.stage_key, detail
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "governance packet was rejected for planning stage {}",
                            request.stage_key
                        )
                    }),
            )
        } else {
            matches!(
                response.status,
                GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed
            )
            .then(|| response.message.clone())
        };

        let record = GovernedStageRecord {
            stage_key: request.stage_key.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: effective_status,
            required: true,
            autopilot_enabled: request.autopilot,
            approval_state: response.approval_state,
            canon_run_ref: response.run_ref.clone(),
            governance_attempt_id: request.governance_attempt_id.clone(),
            previous_governance_attempt_id: self
                .latest_planning_stage_record(session, &request.stage_key)
                .map(|record| record.governance_attempt_id.clone()),
            packet_ref: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
            decision_ref: None,
            stage_council,
            blocked_reason: blocked_reason.clone(),
        };

        self.upsert_planning_stage_record(session, record, stage_index, blocked_reason.clone());
        goal_plan.compacted_canon_memory = compacted_canon_memory_from_response(
            &request.stage_key,
            GovernanceRuntimeKind::Canon,
            &response,
        );

        if effective_status == GovernanceLifecycleState::GovernedReady
            && let Some(mode) = request.mode
        {
            let doc_ref = governed_document_ref_from_response(&request.stage_key, mode, &response);
            append_governed_document_to_lifecycle(session, doc_ref);
            self.set_planning_stage_progress(session, stage_index + 1, None);
            return Ok(false);
        }

        Ok(matches!(
            effective_status,
            GovernanceLifecycleState::AwaitingApproval
                | GovernanceLifecycleState::Blocked
                | GovernanceLifecycleState::Failed
        ))
    }

    fn latest_planning_stage_record<'a>(
        &self,
        session: &'a ActiveSessionRecord,
        stage_key: &str,
    ) -> Option<&'a GovernedStageRecord> {
        session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle.stage_records.iter().rev().find(|record| record.stage_key == stage_key)
        })
    }

    fn upsert_planning_stage_record(
        &self,
        session: &mut ActiveSessionRecord,
        record: GovernedStageRecord,
        stage_index: usize,
        terminal_reason: Option<String>,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        if let Some(existing_index) = lifecycle
            .stage_records
            .iter()
            .position(|existing| existing.stage_key == record.stage_key)
        {
            lifecycle.stage_records[existing_index] = record;
        } else {
            lifecycle.stage_records.push(record);
        }
        lifecycle.current_stage_index = stage_index;
        lifecycle.terminal_reason = terminal_reason;
    }

    fn build_planning_governance_request(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
        mode: CanonMode,
    ) -> Result<PreparedPlanningGovernanceRequest, SessionRuntimeError> {
        let stage_key = planning_stage_key_for_mode(mode).ok_or_else(|| {
            SessionRuntimeError::ExecutionInvariant(format!(
                "planning governance stage key is unavailable for Canon mode {}",
                mode.as_str()
            ))
        })?;
        let mut stage_brief_ref = self.materialize_planning_stage_brief(
            stage_key,
            mode,
            goal_plan,
            context_sources,
            &self.planning_accumulated_context(session),
        )?;
        let stage_council = if mode == CanonMode::Discovery {
            let council_request =
                discovery_stage_council_request(stage_key, &goal_plan.goal_text, &stage_brief_ref);
            let outcome = self.execute_discovery_stage_council(&council_request)?;
            session.latest_voting = Some(stage_council_voting_session_state(stage_key, &outcome));
            stage_brief_ref = outcome.revised_output.evidence_ref.clone();
            Some(outcome)
        } else {
            None
        };
        let defaults = self.resolve_planning_governance_defaults(session, mode)?;
        let input_documents = planning_governance_input_documents(
            session.authored_brief.as_ref(),
            &stage_brief_ref,
            goal_plan.compacted_canon_memory.as_ref(),
        );

        let (request_kind, run_ref, packet_ref) =
            self.resolve_planning_request_kind(session, stage_key, context_sources);

        Ok(PreparedPlanningGovernanceRequest {
            request: GovernanceRuntimeRequest {
                request_kind,
                governance_attempt_id: Uuid::new_v4().to_string(),
                stage_key: stage_key.to_string(),
                goal: goal_plan.goal_text.clone(),
                workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
                autopilot: false,
                mode: Some(mode),
                system_context: Some(defaults.system_context),
                risk: Some(defaults.risk),
                zone: Some(defaults.zone),
                owner: Some(defaults.owner),
                run_ref,
                packet_ref,
                bounded_context: crate::adapters::governance_runtime::GovernanceBoundedContext {
                    read_targets: self.planning_governance_read_targets(goal_plan, context_sources),
                    stage_brief_ref: Some(stage_brief_ref),
                    reused_packets: Vec::new(),
                },
                input_documents,
            },
            stage_council,
        })
    }

    fn resolve_planning_request_kind(
        &self,
        session: &ActiveSessionRecord,
        stage_key: &str,
        _context_sources: &PlanningContextSources,
    ) -> (GovernanceRequestKind, Option<String>, Option<String>) {
        let existing_record = session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle.stage_records.iter().find(|record| record.stage_key == stage_key)
        });

        let Some(record) = existing_record else {
            return (GovernanceRequestKind::Start, None, None);
        };
        let Some(run_ref) = record.canon_run_ref.as_ref().filter(|r| !r.is_empty()) else {
            return (GovernanceRequestKind::Start, None, None);
        };

        (GovernanceRequestKind::Refresh, Some(run_ref.clone()), record.packet_ref.clone())
    }

    fn resolve_planning_governance_defaults(
        &self,
        session: &ActiveSessionRecord,
        mode: CanonMode,
    ) -> Result<ResolvedPlanningGovernanceDefaults, SessionRuntimeError> {
        let canon_preferences = FileConfigStore::for_workspace(&self.workspace_ref)
            .load_local()
            .ok()
            .flatten()
            .and_then(|config| config.canon);
        let governance_intent =
            session.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.as_ref());

        let system_context = canon_preferences
            .as_ref()
            .and_then(|prefs| prefs.default_system_context.as_deref())
            .and_then(parse_planning_system_context)
            .unwrap_or_else(|| default_planning_system_context(mode));
        let risk = governance_intent
            .and_then(|intent| intent.risk.clone())
            .or_else(|| canon_preferences.as_ref().and_then(|prefs| prefs.default_risk.clone()))
            .map(|risk| {
                CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
            })
            .ok_or_else(|| missing_planning_governance_field(mode, "risk"))?;
        let zone = governance_intent
            .and_then(|intent| intent.zone.clone())
            .or_else(|| canon_preferences.as_ref().and_then(|prefs| prefs.default_zone.clone()))
            .map(|zone| {
                CanonAuthorityZone::canonicalize_label(&zone).map(str::to_string).unwrap_or(zone)
            })
            .ok_or_else(|| missing_planning_governance_field(mode, "zone"))?;
        let owner = governance_intent
            .and_then(|intent| intent.owner.clone())
            .or_else(|| canon_preferences.as_ref().and_then(|prefs| prefs.default_owner.clone()))
            .map(|owner| {
                CanonIntendedPersona::canonicalize_label(&owner)
                    .map(str::to_string)
                    .unwrap_or(owner)
            })
            .ok_or_else(|| missing_planning_governance_field(mode, "owner"))?;

        Ok(ResolvedPlanningGovernanceDefaults { system_context, risk, zone, owner })
    }
}

fn missing_planning_canon_runtime_reason() -> String {
    "planning governance requires Canon initialization, but .boundline/execution.json is missing governance.canon"
        .to_string()
}

fn unavailable_planning_canon_command_reason(command: &str) -> String {
    format!("planning governance requires Canon, but command '{}' is unavailable", command)
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use crate::adapters::governance_runtime::{
        GovernanceBoundedContext, GovernanceRequestKind, GovernanceRuntimeRequest,
        GovernanceRuntimeResponse,
    };
    use crate::domain::brief::{GovernanceIntent, normalize_inputs_with_governance};
    use crate::domain::configuration::{CanonPreferences, ConfigFile};
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::governance::{
        ApprovalState, CanonMode, CanonModeSelectionPreference, GovernanceLifecycleState,
        GovernanceRuntimeKind, GovernedDocumentRef, GovernedSessionLifecycle, GovernedStagePacket,
        GovernedStageRecord, PacketReadiness, SystemContextBinding,
    };
    use crate::domain::session::ActiveSessionRecord;
    use crate::orchestrator::goal_planner::PlanningContextSources;

    use super::{
        SessionRuntime, missing_planning_canon_runtime_reason,
        unavailable_planning_canon_command_reason,
    };
    use crate::orchestrator::session_runtime::SYSTEM_CONTEXT_EXISTING_TEXT;

    const BRIEF_FILE_NAME: &str = "brief.md";
    const DEFAULT_OWNER: &str = "  Platform  ";
    const DEFAULT_RISK: &str = "  Medium  ";
    const DEFAULT_ZONE: &str = "  Engineering  ";
    const GOVERNANCE_ATTEMPT_ID: &str = "attempt-1";
    const NEXT_GOVERNANCE_ATTEMPT_ID: &str = "attempt-2";
    const GOVERNED_PACKET_REF: &str = ".canon/requirements";
    const GOVERNED_STAGE_KEY: &str = "plan:requirements";
    const GOVERNED_TERMINAL_REASON: &str = "awaiting approval";
    const GOAL_TEXT: &str = "Deliver a governed feature";
    const MISSING_SECTION_CONSTRAINTS: &str = "constraints";
    const MISSING_SECTION_SCOPE: &str = "scope";
    const REJECTED_PACKET_REASON_CODE: &str = "packet_rejected";
    const RUN_REF: &str = "canon-run-1";
    const SECOND_RUN_REF: &str = "canon-run-2";
    const UNAVAILABLE_CANON_COMMAND: &str = "canon-command-that-does-not-exist";
    const UPDATED_AT: u64 = 42;

    #[test]
    fn planning_canon_helpers_cover_stage_gate_reason_and_request_kind_resolution()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-planning-canon-helpers")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());

        assert!(!runtime.planning_stage_already_ready(&session, GOVERNED_STAGE_KEY));
        assert!(!runtime.planning_stage_has_unresolved_gate(&session, GOVERNED_STAGE_KEY));
        assert_eq!(runtime.latest_planning_stage_reason(&session), None);
        assert!(runtime.planning_accumulated_context(&session).is_empty());

        session.governance_lifecycle = Some(sample_lifecycle());
        assert!(runtime.planning_stage_already_ready(&session, GOVERNED_STAGE_KEY));
        assert!(!runtime.planning_stage_has_unresolved_gate(&session, GOVERNED_STAGE_KEY));
        assert_eq!(
            runtime.latest_planning_stage_reason(&session),
            Some(GOVERNED_TERMINAL_REASON.to_string())
        );
        assert_eq!(runtime.planning_accumulated_context(&session).len(), 1);

        let blocked_record = GovernedStageRecord {
            lifecycle_state: GovernanceLifecycleState::Blocked,
            blocked_reason: Some("missing packet".to_string()),
            ..sample_stage_record()
        };
        runtime.upsert_planning_stage_record(
            &mut session,
            blocked_record,
            2,
            Some("blocked now".to_string()),
        );
        assert!(runtime.planning_stage_has_unresolved_gate(&session, GOVERNED_STAGE_KEY));
        assert_eq!(
            runtime.latest_planning_stage_reason(&session),
            Some("missing packet".to_string())
        );

        runtime.set_planning_stage_progress(&mut session, 3, Some("progressed".to_string()));
        let lifecycle =
            session.governance_lifecycle.as_ref().ok_or("missing governance lifecycle")?;
        assert_eq!(lifecycle.current_stage_index, 3);
        assert_eq!(lifecycle.terminal_reason.as_deref(), Some("progressed"));

        let existing_record = lifecycle.stage_records.first().ok_or("missing stage record")?;
        let planning_sources = PlanningContextSources::default();
        assert_eq!(
            runtime.resolve_planning_request_kind(&session, "plan:architecture", &planning_sources),
            (super::GovernanceRequestKind::Start, None, None)
        );
        assert_eq!(
            runtime.resolve_planning_request_kind(&session, GOVERNED_STAGE_KEY, &planning_sources),
            (
                super::GovernanceRequestKind::Refresh,
                Some(RUN_REF.to_string()),
                existing_record.packet_ref.clone(),
            )
        );

        assert!(missing_planning_canon_runtime_reason().contains("governance.canon"));
        assert!(unavailable_planning_canon_command_reason("canon").contains("command 'canon'"));

        Ok(())
    }

    #[test]
    fn planning_canon_helpers_cover_runtime_and_governance_default_resolution()
    -> Result<(), Box<dyn Error>> {
        let missing_runtime_workspace = temp_workspace("boundline-planning-canon-no-runtime")?;
        let missing_runtime = SessionRuntime::for_workspace(missing_runtime_workspace.as_path());
        assert!(missing_runtime.resolve_planning_canon_runtime().is_none());

        let workspace = temp_workspace("boundline-planning-canon-defaults")?;
        seed_brief(workspace.as_path())?;
        let config_store = super::FileConfigStore::for_workspace(workspace.as_path());
        config_store.save_local(&ConfigFile {
            canon: Some(CanonPreferences {
                mode_selection: CanonModeSelectionPreference::AutoConfirm,
                default_risk: Some(DEFAULT_RISK.to_string()),
                default_zone: Some(DEFAULT_ZONE.to_string()),
                default_owner: Some(DEFAULT_OWNER.to_string()),
                default_system_context: Some(SYSTEM_CONTEXT_EXISTING_TEXT.to_string()),
            }),
            ..ConfigFile::default()
        })?;

        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());
        session.authored_brief = Some(normalize_inputs_with_governance(
            workspace.as_path(),
            Some(GOAL_TEXT),
            &[PathBuf::from(BRIEF_FILE_NAME)],
            Some(GovernanceIntent {
                requested: true,
                runtime_preference: Some(GovernanceRuntimeKind::Canon),
                risk: None,
                zone: None,
                owner: None,
                explicit_mode: None,
                explicit_no_canon: false,
            }),
        )?);

        let defaults =
            runtime.resolve_planning_governance_defaults(&session, CanonMode::Requirements)?;
        assert_eq!(defaults.system_context, SystemContextBinding::Existing);
        assert_eq!(defaults.risk, "bounded-impact");
        assert_eq!(defaults.zone, "yellow");
        assert_eq!(defaults.owner, "delivery-engineer");

        let author_override_session = ActiveSessionRecord {
            authored_brief: Some(normalize_inputs_with_governance(
                workspace.as_path(),
                Some(GOAL_TEXT),
                &[PathBuf::from(BRIEF_FILE_NAME)],
                Some(GovernanceIntent {
                    requested: true,
                    runtime_preference: Some(GovernanceRuntimeKind::Canon),
                    risk: Some("High".to_string()),
                    zone: Some("Red".to_string()),
                    owner: Some("Council".to_string()),
                    explicit_mode: None,
                    explicit_no_canon: false,
                }),
            )?),
            ..sample_session(workspace.as_path())
        };
        let overridden = runtime.resolve_planning_governance_defaults(
            &author_override_session,
            CanonMode::Requirements,
        )?;
        assert_eq!(overridden.risk, "systemic-impact");
        assert_eq!(overridden.zone, "red");
        assert_eq!(overridden.owner, "Council");

        let missing_defaults_workspace =
            temp_workspace("boundline-planning-canon-missing-defaults")?;
        seed_brief(missing_defaults_workspace.as_path())?;
        let missing_defaults_runtime =
            SessionRuntime::for_workspace(missing_defaults_workspace.as_path());
        let missing_defaults_session = sample_session(missing_defaults_workspace.as_path());
        let error = missing_defaults_runtime
            .resolve_planning_governance_defaults(
                &missing_defaults_session,
                CanonMode::Requirements,
            )
            .unwrap_err();
        assert!(error.to_string().contains("requires field 'risk'"));

        Ok(())
    }

    #[test]
    fn planning_canon_helpers_cover_preflight_block_and_blocked_response_projection()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-planning-canon-blocks")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());

        let mut preflight_session = sample_session(workspace.as_path());
        preflight_session.governance_lifecycle = Some(sample_lifecycle());
        let mut preflight_goal_plan = sample_goal_plan()?;
        let request = sample_governance_request(workspace.as_path());
        let expected_reason = unavailable_planning_canon_command_reason(UNAVAILABLE_CANON_COMMAND);

        let blocked = runtime.record_planning_preflight_block(
            &mut preflight_session,
            &mut preflight_goal_plan,
            &request,
            0,
            &crate::domain::governance::CanonRuntimeConfig {
                command: UNAVAILABLE_CANON_COMMAND.to_string(),
                default_owner: None,
                default_risk: None,
                default_zone: None,
                default_system_context: None,
            },
            None,
        );
        assert!(blocked);
        assert!(runtime.planning_stage_has_unresolved_gate(&preflight_session, GOVERNED_STAGE_KEY));
        let preflight_record = preflight_session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.stage_records.first())
            .ok_or("missing preflight record")?;
        assert_eq!(preflight_record.lifecycle_state, GovernanceLifecycleState::Blocked);
        assert_eq!(
            preflight_record.previous_governance_attempt_id.as_deref(),
            Some(GOVERNANCE_ATTEMPT_ID)
        );
        assert_eq!(preflight_record.blocked_reason.as_deref(), Some(expected_reason.as_str()));
        let preflight_memory = preflight_goal_plan
            .compacted_canon_memory
            .as_ref()
            .ok_or("missing preflight canon memory")?;
        assert_eq!(preflight_memory.stage_key.as_deref(), Some(GOVERNED_STAGE_KEY));
        assert_eq!(preflight_memory.reason_code.as_deref(), Some("blocked_context"));

        let mut response_session = sample_session(workspace.as_path());
        response_session.governance_lifecycle = Some(sample_lifecycle());
        let mut response_goal_plan = sample_goal_plan()?;
        let should_halt = runtime.record_planning_governance_response(
            &mut response_session,
            &mut response_goal_plan,
            &request,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::GovernedReady,
                approval_state: ApprovalState::Requested,
                run_ref: Some(SECOND_RUN_REF.to_string()),
                packet: Some(GovernedStagePacket {
                    packet_ref: "packet-2".to_string(),
                    runtime: GovernanceRuntimeKind::Canon,
                    canon_mode: Some(CanonMode::Requirements),
                    expected_document_refs: vec!["requirements.md".to_string()],
                    document_refs: vec!["requirements.md".to_string()],
                    readiness: PacketReadiness::Rejected,
                    missing_sections: vec![
                        MISSING_SECTION_SCOPE.to_string(),
                        MISSING_SECTION_CONSTRAINTS.to_string(),
                    ],
                    headline: "Requirements packet rejected".to_string(),
                    reason_code: Some(REJECTED_PACKET_REASON_CODE.to_string()),
                    authority_governance: None,
                    adaptive_governance: None,
                    semantic_descriptor: None,
                }),
                reason_code: Some(REJECTED_PACKET_REASON_CODE.to_string()),
                message: "Needs more planning context".to_string(),
            },
            1,
            None,
        )?;
        assert!(should_halt);
        let response_record = response_session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.stage_records.first())
            .ok_or("missing response record")?;
        assert_eq!(response_record.lifecycle_state, GovernanceLifecycleState::Blocked);
        assert_eq!(response_record.canon_run_ref.as_deref(), Some(SECOND_RUN_REF));
        assert_eq!(response_record.packet_ref.as_deref(), Some("packet-2"));
        let blocked_reason =
            response_record.blocked_reason.as_deref().ok_or("missing blocked reason")?;
        assert!(blocked_reason.contains(MISSING_SECTION_SCOPE));
        assert!(blocked_reason.contains(MISSING_SECTION_CONSTRAINTS));
        let response_memory = response_goal_plan
            .compacted_canon_memory
            .as_ref()
            .ok_or("missing response canon memory")?;
        assert_eq!(response_memory.stage_key.as_deref(), Some(GOVERNED_STAGE_KEY));
        assert_eq!(response_memory.packet_ref.as_deref(), Some("packet-2"));
        assert_eq!(response_memory.reason_code.as_deref(), Some(REJECTED_PACKET_REASON_CODE));

        Ok(())
    }

    fn sample_session(workspace: &Path) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: format!("session-{}", Uuid::new_v4()),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some(GOAL_TEXT.to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: crate::domain::session::SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: UPDATED_AT,
            updated_at: UPDATED_AT,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        }
    }

    fn sample_lifecycle() -> GovernedSessionLifecycle {
        GovernedSessionLifecycle {
            governance_runtime: GovernanceRuntimeKind::Canon,
            explicit_opt_out: false,
            mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
            selected_mode: Some(CanonMode::Requirements),
            selected_mode_sequence: vec![CanonMode::Requirements],
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: vec![sample_stage_record()],
            accumulated_context: vec![GovernedDocumentRef {
                stage_key: GOVERNED_STAGE_KEY.to_string(),
                canon_mode: CanonMode::Requirements,
                packet_ref: GOVERNED_PACKET_REF.to_string(),
                document_path: Some("brief.md".to_string()),
                readiness: PacketReadiness::Reusable,
            }],
            terminal_reason: Some(GOVERNED_TERMINAL_REASON.to_string()),
            planning_input_fingerprint: None,
        }
    }

    fn sample_stage_record() -> GovernedStageRecord {
        GovernedStageRecord {
            stage_key: GOVERNED_STAGE_KEY.to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::GovernedReady,
            required: true,
            autopilot_enabled: false,
            approval_state: ApprovalState::Requested,
            canon_run_ref: Some(RUN_REF.to_string()),
            governance_attempt_id: GOVERNANCE_ATTEMPT_ID.to_string(),
            previous_governance_attempt_id: None,
            packet_ref: Some(GOVERNED_PACKET_REF.to_string()),
            decision_ref: None,
            stage_council: None,
            blocked_reason: None,
        }
    }

    fn sample_governance_request(workspace: &Path) -> GovernanceRuntimeRequest {
        GovernanceRuntimeRequest {
            request_kind: GovernanceRequestKind::Refresh,
            governance_attempt_id: NEXT_GOVERNANCE_ATTEMPT_ID.to_string(),
            stage_key: GOVERNED_STAGE_KEY.to_string(),
            goal: GOAL_TEXT.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            autopilot: false,
            mode: Some(CanonMode::Requirements),
            system_context: Some(SystemContextBinding::Existing),
            risk: Some("bounded-impact".to_string()),
            zone: Some("yellow".to_string()),
            owner: Some("delivery-engineer".to_string()),
            run_ref: Some(RUN_REF.to_string()),
            packet_ref: Some(GOVERNED_PACKET_REF.to_string()),
            bounded_context: GovernanceBoundedContext {
                read_targets: Vec::new(),
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            input_documents: Vec::new(),
        }
    }

    fn sample_goal_plan() -> Result<GoalPlan, Box<dyn Error>> {
        GoalPlan::new(
            GOAL_TEXT,
            vec![PlannedTask {
                task_id: "T001".to_string(),
                description: "Prepare governed planning context".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: None,
                decision_type_hint: None,
            }],
        )
        .map_err(Into::into)
    }

    fn seed_brief(workspace: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(workspace.join("src"))?;
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )?;
        fs::write(
            workspace.join(BRIEF_FILE_NAME),
            "Deliver the governed feature through requirements for src/lib.rs and keep the scope bounded.\n",
        )?;
        Ok(())
    }

    fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
        TestWorkspace::new(prefix)
    }

    struct TestWorkspace {
        path: PathBuf,
    }

    impl TestWorkspace {
        fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
            let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn as_path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
