use super::*;

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

            if let Some(outcome) = prepared.stage_council.clone()
                && outcome.status == StageCouncilStatus::Blocked
            {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    planning_stage_council_block_reason(&request.stage_key, &outcome),
                    Some(outcome),
                );
                break;
            }

            let Some(canon) = canon.as_ref() else {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    "planning governance requires Canon initialization, but .boundline/execution.json is missing governance.canon"
                        .to_string(),
                    prepared.stage_council.clone(),
                );
                break;
            };

            enrich_bounded_context_with_accumulated(
                &mut request.bounded_context,
                &self.planning_accumulated_context(session),
            );

            if !runtime_command_available(&canon.command) {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    format!(
                        "planning governance requires Canon, but command '{}' is unavailable",
                        canon.command
                    ),
                    prepared.stage_council.clone(),
                );
                break;
            }

            if let Some(reason) = canon_workspace_scope_mismatch_reason(&self.workspace_ref) {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    reason,
                    prepared.stage_council.clone(),
                );
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
