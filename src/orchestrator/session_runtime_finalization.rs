use super::*;

impl SessionRuntime {
    // Applies terminal state to task, trace, and session in one place so the
    // persisted snapshot stays aligned across all operator surfaces.
    pub(super) fn finalize_task(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        reason: TerminalReason,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let terminal_status = task_status_for_condition(reason.condition);
        if terminal_status != TaskStatus::Succeeded {
            let step_id = task
                .plan
                .current_step()
                .map(|step| step.id.clone())
                .unwrap_or_else(|| "terminal".to_string());
            self.record_stage_failure(trace, session, &step_id, task.plan.revision, &reason);
        }
        if !trace.events.iter().any(|event| event.event_type.is_reasoning_event())
            && let Some(reasoning_profile) = session
                .governance_lifecycle
                .as_ref()
                .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
        {
            let step_id =
                task.plan.current_step().map(|step| step.id.as_str()).unwrap_or("terminal");
            record_reasoning_profile_events(trace, step_id, task.plan.revision, reasoning_profile);
        }
        trace.record_event(
            TraceEventType::TerminalRecorded,
            None,
            task.plan.revision,
            json!({
                "terminal_status": terminal_status,
                "terminal_reason": reason,
            }),
        );
        task.apply_terminal(terminal_status, reason.clone());
        trace.finalize(terminal_status, reason.clone());
        let trace_location = self.persist_trace(&session.session_id, trace)?;

        session.latest_status = session_status_for_task_status(terminal_status);
        session.latest_terminal_reason = Some(reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status,
            terminal_reason: reason,
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    // Persist twice so the stored trace payload also contains its own final
    // trace location for downstream inspect and status rendering.
    pub(super) fn persist_trace(
        &self,
        session_id: &str,
        trace: &mut ExecutionTrace,
    ) -> Result<String, SessionRuntimeError> {
        let trace_store = FileTraceStore::for_session(&self.workspace_ref, session_id);
        let path = trace_store.persist(trace).map_err(SessionRuntimeError::TraceStore)?;
        let trace_location = path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        trace_store.persist(trace).map_err(SessionRuntimeError::TraceStore)?;
        self.project_trace_events_to_session_audit(session_id, &trace_location, trace)?;
        Ok(trace_location)
    }
}
