use super::*;

impl SessionRuntime {
    pub(super) fn run_native_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<CheckpointProjectionState>,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let Some(mut goal_plan) = session.goal_plan.clone() else {
            return Err(SessionRuntimeError::MissingGoalPlan);
        };
        self.attempt_auto_clear_provider_block(session);
        if let Some(stage_record) = self.unresolved_planning_governance_record(session) {
            return Err(SessionRuntimeError::PlanningGovernanceUnresolved {
                stage_key: stage_record.stage_key.clone(),
                state: stage_record.lifecycle_state,
                reason: stage_record.blocked_reason.clone().or_else(|| {
                    session.governance_lifecycle.as_ref().and_then(|l| l.terminal_reason.clone())
                }),
            });
        }

        if goal_plan.requires_confirmation() {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
            session.goal_plan = Some(goal_plan.clone());
        }

        if let Some(delegation) = self.goal_plan_delegation_view(&goal_plan)
            && matches!(
                delegation.mode,
                DelegationContinuityMode::HandoffRequired
                    | DelegationContinuityMode::EscalationRequired
                    | DelegationContinuityMode::Stuck
                    | DelegationContinuityMode::InspectOnly
                    | DelegationContinuityMode::Exhausted
            )
        {
            let reason = session.latest_terminal_reason.clone().unwrap_or_else(|| {
                build_terminal_reason(
                    TerminalCondition::NoCredibleNextStep,
                    delegation.headline.clone(),
                    delegation_trace_details(Some(delegation.clone())),
                )
            });
            let trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
            return self.persist_native_result(
                session,
                goal_plan,
                Vec::new(),
                trace,
                NativePersistenceInput {
                    checkpoint_projection: checkpoint_projection.clone(),
                    terminal_reason: reason,
                    limits: RunLimits::default(),
                    native_context: TaskContext::new(
                        session.session_id.clone(),
                        session.workspace_ref.clone(),
                        RunLimits::default(),
                        Map::new(),
                    ),
                    record_terminal_event: true,
                    projected_task: None,
                },
            );
        }

        if let Some((packet, continuity)) = self.native_delegation_for_goal_plan(&goal_plan) {
            goal_plan
                .record_delegation_packet(packet, continuity)
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
            let delegation = self.goal_plan_delegation_view(&goal_plan);
            let reason = build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                delegation.as_ref().map(|view| view.headline.clone()).unwrap_or_else(|| {
                    "native goal plan reached a delegated continuity boundary".to_string()
                }),
                delegation_trace_details(delegation.clone()),
            );
            let trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
            return self.persist_native_result(
                session,
                goal_plan,
                Vec::new(),
                trace,
                NativePersistenceInput {
                    checkpoint_projection: checkpoint_projection.clone(),
                    terminal_reason: reason,
                    limits: RunLimits::default(),
                    native_context: TaskContext::new(
                        session.session_id.clone(),
                        session.workspace_ref.clone(),
                        RunLimits::default(),
                        Map::new(),
                    ),
                    record_terminal_event: true,
                    projected_task: None,
                },
            );
        }

        let runtime = self.build_runtime(session)?;
        let (native_governance_task, governance_events) =
            match self.prepare_native_governance_projection(session, &runtime, &goal_plan)? {
                NativeGovernanceProjection::None => (None, Vec::new()),
                NativeGovernanceProjection::Task { task, events } => (Some(*task), events),
                NativeGovernanceProjection::Terminal { response, task } => {
                    session.active_task = Some(*task);
                    session.goal_plan = Some(goal_plan);
                    session.decisions.clear();
                    return Ok(*response);
                }
            };
        let enable_flow_retry_probe = session.active_flow.is_some()
            && runtime.profile.governance.is_none()
            && runtime.profile.legacy_source.as_deref() != Some("native_goal_plan_synthesized");
        let decision_loop = DecisionLoop::new(
            runtime.agents.clone(),
            runtime.tools.clone(),
            self.trace_store.clone(),
            runtime.profile.limits.max_steps,
        );
        let (terminal, decisions, mut trace, mut native_task_context) = decision_loop
            .run_with_options_and_context(
                &goal_plan,
                session.active_flow_policy.as_ref(),
                &session.workspace_ref,
                &session.session_id,
                enable_flow_retry_probe,
            )
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;
        let mut reason = self.native_terminal_reason(&terminal);
        self.backfill_native_execution_state(
            &runtime,
            &mut native_task_context,
            task_status_for_condition(reason.condition),
        );
        if task_status_for_condition(reason.condition) == TaskStatus::Succeeded {
            self.execute_post_implementation_governance(
                session,
                &runtime,
                &mut goal_plan,
                &decisions,
                &mut native_task_context,
                &mut trace,
            )?;
        }
        let native_review = if task_status_for_condition(reason.condition) == TaskStatus::Succeeded
        {
            let native_review = self.execute_native_review_sequence(
                session,
                &runtime,
                &goal_plan,
                &mut native_task_context,
            )?;
            if let Some(review_reason) = native_review.terminal_reason.clone() {
                reason = review_reason;
            }
            if task_status_for_condition(reason.condition) == TaskStatus::Succeeded {
                self.propagate_cluster_delivery_changes(&goal_plan, &runtime)?;
            }
            native_review
        } else {
            NativeReviewExecution::default()
        };
        if !native_review.events.is_empty() {
            self.insert_trace_events_before_terminal(&mut trace, native_review.events);
        }
        if !governance_events.is_empty() {
            self.insert_trace_events_before_terminal(&mut trace, governance_events);
        }
        let projected_task = native_governance_task.map(|task| {
            self.finalize_native_projected_task(
                task,
                task_status_for_condition(reason.condition),
                &reason,
                &native_task_context,
            )
        });

        self.persist_native_result(
            session,
            goal_plan,
            decisions,
            trace,
            NativePersistenceInput {
                checkpoint_projection,
                terminal_reason: reason,
                limits: runtime.profile.limits.clone(),
                native_context: native_task_context,
                record_terminal_event: false,
                projected_task,
            },
        )
    }

    pub(super) fn build_goal_plan_trace(
        &self,
        session_id: &str,
        goal_plan: &GoalPlan,
    ) -> ExecutionTrace {
        let mut trace = ExecutionTrace::new(
            goal_plan.plan_id.clone(),
            session_id.to_string(),
            goal_plan.goal_text.clone(),
        );
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            0,
            self.goal_plan_trace_payload(goal_plan),
        );
        trace
    }

    fn goal_plan_trace_payload(&self, goal_plan: &GoalPlan) -> Value {
        let payload = GoalPlanTracePayload::from_goal_plan(
            goal_plan,
            self.goal_plan_routing_projection(),
            self.goal_plan_delegation_view(goal_plan),
        );
        serialize_trace_payload(&payload)
    }

    fn goal_plan_routing_projection(&self) -> RoutingDecisionProjection {
        let workspace_routing =
            FileConfigStore::for_workspace(&self.workspace_ref).local_routing().ok().flatten();
        let cluster_routing = FileClusterStore::for_workspace(&self.workspace_ref)
            .load()
            .ok()
            .flatten()
            .map(|config| config.routing);
        let global_routing = FileConfigStore::global_routing().ok().flatten();
        let effective_routing = resolve_effective_routing(
            &RoutingOverrides::default(),
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_capabilities = resolve_effective_runtime_capabilities(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_effort = resolve_effective_slot_effort_policies(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );

        RoutingDecisionProjection::from_effective_state(
            &effective_routing,
            &effective_capabilities,
            &effective_effort,
        )
    }

    fn goal_plan_delegation_view(&self, goal_plan: &GoalPlan) -> Option<DelegationStatusView> {
        goal_plan.delegation_continuity().and_then(|continuity| {
            DelegationStatusView::from_continuity(continuity, goal_plan.delegation_packet_history())
                .ok()
        })
    }

    pub(super) fn native_delegation_for_goal_plan(
        &self,
        goal_plan: &GoalPlan,
    ) -> Option<(DelegationPacket, DelegationContinuityState)> {
        if !goal_plan.flow.as_ref().is_some_and(|flow| flow.confirmed) {
            return None;
        }

        let workspace_routing =
            FileConfigStore::for_workspace(&self.workspace_ref).local_routing().ok().flatten();
        let cluster_routing = FileClusterStore::for_workspace(&self.workspace_ref)
            .load()
            .ok()
            .flatten()
            .map(|config| config.routing);
        let global_routing = FileConfigStore::global_routing().ok().flatten();
        let effective_routing = resolve_effective_routing(
            &RoutingOverrides::default(),
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_capabilities = resolve_effective_runtime_capabilities(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_effort = resolve_effective_slot_effort_policies(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );

        let implementation_runtime = effective_routing.implementation.route.runtime;
        let assistant_runtimes = effective_assistant_runtimes(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let assistant_runtime_mismatch =
            !assistant_runtimes.is_empty() && !assistant_runtimes.contains(&implementation_runtime);
        let implementation_capability = effective_capabilities.get(&implementation_runtime);
        let implementation_effort = effective_effort.get(&RouteSlot::Implementation);
        let requires_preserved_capability_handoff = implementation_capability
            .is_some_and(|capability| !capability.profile.continuation.is_supported())
            && implementation_effort
                .is_some_and(|effort| effort.policy.fallback == EffortFallbackPolicy::Preserve);

        if !requires_preserved_capability_handoff {
            if assistant_runtime_mismatch {
                let available_runtimes = assistant_runtimes
                    .iter()
                    .map(|runtime| runtime.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let evidence_summary = format!(
                    "implementation route requires {}, but available assistant runtimes are: {}",
                    implementation_runtime.as_str(),
                    available_runtimes
                );
                let packet = DelegationPacket {
                    packet_id: Uuid::new_v4().to_string(),
                    kind: DelegationPacketKind::Escalation,
                    state: DelegationPacketState::Active,
                    created_at: current_timestamp_millis(),
                    resolved_at: None,
                    source_route_owner: implementation_runtime.as_str().to_string(),
                    target_owner: "operator".to_string(),
                    continuity_reason: evidence_summary.clone(),
                    recommended_next_action: "boundline inspect".to_string(),
                    evidence_refs: Vec::new(),
                    capability_summary: Some(evidence_summary.clone()),
                    stuck_marker: None,
                    superseded_by_packet_id: None,
                };
                let continuity = DelegationContinuityState {
                    active_packet_id: Some(packet.packet_id.clone()),
                    mode: DelegationContinuityMode::EscalationRequired,
                    authority_source: ContinuityAuthority::NativeSession,
                    next_command: "boundline inspect".to_string(),
                    headline: packet.headline(),
                    evidence_summary: packet.evidence_summary(),
                };
                return Some((packet, continuity));
            }
            return None;
        }

        let implementation_capability = implementation_capability?;

        let evidence_summary = format!(
            "{} lacks continuation support for implementation",
            implementation_runtime.as_str()
        );

        if let Some(target_runtime) = assistant_runtimes.into_iter().find(|runtime| {
            effective_capabilities.get(runtime).is_some_and(|capability| {
                capability.profile.continuation.is_supported()
                    && capability.profile.handoff_target.is_supported()
            })
        }) {
            let packet = DelegationPacket {
                packet_id: Uuid::new_v4().to_string(),
                kind: DelegationPacketKind::Handoff,
                state: DelegationPacketState::Active,
                created_at: current_timestamp_millis(),
                resolved_at: None,
                source_route_owner: implementation_runtime.as_str().to_string(),
                target_owner: target_runtime.as_str().to_string(),
                continuity_reason: "implementation route cannot continue".to_string(),
                recommended_next_action: "boundline status".to_string(),
                evidence_refs: Vec::new(),
                capability_summary: Some(evidence_summary.clone()),
                stuck_marker: None,
                superseded_by_packet_id: None,
            };
            let continuity = DelegationContinuityState {
                active_packet_id: Some(packet.packet_id.clone()),
                mode: DelegationContinuityMode::HandoffRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline status".to_string(),
                headline: packet.headline(),
                evidence_summary: packet.evidence_summary(),
            };
            return Some((packet, continuity));
        }

        if implementation_capability.profile.escalation_context.is_supported() {
            let packet = DelegationPacket {
                packet_id: Uuid::new_v4().to_string(),
                kind: DelegationPacketKind::Escalation,
                state: DelegationPacketState::Active,
                created_at: current_timestamp_millis(),
                resolved_at: None,
                source_route_owner: implementation_runtime.as_str().to_string(),
                target_owner: "operator".to_string(),
                continuity_reason: "implementation route cannot continue".to_string(),
                recommended_next_action: "boundline inspect".to_string(),
                evidence_refs: Vec::new(),
                capability_summary: Some(evidence_summary),
                stuck_marker: None,
                superseded_by_packet_id: None,
            };
            let continuity = DelegationContinuityState {
                active_packet_id: Some(packet.packet_id.clone()),
                mode: DelegationContinuityMode::EscalationRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: packet.headline(),
                evidence_summary: packet.evidence_summary(),
            };
            return Some((packet, continuity));
        }

        None
    }

    pub(super) fn build_cluster_delivery_story(
        &self,
        projection: &ClusterSessionProjection,
        terminal_status: TaskStatus,
    ) -> ClusterDeliveryStory {
        let blocking_workspace_ref = projection
            .member_workspace_refs
            .iter()
            .filter(|workspace_ref| *workspace_ref != &projection.primary_workspace_ref)
            .find(|workspace_ref| cluster_workspace_is_blocked(workspace_ref))
            .cloned();

        let execution_condition =
            if let Some(blocking_workspace_ref) = blocking_workspace_ref.clone() {
                ClusteredExecutionCondition {
                    kind: ClusteredExecutionKind::Failed,
                    active_workspace_ref: Some(projection.primary_workspace_ref.clone()),
                    blocking_workspace_ref: Some(blocking_workspace_ref.clone()),
                    summary: format!(
                        "cluster delivery is blocked by workspace {blocking_workspace_ref}"
                    ),
                    recovery_allowed: true,
                }
            } else {
                ClusteredExecutionCondition {
                    kind: match terminal_status {
                        TaskStatus::Succeeded => ClusteredExecutionKind::Success,
                        TaskStatus::Failed | TaskStatus::Aborted => ClusteredExecutionKind::Failed,
                        TaskStatus::Exhausted => ClusteredExecutionKind::Exhausted,
                        TaskStatus::Planned | TaskStatus::Running => ClusteredExecutionKind::Paused,
                    },
                    active_workspace_ref: Some(projection.primary_workspace_ref.clone()),
                    blocking_workspace_ref: None,
                    summary: format!(
                        "native cluster delivery executed from {}",
                        projection.primary_workspace_ref
                    ),
                    recovery_allowed: terminal_status != TaskStatus::Succeeded,
                }
            };

        let participating_workspaces = projection
            .member_workspace_refs
            .iter()
            .enumerate()
            .map(|(order, workspace_ref)| {
                let (participation_kind, latest_status, headline) =
                    if workspace_ref == &projection.primary_workspace_ref {
                        (
                            WorkspaceParticipationKind::Mutated,
                            Some(cluster_task_status_text(terminal_status).to_string()),
                            "authoritative native workspace executed the bounded goal".to_string(),
                        )
                    } else if blocking_workspace_ref.as_deref() == Some(workspace_ref.as_str()) {
                        (
                            WorkspaceParticipationKind::Blocked,
                            Some("blocked".to_string()),
                            "workspace currently blocks clustered follow-through".to_string(),
                        )
                    } else {
                        (
                            WorkspaceParticipationKind::ReadOnly,
                            Some("ready".to_string()),
                            "workspace remains aligned with the authoritative cluster route"
                                .to_string(),
                        )
                    };

                WorkspaceParticipationRecord {
                    workspace_ref: workspace_ref.clone(),
                    participation_kind,
                    order,
                    latest_trace_ref: None,
                    latest_status,
                    headline,
                    terminal_reason: None,
                }
            })
            .collect();

        ClusterDeliveryStory {
            cluster_id: projection.cluster_id.clone(),
            primary_workspace_ref: projection.primary_workspace_ref.clone(),
            authoritative_workspace_ref: projection.primary_workspace_ref.clone(),
            route_owner: ClusterRouteOwner::Native,
            member_workspace_refs: projection.member_workspace_refs.clone(),
            participating_workspaces,
            started_from_command: projection.started_from_command.clone(),
            execution_condition,
            updated_at: current_timestamp_millis(),
        }
    }

    pub(super) fn propagate_cluster_delivery_changes(
        &self,
        goal_plan: &GoalPlan,
        runtime: &FixtureRuntime,
    ) -> Result<(), SessionRuntimeError> {
        let Some(projection) = goal_plan.cluster_session_projection.as_ref() else {
            return Ok(());
        };

        let changed_paths = runtime
            .profile
            .attempts
            .iter()
            .flat_map(|attempt| attempt.changes.iter().map(|change| change.path.clone()))
            .collect::<BTreeSet<_>>();
        if changed_paths.is_empty() {
            return Ok(());
        }

        for workspace_ref in projection
            .member_workspace_refs
            .iter()
            .filter(|workspace_ref| *workspace_ref != &projection.primary_workspace_ref)
        {
            if cluster_workspace_is_blocked(workspace_ref) {
                continue;
            }

            for relative_path in &changed_paths {
                let source_path = self.workspace_ref.join(relative_path);
                let target_path = Path::new(workspace_ref).join(relative_path);
                let contents = std::fs::read(&source_path).map_err(|source| {
                    SessionRuntimeError::FixtureRuntime(FixtureRuntimeError::Io {
                        path: source_path.clone(),
                        source,
                    })
                })?;
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|source| {
                        SessionRuntimeError::FixtureRuntime(FixtureRuntimeError::Io {
                            path: parent.to_path_buf(),
                            source,
                        })
                    })?;
                }
                std::fs::write(&target_path, contents).map_err(|source| {
                    SessionRuntimeError::FixtureRuntime(FixtureRuntimeError::Io {
                        path: target_path.clone(),
                        source,
                    })
                })?;
            }
        }

        Ok(())
    }
}
