use super::*;

impl SessionRuntime {
    // Builds a compatibility task when fixture execution remains the
    // authoritative runtime for the chosen flow.
    pub(super) fn plan_compatibility_task(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        if let Some(active_flow) = &session.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

        let request = build_task_request(
            &self.workspace_ref,
            &goal,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan =
            build_fixture_plan_for_goal(&self.workspace_ref, session.active_flow.as_ref(), &goal)
                .map_err(SessionRuntimeError::FixtureRuntime)?;
        let task = Task::new(Uuid::new_v4().to_string(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;

        session.goal_plan = None;
        session.active_task = Some(task);
        session.decisions.clear();
        session.active_flow_policy = session
            .active_flow
            .as_ref()
            .and_then(|flow| FlowPolicy::from_builtin(&flow.flow_name).ok());
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    // Builds or refreshes the native goal plan, preserving partial planning
    // state when bounded context is still insufficient.
    pub(super) fn plan_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
        requested_flow: Option<&str>,
        no_flow: bool,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let project_scale_state = project_scale_state_for_goal(&goal, "confirm_project_scale_path");
        if !no_flow
            && requested_flow.is_none()
            && let Some(active_flow) = &session.active_flow
        {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }
        if let Some(flow_name) = requested_flow {
            built_in_flow(flow_name).ok_or_else(|| SessionRuntimeError::UnknownFlow {
                requested: flow_name.to_string(),
                supported: supported_flow_names_csv(),
            })?;
        }

        if let Some(packet) = self.session_negotiation_packet(session, &goal)
            && packet.resolution_state == NegotiationResolutionState::PendingClarification
        {
            session.active_task = None;
            session.goal_plan = None;
            session.project_scale = project_scale_state.clone();
            session.decisions.clear();
            session.latest_status = SessionStatus::GoalCaptured;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();
            let prompt = packet
                .constraints
                .iter()
                .find(|constraint| constraint.blocks_planning)
                .map(|constraint| constraint.summary.clone())
                .unwrap_or_else(|| {
                    "resolve the blocking clarification before planning can continue".to_string()
                });
            return Err(SessionRuntimeError::ClarificationRequired {
                headline: packet
                    .clarification_headline
                    .unwrap_or_else(|| "clarification required before planning".to_string()),
                prompt,
            });
        }

        if let Some(authored_brief) = session.authored_brief.as_ref()
            && authored_brief.clarification.is_some()
        {
            session.active_task = None;
            session.goal_plan = None;
            session.project_scale = project_scale_state.clone();
            session.decisions.clear();
            session.latest_status = SessionStatus::GoalCaptured;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();
            return Err(SessionRuntimeError::ClarificationRequired {
                headline: authored_brief
                    .clarification_headline()
                    .unwrap_or_else(|| "bounded context required before planning".to_string()),
                prompt: authored_brief.clarification_prompt().unwrap_or_else(|| {
                    "capture a narrower goal before planning can continue".to_string()
                }),
            });
        }

        let context_sources = self.planning_context_sources(session, &goal);
        let native_flow_state = if no_flow {
            None
        } else if let Some(flow_name) = requested_flow {
            built_in_flow(flow_name).map(|flow| flow.initial_state())
        } else {
            session.active_flow.clone()
        };
        let preserved_flow_policy =
            if native_flow_state.is_some() { session.active_flow_policy.clone() } else { None };
        let preferred_flow = native_flow_state.as_ref().map(|flow| flow.flow_name.as_str());
        let mut goal_plan = match build_goal_plan_with_sources(
            &goal,
            &self.workspace_ref,
            &context_sources,
            preferred_flow,
        ) {
            Ok(goal_plan) => goal_plan,
            Err(GoalPlannerError::MissingGoal) => return Err(SessionRuntimeError::MissingGoal),
            Err(GoalPlannerError::InsufficientContext { summary, goal_plan }) => {
                let mut goal_plan = *goal_plan;
                self.apply_negotiation_projection(session, &goal, &mut goal_plan);
                if no_flow {
                    goal_plan.mark_flow_skipped();
                }

                session.active_flow = native_flow_state.clone();
                session.active_task = None;
                session.goal_plan = Some(goal_plan);
                session.project_scale = project_scale_state_for_goal(&goal, "repair_context");
                session.decisions.clear();
                session.active_flow_policy = preserved_flow_policy.clone();
                session.latest_status = SessionStatus::Blocked;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = None;
                session.updated_at = current_timestamp_millis();

                return Err(SessionRuntimeError::ClarificationRequired {
                    headline: "bounded context required before planning".to_string(),
                    prompt: summary,
                });
            }
            Err(GoalPlannerError::PlanCreation(error)) => {
                return Err(SessionRuntimeError::GoalPlan(error.to_string()));
            }
        };
        if session.authored_brief.is_none()
            && plain_goal_requires_planning_clarification(&goal, &context_sources)
        {
            self.apply_negotiation_projection(session, &goal, &mut goal_plan);
            if no_flow {
                goal_plan.mark_flow_skipped();
            }

            session.active_flow = native_flow_state.clone();
            session.active_task = None;
            session.goal_plan = Some(goal_plan);
            session.project_scale = project_scale_state_for_goal(&goal, "repair_context");
            session.decisions.clear();
            session.active_flow_policy = preserved_flow_policy.clone();
            session.latest_status = SessionStatus::Blocked;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();

            return Err(SessionRuntimeError::ClarificationRequired {
                headline: "bounded context required before planning".to_string(),
                prompt: plain_goal_planning_clarification_prompt(),
            });
        }

        if let Some(previous_goal_plan) = session.goal_plan.as_ref() {
            let previous_revision = previous_goal_plan.proposal_revision;
            goal_plan.proposal_revision = previous_goal_plan.next_revision();
            goal_plan.planning_rationale = Some(match goal_plan.planning_rationale.take() {
                Some(rationale) => format!(
                    "{rationale}; supersedes revision {previous_revision} because workspace evidence changed or the operator requested a fresh plan"
                ),
                None => format!(
                    "supersedes revision {previous_revision} because workspace evidence changed or the operator requested a fresh plan"
                ),
            });
        }

        let planned_governed_flow_name = if no_flow {
            None
        } else {
            native_flow_state
                .as_ref()
                .map(|flow| flow.flow_name.clone())
                .or_else(|| goal_plan.flow.as_ref().map(|flow| flow.flow_name.clone()))
        };

        self.apply_negotiation_projection(session, &goal, &mut goal_plan);
        if no_flow {
            goal_plan.mark_flow_skipped();
        }
        let plan_quality = goal_plan.plan_quality_assessment();
        if !matches!(plan_quality.state, PlanQualityState::Ready) {
            let (headline, prompt) = Self::plan_quality_gate_details(&goal_plan, &plan_quality);

            session.active_flow = native_flow_state.clone();
            session.active_task = None;
            session.goal_plan = Some(goal_plan);
            session.project_scale = project_scale_state_for_goal(&goal, "repair_context");
            session.decisions.clear();
            session.active_flow_policy = preserved_flow_policy.clone();
            session.latest_status = SessionStatus::Blocked;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();

            return Err(SessionRuntimeError::ClarificationRequired { headline, prompt });
        }
        if requested_flow.is_some() || session.active_flow.is_some() || no_flow {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
        }

        self.ensure_workspace_governance_lifecycle(session);
        let planning_fingerprint = compute_planning_input_fingerprint(&goal, session);
        self.reset_planning_governance_state(session, &planning_fingerprint);
        self.sync_governed_planning_sequence(session, planned_governed_flow_name.as_deref());
        let planning_requests =
            self.prepare_planning_governance_requests(session, &goal_plan, &context_sources)?;
        self.execute_planning_governance_requests(
            session,
            &mut goal_plan,
            planning_requests,
            &context_sources,
        )?;
        if let Some(lifecycle) = session.governance_lifecycle.as_mut() {
            lifecycle.planning_input_fingerprint = Some(planning_fingerprint);
        }
        let planning_blocked = self.unresolved_planning_governance_record(session).is_some();
        goal_plan.planning_analysis = if planning_blocked {
            None
        } else {
            self.planning_analysis_projection(session, &goal_plan)
        };
        let planning_analysis_blocked = goal_plan
            .planning_analysis
            .as_ref()
            .is_some_and(|projection| matches!(projection.state, PlanningAnalysisState::Blocked));

        session.active_flow = native_flow_state;
        session.active_task = None;
        session.goal_plan = Some(goal_plan);
        session.project_scale = project_scale_state;
        session.decisions.clear();
        session.active_flow_policy = preserved_flow_policy;
        session.latest_status = if planning_blocked || planning_analysis_blocked {
            SessionStatus::Blocked
        } else {
            SessionStatus::Planned
        };
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    fn plan_quality_gate_details(
        goal_plan: &GoalPlan,
        assessment: &PlanQualityAssessment,
    ) -> (String, String) {
        if matches!(assessment.state, PlanQualityState::Blocked) {
            if let Some(context_pack) = goal_plan.context_pack.as_ref() {
                match context_pack.credibility {
                    ContextPackCredibility::Insufficient => {
                        return (
                            PLAN_QUALITY_BLOCKED_HEADLINE.to_string(),
                            context_pack.summary.clone(),
                        );
                    }
                    ContextPackCredibility::Stale => {
                        return (
                            PLAN_QUALITY_BLOCKED_HEADLINE.to_string(),
                            context_pack
                                .staleness_reason
                                .clone()
                                .unwrap_or_else(|| context_pack.summary.clone()),
                        );
                    }
                    ContextPackCredibility::Credible => {}
                }
            }

            return (
                PLAN_QUALITY_BLOCKED_HEADLINE.to_string(),
                PLAN_QUALITY_BLOCKED_DEFAULT_PROMPT.to_string(),
            );
        }

        if assessment.findings.is_empty() {
            return (
                PLAN_QUALITY_CLARIFICATION_HEADLINE.to_string(),
                PLAN_QUALITY_CLARIFICATION_DEFAULT_PROMPT.to_string(),
            );
        }

        (
            PLAN_QUALITY_CLARIFICATION_HEADLINE.to_string(),
            format!("{PLAN_QUALITY_CLARIFICATION_PROMPT_PREFIX}{}", assessment.findings.join(", ")),
        )
    }

    fn planning_analysis_projection(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
    ) -> Option<PlanningAnalysisProjection> {
        let backlog_snapshot = session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            backlog_quality_snapshot_for_lifecycle(lifecycle, &self.workspace_ref)
        });

        if let Some(snapshot) = backlog_snapshot {
            if !matches!(snapshot.assessment.state, BacklogQualityState::Ready) {
                return None;
            }

            return Some(
                goal_plan
                    .planning_analysis_projection(&snapshot.assessment, &snapshot.document_bodies),
            );
        }

        Some(
            goal_plan.planning_analysis_projection(
                &Self::default_planning_analysis_backlog_quality(),
                &[],
            ),
        )
    }

    fn default_planning_analysis_backlog_quality() -> BacklogQualityAssessment {
        BacklogQualityAssessment {
            state: BacklogQualityState::Ready,
            findings: Vec::new(),
            task_count: None,
            mvp_scope: None,
            unmapped_items: Vec::new(),
        }
    }

    pub(super) fn ensure_workspace_governance_lifecycle(&self, session: &mut ActiveSessionRecord) {
        if session.governance_lifecycle.is_some() {
            return;
        }

        let Some(governance_runtime) = self.resolve_workspace_governance_runtime(session) else {
            return;
        };

        session.governance_lifecycle = Some(GovernedSessionLifecycle {
            governance_runtime,
            explicit_opt_out: governance_runtime == GovernanceRuntimeKind::Local,
            mode_selection_preference: self.resolve_workspace_mode_selection_preference(),
            selected_mode: session
                .authored_brief
                .as_ref()
                .and_then(|bundle| bundle.governance_intent.as_ref())
                .and_then(|intent| intent.explicit_mode),
            selected_mode_sequence: Vec::new(),
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: Vec::new(),
            accumulated_context: Vec::new(),
            terminal_reason: None,
            planning_input_fingerprint: None,
        });
    }

    fn reset_planning_governance_state(
        &self,
        session: &mut ActiveSessionRecord,
        new_fingerprint: &str,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        if lifecycle.planning_input_fingerprint.as_deref() == Some(new_fingerprint) {
            let all_planning_stages_clear = lifecycle
                .stage_records
                .iter()
                .filter(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_some())
                .all(|record| {
                    matches!(
                        record.lifecycle_state,
                        GovernanceLifecycleState::GovernedReady
                            | GovernanceLifecycleState::Completed
                    )
                });
            if all_planning_stages_clear {
                return;
            }
            for record in lifecycle.stage_records.iter_mut() {
                if planning_canon_mode_for_stage_key(&record.stage_key).is_some()
                    && !matches!(
                        record.lifecycle_state,
                        GovernanceLifecycleState::GovernedReady
                            | GovernanceLifecycleState::Completed
                    )
                {
                    record.lifecycle_state = GovernanceLifecycleState::PendingSelection;
                    record.blocked_reason = None;
                }
            }
            lifecycle.current_stage_index = 0;
            lifecycle.terminal_reason = None;
            return;
        }

        lifecycle
            .stage_records
            .retain(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_none());
        lifecycle
            .accumulated_context
            .retain(|reference| planning_canon_mode_for_stage_key(&reference.stage_key).is_none());
        lifecycle.current_stage_index = 0;
        lifecycle.terminal_reason = None;
    }

    fn resolve_workspace_governance_runtime(
        &self,
        session: &ActiveSessionRecord,
    ) -> Option<GovernanceRuntimeKind> {
        if let Some(governance_intent) =
            session.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.as_ref())
        {
            if governance_intent.explicit_no_canon {
                return Some(GovernanceRuntimeKind::Local);
            }
            if let Some(runtime_preference) = governance_intent.runtime_preference {
                return Some(runtime_preference);
            }
        }

        let local_config =
            FileConfigStore::for_workspace(&self.workspace_ref).load_local().ok().flatten();
        let global_config = FileConfigStore::load_global().ok().flatten();

        load_workspace_execution_profile(&self.workspace_ref)
            .ok()
            .and_then(|profile| profile.governance.map(|governance| governance.default_runtime))
            .or_else(|| {
                (local_config.as_ref().and_then(|config| config.canon.as_ref()).is_some()
                    || global_config.as_ref().and_then(|config| config.canon.as_ref()).is_some())
                .then_some(GovernanceRuntimeKind::Canon)
            })
    }

    fn resolve_workspace_mode_selection_preference(&self) -> CanonModeSelectionPreference {
        let local_config =
            FileConfigStore::for_workspace(&self.workspace_ref).load_local().ok().flatten();
        let global_config = FileConfigStore::load_global().ok().flatten();

        local_config
            .and_then(|config| config.canon.map(|canon| canon.mode_selection))
            .or_else(|| {
                global_config.and_then(|config| config.canon.map(|canon| canon.mode_selection))
            })
            .unwrap_or_default()
    }

    fn sync_governed_planning_sequence(
        &self,
        session: &mut ActiveSessionRecord,
        flow_name: Option<&str>,
    ) {
        let Some(flow_name) = flow_name else {
            return;
        };
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };
        if lifecycle.governance_runtime != GovernanceRuntimeKind::Canon
            || lifecycle.explicit_opt_out
            || !lifecycle.selected_mode_sequence.is_empty()
        {
            return;
        }

        let planned_sequence = planned_canon_mode_sequence_for_flow(flow_name);
        if planned_sequence.is_empty() {
            return;
        }

        if lifecycle.selected_mode.is_none() {
            lifecycle.selected_mode = planned_sequence.first().copied();
        }
        lifecycle.selected_mode_sequence = planned_sequence;
    }

    pub(super) fn planning_governance_read_targets(
        &self,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
    ) -> Vec<String> {
        let mut read_targets = Vec::new();
        let mut seen = BTreeSet::new();

        for target in goal_plan
            .context_pack
            .as_ref()
            .map(|context_pack| context_pack.selected_targets.as_slice())
            .unwrap_or_default()
        {
            if seen.insert(target.clone()) {
                read_targets.push(target.clone());
            }
        }
        for target in &context_sources.execution_profile_read_targets {
            if seen.insert(target.clone()) {
                read_targets.push(target.clone());
            }
        }
        for target in &context_sources.latest_changed_files {
            if seen.insert(target.clone()) {
                read_targets.push(target.clone());
            }
        }

        read_targets
    }

    pub(super) fn materialize_planning_stage_brief(
        &self,
        stage_key: &str,
        mode: CanonMode,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Result<String, SessionRuntimeError> {
        let stage_brief_ref = planning_stage_brief_ref(stage_key).ok_or_else(|| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to resolve planning stage brief path for {stage_key}"
            ))
        })?;
        let stage_brief_path = self.workspace_ref.join(&stage_brief_ref);
        let stage_directory = stage_brief_path.parent().ok_or_else(|| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to resolve planning stage brief directory for {stage_key}"
            ))
        })?;
        fs::create_dir_all(stage_directory).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to create planning governance directory for {stage_key}: {error}"
            ))
        })?;

        let mut brief_content =
            render_planning_stage_brief(stage_key, mode, goal_plan, context_sources);

        if let Some(upstream_section) =
            self.render_upstream_evidence_for_mode(mode, accumulated_context)
        {
            brief_content.push_str(&upstream_section);
        }

        fs::write(&stage_brief_path, &brief_content).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to write planning stage brief for {stage_key}: {error}"
            ))
        })?;

        Ok(stage_brief_ref)
    }

    fn render_upstream_evidence_for_mode(
        &self,
        mode: CanonMode,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Option<String> {
        match mode {
            CanonMode::Architecture => {
                self.render_architecture_upstream_evidence(accumulated_context)
            }
            CanonMode::Backlog => self.render_backlog_upstream_evidence(accumulated_context),
            _ => None,
        }
    }

    fn render_architecture_upstream_evidence(
        &self,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Option<String> {
        let system_shaping_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::SystemShaping);
        let requirements_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::Requirements);

        let mut section = String::new();
        let mut has_content = false;

        if let Some(doc_ref) = system_shaping_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_SYSTEM_SHAPE_FILE)
            {
                section.push_str("\n\n## Boundaries\n\n### System Context\n\n");
                section.push_str(&content);
                has_content = true;
            }
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_DOMAIN_MODEL_FILE)
            {
                section.push_str("\n\n### Domain Model\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if let Some(doc_ref) = requirements_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_CONSTRAINTS_FILE)
            {
                section.push_str("\n\n### Constraints\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if has_content { Some(section) } else { None }
    }

    fn render_backlog_upstream_evidence(
        &self,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Option<String> {
        let architecture_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::Architecture);
        let requirements_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::Requirements);

        let mut section = String::new();
        let mut has_content = false;

        if let Some(doc_ref) = architecture_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_ARCHITECTURE_DECISIONS_FILE)
            {
                section.push_str("\n\n## Planning Scope\n\n### Architecture Decisions\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if let Some(doc_ref) = requirements_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) = read_upstream_artifact_capped(&packet_dir, UPSTREAM_PRD_FILE) {
                let heading = if has_content {
                    "\n\n### Product Scope\n\n"
                } else {
                    "\n\n## Planning Scope\n\n### Product Scope\n\n"
                };
                section.push_str(heading);
                section.push_str(&content);
                has_content = true;
            }
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_SCOPE_CUTS_FILE)
            {
                section.push_str("\n\n### Scope Cuts\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if has_content { Some(section) } else { None }
    }

    pub(super) fn execute_discovery_stage_council(
        &self,
        request: &StageCouncilRequest,
    ) -> Result<StageCouncilOutcome, SessionRuntimeError> {
        let current_artifact_ref = request.current_artifact_ref.as_ref().ok_or_else(|| {
            SessionRuntimeError::ExecutionInvariant(
                "stage council requires current_artifact_ref for discovery planning".to_string(),
            )
        })?;
        let current_artifact_path = self.workspace_ref.join(current_artifact_ref);
        let current_artifact = fs::read_to_string(&current_artifact_path).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to read discovery stage artifact {}: {error}",
                current_artifact_path.display()
            ))
        })?;
        let producer_ref =
            self.write_stage_council_artifact(request, "producer", &current_artifact)?;
        let producer_output = StageCouncilArtifact {
            route_slot: request.producer_slot.clone(),
            evidence_ref: producer_ref.clone(),
            summary: Some("planner produced the discovery artifact for council review".to_string()),
        };
        let routing = self.planning_council_effective_routing();
        let reviewer_routes = discovery_stage_council_reviewers(&routing);
        let reviewers =
            reviewer_routes.iter().map(|route| route.reviewer.clone()).collect::<Vec<_>>();
        let participants = reviewer_routes
            .iter()
            .map(|route| {
                let available = route_is_available(&route.route);
                ReviewerParticipation {
                    reviewer_id: route.reviewer.reviewer_id.clone(),
                    status: if available {
                        ReviewerParticipationStatus::Completed
                    } else {
                        ReviewerParticipationStatus::Omitted
                    },
                    reason: (!available).then(|| {
                        format!(
                            "route {} is unavailable for provider-backed council review",
                            route
                                .reviewer
                                .source
                                .clone()
                                .unwrap_or_else(|| model_route_label(&route.route))
                        )
                    }),
                    effective_route: route.reviewer.source.clone(),
                }
            })
            .collect::<Vec<_>>();

        if let Err(error) =
            resolve_council_assembly(CouncilProfile::YellowPair, &reviewers, &participants)
        {
            return self.stage_council_blocked_outcome(
                request,
                &producer_output,
                &error.to_string(),
                "configure distinct provider-backed reviewer routes before rerunning boundline plan",
            );
        }

        let artifact_file = ProviderWorkspaceFile {
            path: producer_ref.clone(),
            contents: current_artifact.clone(),
        };
        let prior_context = json!({
            "stage_key": request.stage_key,
            "target_refs": request.target_refs,
            "constraints": request.constraints,
            "current_artifact_ref": current_artifact_ref,
        });
        let mut effective_routes = BTreeMap::new();
        let mut review_findings = Vec::new();
        let mut stage_findings = Vec::new();

        for reviewer_route in &reviewer_routes {
            let effective_route = reviewer_route
                .reviewer
                .source
                .clone()
                .unwrap_or_else(|| model_route_label(&reviewer_route.route));
            effective_routes
                .insert(reviewer_route.reviewer.reviewer_id.clone(), effective_route.clone());
            let response = match review_workspace(
                &reviewer_route.route,
                &ProviderReviewRequest {
                    goal: request.goal.clone(),
                    phase: request.phase.clone(),
                    reviewer_id: reviewer_route.reviewer.reviewer_id.clone(),
                    reviewer_role: reviewer_route.reviewer.role.clone(),
                    attempt_id: format!(
                        "{}-{}",
                        request.stage_key.replace(':', "-"),
                        reviewer_route.reviewer.reviewer_id
                    ),
                    files: vec![artifact_file.clone()],
                    prior_context: prior_context.clone(),
                },
            ) {
                Ok(response) => response,
                Err(error) => {
                    return self.stage_council_blocked_outcome(
                        request,
                        &producer_output,
                        &format!(
                            "reviewer {} failed: {error}",
                            reviewer_route.reviewer.reviewer_id
                        ),
                        "restore provider review availability before rerunning boundline plan",
                    );
                }
            };

            let mut finding = ReviewerFinding::new(
                reviewer_route.reviewer.reviewer_id.clone(),
                reviewer_disposition_from_provider(response.disposition),
                response.summary.clone(),
            );
            finding.details = response.details.clone();
            finding.runtime_role = Some(reviewer_route.reviewer.role.clone());
            finding.required_action = response.required_action.clone();
            finding.evidence_refs = if response.evidence_refs.is_empty() {
                vec![producer_ref.clone()]
            } else {
                response.evidence_refs.clone()
            };
            review_findings.push(finding);

            stage_findings.push(StageCouncilFinding {
                reviewer_id: reviewer_route.reviewer.reviewer_id.clone(),
                effective_route,
                disposition: stage_council_disposition_from_provider(response.disposition),
                summary: response.summary,
                accepted: false,
            });
        }

        let vote_resolution = VoteRuleDefinition::default()
            .resolve(&reviewers, &review_findings, Some(&effective_routes))
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
        let mut accepted_findings = stage_findings
            .iter()
            .filter(|finding| finding.disposition != StageCouncilFindingDisposition::Approve)
            .map(|finding| finding.reviewer_id.clone())
            .collect::<Vec<_>>();
        let mut rejected_findings = stage_findings
            .iter()
            .filter(|finding| finding.disposition == StageCouncilFindingDisposition::Approve)
            .map(|finding| finding.reviewer_id.clone())
            .collect::<Vec<_>>();
        let mut adjudication = None;
        let mut blocking = stage_findings
            .iter()
            .any(|finding| finding.disposition == StageCouncilFindingDisposition::Block)
            || vote_resolution.decision == VoteDecision::Rejected;

        if vote_resolution.decision == VoteDecision::NeedsAdjudication {
            if !route_is_available(&routing.adjudication.route) {
                return self.stage_council_blocked_outcome(
                    request,
                    &producer_output,
                    "adjudication was required but the adjudication route is unavailable",
                    "configure an adjudication route before rerunning boundline plan",
                );
            }

            let adjudication_response = match review_workspace(
                &routing.adjudication.route,
                &ProviderReviewRequest {
                    goal: request.goal.clone(),
                    phase: format!("{}-adjudication", request.phase),
                    reviewer_id: "arbiter".to_string(),
                    reviewer_role: "discovery adjudicator".to_string(),
                    attempt_id: format!("{}-arbiter", request.stage_key.replace(':', "-")),
                    files: vec![artifact_file.clone()],
                    prior_context: json!({
                        "review_findings": review_findings.clone(),
                        "stage_findings": stage_findings.clone(),
                    }),
                },
            ) {
                Ok(response) => response,
                Err(error) => {
                    return self.stage_council_blocked_outcome(
                        request,
                        &producer_output,
                        &format!("adjudication failed: {error}"),
                        "restore adjudication availability before rerunning boundline plan",
                    );
                }
            };

            adjudication = Some(StageCouncilAdjudication {
                adjudicator_route: model_route_label(&routing.adjudication.route),
                decision: provider_review_disposition_text(adjudication_response.disposition)
                    .to_string(),
                rationale: adjudication_response.summary.clone(),
            });

            match adjudication_response.disposition {
                ProviderReviewDisposition::Approve => {
                    accepted_findings.clear();
                    rejected_findings = stage_findings
                        .iter()
                        .filter(|finding| {
                            finding.disposition != StageCouncilFindingDisposition::Approve
                        })
                        .map(|finding| finding.reviewer_id.clone())
                        .collect();
                    blocking = false;
                }
                ProviderReviewDisposition::Concern => {
                    blocking = false;
                }
                ProviderReviewDisposition::Block => {
                    blocking = true;
                }
            }
        }

        for finding in &mut stage_findings {
            finding.accepted = accepted_findings.contains(&finding.reviewer_id);
        }

        let mut revised_summary = Some(
            "reviser preserved the producer artifact because no council findings were accepted"
                .to_string(),
        );
        let revised_artifact_text = if blocking {
            revised_summary = Some("stage council blocked planning discovery".to_string());
            render_stage_council_blocked_markdown(request, &stage_findings, &accepted_findings)
        } else {
            let accepted_feedback = stage_findings
                .iter()
                .filter(|finding| finding.accepted)
                .map(|finding| format!("{}: {}", finding.reviewer_id, finding.summary))
                .collect::<Vec<_>>();
            if accepted_feedback.is_empty() {
                current_artifact.clone()
            } else {
                if !route_is_available(&routing.planning.route) {
                    return self.stage_council_blocked_outcome(
                        request,
                        &producer_output,
                        "reviser route is unavailable for provider-backed council revision",
                        "configure a planning route before rerunning boundline plan",
                    );
                }
                match revise_artifact(
                    &routing.planning.route,
                    &ProviderRevisionRequest {
                        goal: request.goal.clone(),
                        phase: request.phase.clone(),
                        reviser_id: "reviser".to_string(),
                        target_refs: request.target_refs.clone(),
                        current_artifact: current_artifact.clone(),
                        accepted_feedback,
                        prior_context: json!({
                            "review_findings": stage_findings.clone(),
                            "adjudication": adjudication.clone(),
                        }),
                    },
                ) {
                    Ok(response) => {
                        revised_summary = Some(response.summary);
                        response.revised_artifact
                    }
                    Err(error) => {
                        return self.stage_council_blocked_outcome(
                            request,
                            &producer_output,
                            &format!("reviser failed: {error}"),
                            "restore revision availability before rerunning boundline plan",
                        );
                    }
                }
            }
        };

        let revised_ref =
            self.write_stage_council_artifact(request, "revised", &revised_artifact_text)?;
        let outcome = StageCouncilOutcome {
            producer_output,
            reviewer_findings: stage_findings,
            vote_resolution: StageCouncilVoteResolution {
                strategy: "bounded_majority".to_string(),
                accepted_findings,
                rejected_findings,
                independent_review: true,
            },
            adjudication,
            revised_output: StageCouncilArtifact {
                route_slot: request.producer_slot.clone(),
                evidence_ref: revised_ref,
                summary: revised_summary,
            },
            status: if blocking {
                StageCouncilStatus::Blocked
            } else {
                StageCouncilStatus::Proceed
            },
            next_action: if blocking {
                "repair discovery inputs and rerun boundline plan".to_string()
            } else {
                "continue planning discovery".to_string()
            },
        };
        outcome.validate().map_err(SessionRuntimeError::ExecutionInvariant)?;
        Ok(outcome)
    }
}
