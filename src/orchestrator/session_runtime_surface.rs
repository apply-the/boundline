use std::path::{Path, PathBuf};

use serde_json::json;

use crate::adapters::audit_store::SessionAuditStore;
use crate::adapters::session_store::SessionStore;
use crate::adapters::trace_store::TraceStore;
use crate::domain::trace::ExecutionTrace;

use super::{
    ActiveSessionRecord, AuthoredBriefBundle, ClusterSessionProjection, FileCheckpointStore,
    FileSessionAuditStore, FileSessionStore, FileTraceStore, FlowPolicy, FollowThroughProjection,
    NegotiatedDeliveryPacket, SessionAuditActor, SessionAuditAlgorithm, SessionAuditEntry,
    SessionAuditEntryKind, SessionAuditIdentity, SessionAuditOutcome, SessionAuditOutcomeStatus,
    SessionAuditPhase, SessionAuditSource, SessionAuditSourceKind, SessionRuntime,
    SessionRuntimeError, SessionStatus, built_in_flow, current_timestamp_millis, git_config_value,
    session_audit_outcome_for_status, session_status_text, supported_flow_names_csv,
    trace_event_audit_actor, trace_event_audit_algorithm, trace_event_audit_message,
    trace_event_audit_outcome, trace_event_type_text,
};

impl SessionRuntime {
    /// Returns a runtime bound to one workspace and its persisted stores.
    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        let workspace_ref = workspace_ref.as_ref().to_path_buf();
        Self {
            checkpoint_store: FileCheckpointStore::for_workspace(&workspace_ref),
            session_store: FileSessionStore::for_workspace(&workspace_ref),
            trace_store: FileTraceStore::for_workspace(&workspace_ref),
            workspace_ref,
        }
    }

    /// Returns the workspace this runtime operates on.
    pub fn workspace_ref(&self) -> &Path {
        &self.workspace_ref
    }

    /// Returns the session store used by this runtime.
    pub fn session_store(&self) -> &FileSessionStore {
        &self.session_store
    }

    /// Returns the checkpoint store used by this runtime.
    pub fn checkpoint_store(&self) -> &FileCheckpointStore {
        &self.checkpoint_store
    }

    /// Returns the trace store used by this runtime.
    pub fn trace_store(&self) -> &FileTraceStore {
        &self.trace_store
    }

    /// Loads the active workspace session, if one exists.
    pub fn load_session(&self) -> Result<Option<ActiveSessionRecord>, SessionRuntimeError> {
        self.session_store.load().map_err(SessionRuntimeError::SessionStore)
    }

    /// Persists the active session snapshot.
    pub fn persist_session(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<PathBuf, SessionRuntimeError> {
        let previous = self
            .session_store
            .load_session(&session.session_id)
            .map_err(SessionRuntimeError::SessionStore)?;
        let path =
            self.session_store.persist(session).map_err(SessionRuntimeError::SessionStore)?;
        self.sync_session_audit_lifecycle(previous.as_ref(), session)?;
        Ok(path)
    }

    /// Clears the active workspace session.
    pub fn clear_session(&self) -> Result<(), SessionRuntimeError> {
        self.session_store.clear().map_err(SessionRuntimeError::SessionStore)
    }

    fn sync_session_audit_lifecycle(
        &self,
        previous: Option<&ActiveSessionRecord>,
        session: &ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let audit_store =
            FileSessionAuditStore::for_session(&self.workspace_ref, &session.session_id);
        let mut cursor =
            audit_store.load_cursor().map_err(SessionRuntimeError::SessionAuditStore)?;
        let session_identity = self.resolve_session_audit_identity();
        let system_actor = SessionAuditActor::system("boundline");
        let mut cursor_dirty = false;

        if !cursor.session_start_recorded {
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.created_at,
                SessionAuditEntryKind::SessionStart,
                "session started",
                session_identity.clone(),
                system_actor.clone(),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session opened"),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "workspace_ref": session.workspace_ref,
                    "goal": session.goal,
                    "latest_status": session_status_text(session.latest_status),
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.session_start_recorded = true;
            cursor_dirty = true;
        }

        let previous_status =
            previous.map(|record| session_status_text(record.latest_status).to_string());
        let current_status = session_status_text(session.latest_status).to_string();
        if cursor.latest_session_status.as_deref() != Some(current_status.as_str())
            || previous_status.as_deref() != Some(current_status.as_str())
        {
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.updated_at,
                SessionAuditEntryKind::SessionStatusChanged,
                format!("session status changed to {current_status}"),
                session_identity.clone(),
                system_actor.clone(),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                session_audit_outcome_for_status(session.latest_status),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "previous_status": previous_status,
                    "current_status": current_status,
                    "terminal_reason": session.latest_terminal_reason,
                    "latest_trace_ref": session.latest_trace_ref,
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.latest_session_status = Some(current_status.clone());
            cursor_dirty = true;
        }

        let previous_follow_through =
            previous.map(FollowThroughProjection::from_session_record).unwrap_or_default();
        let current_follow_through = FollowThroughProjection::from_session_record(session);
        if previous_follow_through != current_follow_through
            && (!previous_follow_through.is_empty() || !current_follow_through.is_empty())
        {
            let message = if current_follow_through.is_empty() {
                "follow-through projection cleared".to_string()
            } else {
                current_follow_through
                    .guidance
                    .clone()
                    .map(|guidance| format!("follow-through projection updated: {guidance}"))
                    .unwrap_or_else(|| "follow-through projection updated".to_string())
            };
            let outcome_message = if current_follow_through.is_empty() {
                "projection cleared"
            } else {
                "projection refreshed"
            };
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.updated_at,
                SessionAuditEntryKind::FollowThroughProjected,
                message,
                session_identity.clone(),
                system_actor.clone(),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Inspect,
                    "follow_through",
                    "from_session_record",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Projected, outcome_message),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "previous_follow_through": previous_follow_through,
                    "current_follow_through": current_follow_through,
                    "latest_status": current_status,
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor_dirty = true;
        }

        if session.latest_status.is_terminal() && !cursor.session_end_recorded {
            let terminal_message = session
                .latest_terminal_reason
                .as_ref()
                .map(|reason| reason.message.trim().to_string())
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| {
                    format!(
                        "session ended with status {}",
                        session_status_text(session.latest_status)
                    )
                });
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.updated_at,
                SessionAuditEntryKind::SessionEnd,
                terminal_message.clone(),
                session_identity,
                system_actor,
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                session_audit_outcome_for_status(session.latest_status),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "latest_status": session_status_text(session.latest_status),
                    "terminal_reason": session.latest_terminal_reason,
                    "latest_trace_ref": session.latest_trace_ref,
                    "goal": session.goal,
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.session_end_recorded = true;
            cursor_dirty = true;
        } else if !session.latest_status.is_terminal() && cursor.session_end_recorded {
            cursor.session_end_recorded = false;
            cursor_dirty = true;
        }

        if cursor_dirty {
            audit_store.persist_cursor(&cursor).map_err(SessionRuntimeError::SessionAuditStore)?;
        }

        Ok(())
    }

    fn resolve_session_audit_identity(&self) -> SessionAuditIdentity {
        SessionAuditIdentity {
            git_user_name: git_config_value(&self.workspace_ref, "user.name"),
            git_user_email: git_config_value(&self.workspace_ref, "user.email"),
        }
    }

    pub(super) fn project_trace_events_to_session_audit(
        &self,
        session_id: &str,
        trace_ref: &str,
        trace: &ExecutionTrace,
    ) -> Result<(), SessionRuntimeError> {
        let audit_store = FileSessionAuditStore::for_session(&self.workspace_ref, session_id);
        let mut cursor =
            audit_store.load_cursor().map_err(SessionRuntimeError::SessionAuditStore)?;
        let session_identity = self.resolve_session_audit_identity();
        let mut cursor_dirty = false;

        for event in &trace.events {
            if cursor.already_projected(&trace.task_id, &event.event_id) {
                continue;
            }

            let entry = SessionAuditEntry::new_with_timestamp(
                session_id.to_string(),
                cursor.next_sequence(),
                event.recorded_at,
                SessionAuditEntryKind::TraceEventProjected,
                trace_event_audit_message(event),
                session_identity.clone(),
                trace_event_audit_actor(event),
                trace_event_audit_algorithm(event.event_type),
                trace_event_audit_outcome(event),
                SessionAuditSource {
                    kind: SessionAuditSourceKind::TraceEvent,
                    trace_ref: Some(trace_ref.to_string()),
                    trace_event_id: Some(event.event_id.clone()),
                    trace_event_type: Some(trace_event_type_text(event.event_type)),
                    step_id: event.step_id.clone(),
                    plan_revision: Some(event.plan_revision),
                },
                event.payload.clone(),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.mark_projected(trace.task_id.clone(), event.event_id.clone());
            cursor_dirty = true;
        }

        if cursor_dirty {
            audit_store.persist_cursor(&cursor).map_err(SessionRuntimeError::SessionAuditStore)?;
        }

        Ok(())
    }

    /// Returns the latest persisted trace for the workspace, if available.
    pub fn latest_trace(&self) -> Result<Option<PathBuf>, SessionRuntimeError> {
        self.trace_store.latest().map_err(SessionRuntimeError::TraceStore)
    }

    /// Captures a new goal into the session and resets any active execution
    /// state so planning can restart from a clean bounded snapshot.
    pub fn capture_goal(
        &self,
        session: &mut ActiveSessionRecord,
        goal: &str,
    ) -> Result<(), SessionRuntimeError> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(SessionRuntimeError::MissingGoal);
        }

        session.goal = Some(goal.to_string());
        session.negotiation_packet = Some(session.authored_brief.as_ref().map_or_else(
            || {
                NegotiatedDeliveryPacket::from_goal(
                    &session.session_id,
                    &session.workspace_ref,
                    goal,
                )
            },
            |bundle| {
                NegotiatedDeliveryPacket::from_authored_brief(
                    &session.session_id,
                    &session.workspace_ref,
                    goal,
                    bundle,
                )
            },
        ));
        session.active_task = None;
        session.goal_plan = None;
        session.decisions.clear();
        self.ensure_workspace_governance_lifecycle(session);
        session.latest_status = SessionStatus::GoalCaptured;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn refresh_planning_input(
        &self,
        session: &mut ActiveSessionRecord,
        bundle: AuthoredBriefBundle,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;

        session.authored_brief = Some(bundle.clone());
        session.negotiation_packet = Some(NegotiatedDeliveryPacket::from_authored_brief(
            &session.session_id,
            &session.workspace_ref,
            &goal,
            &bundle,
        ));
        session.active_task = None;
        session.goal_plan = None;
        session.decisions.clear();
        self.ensure_workspace_governance_lifecycle(session);
        session.latest_status = SessionStatus::GoalCaptured;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn select_flow(
        &self,
        session: &mut ActiveSessionRecord,
        flow_name: &str,
    ) -> Result<(), SessionRuntimeError> {
        let flow = built_in_flow(flow_name).ok_or_else(|| SessionRuntimeError::UnknownFlow {
            requested: flow_name.to_string(),
            supported: supported_flow_names_csv(),
        })?;

        if session.active_task.is_some() {
            return Err(SessionRuntimeError::FlowReplacementRequiresReset {
                current: session
                    .active_flow
                    .as_ref()
                    .map(|existing_flow| existing_flow.flow_name.clone())
                    .unwrap_or_else(|| "current-session".to_string()),
                requested: flow.name.to_string(),
            });
        }
        if session.goal_plan.is_some() {
            return Err(SessionRuntimeError::FlowReplacementRequiresReset {
                current: session
                    .active_flow
                    .as_ref()
                    .map(|existing_flow| existing_flow.flow_name.clone())
                    .unwrap_or_else(|| "current-session".to_string()),
                requested: flow.name.to_string(),
            });
        }

        session.active_flow = Some(flow.initial_state());
        session.active_task = None;
        session.goal_plan = None;
        session.decisions.clear();
        session.active_flow_policy = FlowPolicy::from_builtin(flow.name).ok();
        session.latest_trace_ref = None;
        session.latest_terminal_reason = None;
        session.latest_status = if session.goal.is_some() {
            SessionStatus::GoalCaptured
        } else {
            SessionStatus::Initialized
        };
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn plan_task(
        &self,
        session: &mut ActiveSessionRecord,
        requested_flow: Option<&str>,
        no_flow: bool,
    ) -> Result<(), SessionRuntimeError> {
        if session.active_task.is_some()
            && session.goal_plan.is_none()
            && requested_flow.is_none()
            && !no_flow
        {
            return self.plan_compatibility_task(session);
        }

        let result = self.plan_goal_plan(session, requested_flow, no_flow);
        if matches!(result, Err(SessionRuntimeError::ClarificationRequired { .. }))
            && session.active_flow.is_some()
            && session.goal_plan.is_some()
            && requested_flow.is_none()
            && !no_flow
            && self.flow_selected_goal_plan_uses_compatibility_step(session)?
        {
            return self.plan_compatibility_task(session);
        }

        result
    }

    pub fn confirm_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        if session.goal_plan.is_none() {
            return Err(SessionRuntimeError::MissingGoalPlan);
        }
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

        let goal_plan = session.goal_plan.as_mut().ok_or(SessionRuntimeError::MissingGoalPlan)?;
        if goal_plan.requires_confirmation() {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
        }

        session.active_task = None;
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();
        Ok(())
    }

    /// Prepares cluster-scoped state before a clustered run starts.
    pub fn prepare_cluster_run(
        &self,
        session: &mut ActiveSessionRecord,
        projection: &ClusterSessionProjection,
    ) -> Result<(), SessionRuntimeError> {
        if let Some(task) = session.active_task.as_mut() {
            task.context
                .set_cluster_session_projection(projection)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(goal_plan) = session.goal_plan.as_mut() {
            goal_plan.cluster_session_projection = Some(projection.clone());
            goal_plan.cluster_delivery_story = None;
        }

        Ok(())
    }

    /// Returns true when the session is currently operating on a native goal
    /// plan instead of a compatibility task.
    pub fn uses_native_goal_plan(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        Ok(session.goal_plan.is_some())
    }

    /// Projects the effective routing outcome for the current session state.
    pub fn resolve_routing_outcome(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<crate::domain::session::RoutingOutcome, SessionRuntimeError> {
        Ok(crate::domain::session::routing_outcome(session))
    }
}
