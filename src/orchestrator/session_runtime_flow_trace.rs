use super::*;

impl SessionRuntime {
    // Loads the current trace when present; otherwise creates a new trace and
    // records the initial task and flow-selection events.
    pub(super) fn load_or_create_trace(
        &self,
        session: &mut ActiveSessionRecord,
        task: &Task,
    ) -> Result<ExecutionTrace, SessionRuntimeError> {
        if let Some(trace_ref) = &session.latest_trace_ref {
            return self
                .trace_store
                .load(Path::new(trace_ref))
                .map_err(SessionRuntimeError::TraceStore);
        }

        let mut trace = ExecutionTrace::new(
            task.id.clone(),
            task.context.session_id.clone(),
            task.goal.clone(),
        );
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            task.plan.revision,
            json!({
                "goal": task.goal,
                "input": task.input,
                "limits": task.limits,
            }),
        );
        if let Some(active_flow) = &session.active_flow {
            trace.record_event(
                TraceEventType::FlowSelected,
                None,
                task.plan.revision,
                json!({
                    "flow_name": active_flow.flow_name,
                    "current_stage_id": active_flow.current_stage_id,
                    "current_stage_index": active_flow.current_stage_index,
                    "total_stages": active_flow.total_stages,
                }),
            );
        }
        let trace_location = self.persist_trace(&session.session_id, &mut trace)?;
        session.latest_trace_ref = Some(trace_location);

        Ok(trace)
    }

    pub(super) fn advance_session_flow(
        &self,
        session: &mut ActiveSessionRecord,
        task: &Task,
        completed_step_index: usize,
    ) -> Result<
        Option<(FlowStepMetadata, FlowStepMetadata)>,
        crate::domain::flow::FlowValidationError,
    > {
        let Some(active_flow) = session.active_flow.as_mut() else {
            return Ok(None);
        };

        let Some(completed_step) = task.plan.steps.get(completed_step_index) else {
            return Err(crate::domain::flow::FlowValidationError::InvalidStageIndex {
                flow_name: active_flow.flow_name.clone(),
                stage_index: completed_step_index,
                total_stages: task.plan.steps.len(),
            });
        };
        let Some(completed_metadata) = FlowStepMetadata::from_step(completed_step)? else {
            return Ok(None);
        };

        if completed_metadata.flow_name != active_flow.flow_name {
            return Err(crate::domain::flow::FlowValidationError::StageIdMismatch {
                flow_name: active_flow.flow_name.clone(),
                expected: active_flow.current_stage_id.clone(),
                actual: completed_metadata.stage_id,
            });
        }

        if let Some(next_step) = task.plan.steps.get(completed_step_index + 1)
            && let Some(next_metadata) = FlowStepMetadata::from_step(next_step)?
            && next_metadata.stage_index != active_flow.current_stage_index
        {
            active_flow.current_stage_index = next_metadata.stage_index;
            active_flow.current_stage_id = next_metadata.stage_id.clone();
            return Ok(Some((completed_metadata, next_metadata)));
        }

        Ok(None)
    }

    pub(super) fn flow_payload_for_step(
        &self,
        step: &Step,
    ) -> Result<Option<Value>, crate::domain::flow::FlowValidationError> {
        let Some(metadata) = FlowStepMetadata::from_step(step)? else {
            return Ok(None);
        };

        Ok(Some(json!({
            "flow_name": metadata.flow_name,
            "stage_id": metadata.stage_id,
            "stage_index": metadata.stage_index,
            "total_stages": metadata.total_stages,
        })))
    }

    pub(super) fn record_stage_failure(
        &self,
        trace: &mut ExecutionTrace,
        session: &ActiveSessionRecord,
        step_id: &str,
        plan_revision: usize,
        reason: &TerminalReason,
    ) {
        let Some(active_flow) = &session.active_flow else {
            return;
        };

        trace.record_event(
            TraceEventType::StageFailed,
            Some(step_id.to_string()),
            plan_revision,
            json!({
                "flow_name": active_flow.flow_name,
                "stage_id": active_flow.current_stage_id,
                "stage_index": active_flow.current_stage_index,
                "reason": reason.message,
            }),
        );
    }
}
