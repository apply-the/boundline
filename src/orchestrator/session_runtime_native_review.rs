use super::*;

impl SessionRuntime {
    pub(super) fn execute_native_review_sequence(
        &self,
        session: &ActiveSessionRecord,
        runtime: &FixtureRuntime,
        goal_plan: &GoalPlan,
        native_context: &mut TaskContext,
    ) -> Result<NativeReviewExecution, SessionRuntimeError> {
        let Some(review) = runtime.profile.review.as_ref() else {
            return Ok(NativeReviewExecution::default());
        };
        let Some(trigger) = Self::native_review_trigger(review) else {
            return Ok(NativeReviewExecution::default());
        };

        native_context.state.insert(NEXT_REVIEW_TRIGGER_KEY.to_string(), json!(trigger));
        let attempt_id = native_context
            .state
            .get(LATEST_ATTEMPT_ID_KEY)
            .and_then(Value::as_str)
            .unwrap_or(goal_plan.plan_id.as_str())
            .to_string();
        let mut review_trace = ExecutionTrace::new(
            goal_plan.plan_id.clone(),
            session.session_id.clone(),
            goal_plan.goal_text.clone(),
        );

        for reviewer in &review.reviewers {
            let mut step = Step::agent(
                format!("{NATIVE_REVIEW_STEP_PREFIX}-{}", reviewer.reviewer_id),
                NATIVE_REVIEWER_AGENT_NAME,
                json!({
                    "phase": NATIVE_REVIEW_PHASE,
                    "attempt_id": attempt_id.clone(),
                    "reviewer_id": reviewer.reviewer_id.clone(),
                    "adjudication": false,
                    "default_review_trigger": trigger,
                }),
            )
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
            let result = self.execute_native_follow_up_step(
                runtime,
                native_context,
                &mut review_trace,
                &mut step,
                goal_plan.proposal_revision,
            )?;
            if result.status == ExecutionStatus::Failed {
                return Ok(NativeReviewExecution {
                    events: review_trace.events,
                    terminal_reason: Self::native_review_terminal_reason(
                        native_context,
                        result.error.as_ref().map(|error| error.message.as_str()),
                    ),
                });
            }
        }

        let mut vote_step = Step::tool(
            NATIVE_REVIEW_VOTE_STEP_ID,
            NATIVE_REVIEW_VOTER_TOOL_NAME,
            json!({
                "phase": NATIVE_REVIEW_VOTE_PHASE,
                "attempt_id": attempt_id.clone(),
            }),
        )
        .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
        let vote_result = self.execute_native_follow_up_step(
            runtime,
            native_context,
            &mut review_trace,
            &mut vote_step,
            goal_plan.proposal_revision,
        )?;
        if vote_result.status == ExecutionStatus::Failed {
            return Ok(NativeReviewExecution {
                events: review_trace.events,
                terminal_reason: Self::native_review_terminal_reason(
                    native_context,
                    vote_result.error.as_ref().map(|error| error.message.as_str()),
                ),
            });
        }

        if review.adjudication.enabled {
            let adjudicator_id = review.adjudication.reviewer_id.as_ref().ok_or_else(|| {
                SessionRuntimeError::ExecutionInvariant(
                    "native review adjudication is enabled without an adjudicator".to_string(),
                )
            })?;
            let mut step = Step::agent(
                format!("{NATIVE_REVIEW_STEP_PREFIX}-adjudicate"),
                NATIVE_REVIEWER_AGENT_NAME,
                json!({
                    "phase": NATIVE_REVIEW_PHASE,
                    "attempt_id": attempt_id.clone(),
                    "reviewer_id": adjudicator_id,
                    "adjudication": true,
                    "default_review_trigger": trigger,
                }),
            )
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
            let result = self.execute_native_follow_up_step(
                runtime,
                native_context,
                &mut review_trace,
                &mut step,
                goal_plan.proposal_revision,
            )?;
            if result.status == ExecutionStatus::Failed {
                return Ok(NativeReviewExecution {
                    events: review_trace.events,
                    terminal_reason: Self::native_review_terminal_reason(
                        native_context,
                        result.error.as_ref().map(|error| error.message.as_str()),
                    ),
                });
            }
        }

        let mut finalize_step = Step::tool(
            NATIVE_REVIEW_FINALIZE_STEP_ID,
            NATIVE_REVIEW_FINALIZER_TOOL_NAME,
            json!({
                "phase": NATIVE_REVIEW_FINALIZE_PHASE,
                "attempt_id": attempt_id,
            }),
        )
        .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
        let finalize_result = self.execute_native_follow_up_step(
            runtime,
            native_context,
            &mut review_trace,
            &mut finalize_step,
            goal_plan.proposal_revision,
        )?;

        Ok(NativeReviewExecution {
            events: review_trace.events,
            terminal_reason: Self::native_review_terminal_reason(
                native_context,
                finalize_result.error.as_ref().map(|error| error.message.as_str()),
            ),
        })
    }

    fn execute_native_follow_up_step(
        &self,
        runtime: &FixtureRuntime,
        native_context: &mut TaskContext,
        trace: &mut ExecutionTrace,
        step: &mut Step,
        plan_revision: usize,
    ) -> Result<StepExecutionResult, SessionRuntimeError> {
        step.mark_running();
        let started_at = current_timestamp_millis();
        let mut attempt = StepAttempt::new(step.id.clone(), step.input.clone(), started_at);
        trace.record_event(
            TraceEventType::StepStarted,
            Some(step.id.clone()),
            plan_revision,
            json!({
                "attempt_number": step.attempt_count,
                "input": step.input.clone(),
                "step_kind": step.kind,
            }),
        );
        record_review_step_started(
            trace,
            &step.id,
            &step.input,
            &native_context.state,
            plan_revision,
        );

        let result = self.normalize_result(self.execute_step(runtime, step, native_context), step);
        attempt.complete(&result, current_timestamp_millis());
        native_context.push_history_ref(attempt.attempt_id.clone());

        match result.status {
            ExecutionStatus::Succeeded => {
                let output = result.output.clone().ok_or_else(|| {
                    SessionRuntimeError::ExecutionInvariant(format!(
                        "native review step {} reported success without output",
                        step.id
                    ))
                })?;
                step.mark_succeeded(output.clone());
                native_context.apply_success_output(&step.id, &output, result.state_patch.as_ref());
                native_context.set_last_result(StepResultSummary::from_step(step));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step.id.clone()),
                    plan_revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "succeeded",
                        "output": output,
                        "evidence": result.evidence,
                    }),
                );
            }
            ExecutionStatus::Failed => {
                let error = result.error.clone().ok_or_else(|| {
                    SessionRuntimeError::ExecutionInvariant(format!(
                        "native review step {} reported failure without error",
                        step.id
                    ))
                })?;
                step.mark_failed(error.clone(), result.recoverability);
                native_context.apply_failure_error(&step.id, &error);
                if let Some(state_patch) = result.state_patch.as_ref() {
                    native_context.apply_state_patch(state_patch);
                }
                native_context.set_last_result(StepResultSummary::from_step(step));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step.id.clone()),
                    plan_revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "failed",
                        "error": error,
                        "recoverability": result.recoverability,
                        "evidence": result.evidence,
                    }),
                );
            }
        }

        record_review_step_completed(
            trace,
            &step.id,
            &step.input,
            &result,
            &native_context.state,
            plan_revision,
        );

        Ok(result)
    }

    fn native_review_trigger(review: &ReviewProfile) -> Option<ReviewTrigger> {
        review
            .triggers
            .iter()
            .copied()
            .find(|trigger| !matches!(trigger, ReviewTrigger::ValidationFailed))
            .or_else(|| review.triggers.first().copied())
    }

    fn native_review_terminal_reason(
        native_context: &TaskContext,
        failure_message: Option<&str>,
    ) -> Option<TerminalReason> {
        let outcome = native_context
            .state
            .get(LATEST_REVIEW_OUTCOME_KEY)
            .cloned()
            .and_then(|value| serde_json::from_value::<ReviewOutcome>(value).ok());
        let mut details = Map::new();
        for key in [
            "latest_review_trigger",
            "latest_review_findings",
            "latest_review_participants",
            "latest_review_vote_resolution",
            "latest_review_vote",
        ] {
            if let Some(value) = native_context.state.get(key).cloned() {
                details.insert(key.to_string(), value);
            }
        }
        let details = (!details.is_empty()).then_some(Value::Object(details));

        match outcome {
            Some(ReviewOutcome::Accepted) => None,
            Some(ReviewOutcome::Rejected) => Some(build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                failure_message.unwrap_or("native review rejected the delivery result"),
                details,
            )),
            Some(ReviewOutcome::Escalated) => Some(build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                failure_message.unwrap_or("native review escalated and requires follow-up"),
                details,
            )),
            Some(ReviewOutcome::Failed) | None => Some(build_terminal_reason(
                TerminalCondition::UnrecoverableError,
                failure_message.unwrap_or("native review failed"),
                details,
            )),
        }
    }
}
