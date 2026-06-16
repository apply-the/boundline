use serde_json::json;

use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::domain::completion_verification::{
    ClaimConfirmationContext, ClaimConfirmationRequirement, CompletionClaim,
    CompletionRequiredAction, CompletionVerificationFinding, CompletionVerificationFindingKind,
    CompletionVerificationFindingSeverity, CompletionVerificationProjection,
    CompletionVerificationScope, CompletionVerificationState, claim_confirmation_requirement,
    infer_claim_kind_from_text, infer_completion_claim, proof_rules_for_validation_command,
    select_proof_command,
};
use crate::domain::session::ActiveSessionRecord;
use crate::domain::task::{Task, TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::trace::{ExecutionTrace, TraceEventType, current_timestamp_millis};
use crate::orchestrator::review_trace::record_reasoning_profile_events;
use crate::orchestrator::terminal::task_status_for_condition;

use super::native_execution::NativeProofExecutionRequest;
use super::{SessionRuntime, SessionRuntimeError, session_status_for_task_status};

pub(super) struct CompletionCloseoutEvaluationInput<'a> {
    pub task_id: &'a str,
    pub goal: &'a str,
    pub expected_outcome: Option<&'a str>,
    pub changed_files: &'a [String],
    pub existing_claim: Option<CompletionClaim>,
    pub validation_command: Option<&'a str>,
}

pub(super) enum CompletionCloseoutEvaluation {
    SkipVerification,
    ClaimConflict(CompletionVerificationProjection),
    ConfirmationRequired {
        claim: CompletionClaim,
        proof_selection: crate::domain::completion_verification::ProofCommandSelection,
        projection: CompletionVerificationProjection,
    },
    ExecuteProof {
        claim: CompletionClaim,
        proof_selection: crate::domain::completion_verification::ProofCommandSelection,
    },
    MissingProof {
        claim: CompletionClaim,
        projection: CompletionVerificationProjection,
    },
}

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
        if terminal_status == TaskStatus::Succeeded
            && let Some(blocked_response) = self.block_completion_closeout(session, task, trace)?
        {
            return Ok(blocked_response);
        }
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

    fn block_completion_closeout(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        let existing_projection = task
            .context
            .completion_verification_projection()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        if let Some(existing_projection) = existing_projection.as_ref()
            && existing_projection.scope != CompletionVerificationScope::Task
            && existing_projection.completion_verification_state
                != CompletionVerificationState::Ready
        {
            let trace_location = self.persist_trace(&session.session_id, trace)?;
            session.latest_status = crate::domain::session::SessionStatus::Blocked;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = Some(trace_location.clone());
            session.updated_at = current_timestamp_millis();

            return Ok(Some(TaskRunResponse {
                task_id: task.id.clone(),
                terminal_status: TaskStatus::Running,
                terminal_reason: TerminalReason::new(
                    crate::domain::limits::TerminalCondition::GoalSatisfied,
                    "completion verification blocked closeout because a parent scope remains unresolved",
                    None,
                ),
                final_context: task.context.clone(),
                plan_revision: task.plan.revision,
                trace_location,
            }));
        }
        // Closeout must prove the completion action rather than the initial
        // analysis step, so use the terminal planned task outcome.
        let expected_outcome = session
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.tasks.last())
            .and_then(|planned_task| planned_task.expected_outcome.as_deref());
        let changed_files = task
            .context
            .state
            .get("latest_changed_files")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let proof_rules = self
            .build_runtime(session)
            .ok()
            .map(|runtime| {
                proof_rules_for_validation_command(
                    "workspace_validation_command",
                    runtime.profile.validation_command.rendered(),
                )
            })
            .unwrap_or_default();
        let existing_claim = task
            .context
            .completion_claim()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        match evaluate_completion_closeout(
            CompletionCloseoutEvaluationInput {
                task_id: &task.id,
                goal: &task.goal,
                expected_outcome,
                changed_files: &changed_files,
                existing_claim,
                validation_command: proof_rules.first().map(|rule| rule.command_line.as_str()),
            },
            &proof_rules,
        )? {
            CompletionCloseoutEvaluation::SkipVerification => Ok(None),
            CompletionCloseoutEvaluation::ClaimConflict(conflict_projection) => {
                task.set_completion_claim(conflict_projection.claim.as_ref().ok_or_else(|| {
                    SessionRuntimeError::TaskContext(
                        "claim-conflict projection requires an active claim".to_string(),
                    )
                })?)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                task.set_completion_verification_projection(&conflict_projection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                let trace_location = self.persist_trace(&session.session_id, trace)?;
                session.latest_status = crate::domain::session::SessionStatus::Blocked;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location.clone());
                session.updated_at = current_timestamp_millis();

                Ok(Some(TaskRunResponse {
                    task_id: task.id.clone(),
                    terminal_status: TaskStatus::Running,
                    terminal_reason: TerminalReason::new(
                        crate::domain::limits::TerminalCondition::GoalSatisfied,
                        "completion verification blocked closeout pending claim conflict resolution",
                        None,
                    ),
                    final_context: task.context.clone(),
                    plan_revision: task.plan.revision,
                    trace_location,
                }))
            }
            CompletionCloseoutEvaluation::ConfirmationRequired {
                claim,
                proof_selection,
                projection,
            } => {
                task.set_completion_claim(&claim)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                task.set_completion_proof_selection(&proof_selection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                task.set_completion_verification_projection(&projection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                let trace_location = self.persist_trace(&session.session_id, trace)?;
                session.latest_status = crate::domain::session::SessionStatus::Blocked;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location.clone());
                session.updated_at = current_timestamp_millis();

                Ok(Some(TaskRunResponse {
                    task_id: task.id.clone(),
                    terminal_status: TaskStatus::Running,
                    terminal_reason: TerminalReason::new(
                        crate::domain::limits::TerminalCondition::GoalSatisfied,
                        "completion verification blocked closeout pending claim confirmation",
                        None,
                    ),
                    final_context: task.context.clone(),
                    plan_revision: task.plan.revision,
                    trace_location,
                }))
            }
            CompletionCloseoutEvaluation::ExecuteProof { claim, proof_selection } => {
                task.set_completion_claim(&claim)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                task.set_completion_proof_selection(&proof_selection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                Ok(Some(self.execute_completion_proof(
                    session,
                    task,
                    trace,
                    NativeProofExecutionRequest {
                        claim,
                        documentation_relevant: proof_selection.documentation_relevant,
                        command_line: proof_selection.command_line,
                        command_ref: proof_selection.command_ref,
                    },
                )?))
            }
            CompletionCloseoutEvaluation::MissingProof { claim, projection } => {
                task.set_completion_claim(&claim)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                task.set_completion_verification_projection(&projection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;

                let trace_location = self.persist_trace(&session.session_id, trace)?;
                session.latest_status = crate::domain::session::SessionStatus::Blocked;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location.clone());
                session.updated_at = current_timestamp_millis();

                Ok(Some(TaskRunResponse {
                    task_id: task.id.clone(),
                    terminal_status: TaskStatus::Running,
                    terminal_reason: TerminalReason::new(
                        crate::domain::limits::TerminalCondition::GoalSatisfied,
                        "completion verification blocked closeout pending proof",
                        None,
                    ),
                    final_context: task.context.clone(),
                    plan_revision: task.plan.revision,
                    trace_location,
                }))
            }
        }
    }
}

pub(super) fn conflicting_claim_projection(
    task_id: &str,
    goal: &str,
    expected_outcome: Option<&str>,
    changed_files: &[String],
) -> Result<Option<CompletionVerificationProjection>, SessionRuntimeError> {
    let Some(expected_outcome) = expected_outcome.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let Some(goal_kind) = infer_claim_kind_from_text(goal) else {
        return Ok(None);
    };
    let Some(expected_kind) = infer_claim_kind_from_text(expected_outcome) else {
        return Ok(None);
    };
    if goal_kind == expected_kind {
        return Ok(None);
    }

    let claim = match infer_completion_claim(
        format!("claim-{task_id}"),
        goal,
        Some(expected_outcome),
        changed_files,
        None,
    ) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    Ok(Some(CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::Blocked,
        scope: CompletionVerificationScope::Task,
        claim: Some(claim.clone()),
        completion_blocked_claims: vec![claim.kind],
        completion_evidence_refs: Vec::new(),
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::ClaimConflict,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: format!(
                "task metadata conflicts with runtime context: goal implies `{}`, expected outcome implies `{}`",
                goal_kind.as_str(),
                expected_kind.as_str()
            ),
            proof_ref: None,
            task_id: Some(task_id.to_string()),
            changed_paths: Vec::new(),
            required_action: CompletionRequiredAction::ResolveConflict,
        }],
        child_summary: None,
    }))
}

pub(super) fn confirmation_required_projection(
    task_id: &str,
    claim: &CompletionClaim,
    proof_selection: Option<&crate::domain::completion_verification::ProofCommandSelection>,
) -> Option<CompletionVerificationProjection> {
    let requirement = claim_confirmation_requirement(
        claim.confidence,
        &ClaimConfirmationContext {
            multiple_plausible_claims: false,
            proof_only_partially_covers_claim: proof_selection
                .is_some_and(|selection| !selection.fully_covers_claim()),
            risky_surface: claim.kind
                == crate::domain::completion_verification::CompletionClaimKind::MigrationValid,
            conflicting_claim_signals: false,
            policy_allows_medium_without_confirmation: false,
        },
    );
    if requirement != ClaimConfirmationRequirement::ConfirmationRequired {
        return None;
    }
    let selection = proof_selection?;
    let mut message = format!(
        "inferred claim `{}` has {} confidence and requires operator confirmation before proof execution can proceed; selected proof command `{}`",
        claim.kind.as_str(),
        claim
            .confidence
            .map(|value| match value {
                crate::domain::completion_verification::ClaimInferenceConfidence::High => "high",
                crate::domain::completion_verification::ClaimInferenceConfidence::Medium =>
                    "medium",
                crate::domain::completion_verification::ClaimInferenceConfidence::Low => "low",
            })
            .unwrap_or("unknown"),
        selection.command_line
    );
    if !claim.supporting_signals.is_empty() {
        message.push_str(&format!("; evidence used: {}", claim.supporting_signals.join(", ")));
    }
    Some(CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::Blocked,
        scope: CompletionVerificationScope::Task,
        claim: Some(claim.clone()),
        completion_blocked_claims: vec![claim.kind],
        completion_evidence_refs: Vec::new(),
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::MismatchedProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message,
            proof_ref: Some(selection.command_ref.clone()),
            task_id: Some(task_id.to_string()),
            changed_paths: Vec::new(),
            required_action: CompletionRequiredAction::ConfirmClaim,
        }],
        child_summary: None,
    })
}

fn missing_proof_projection(
    task_id: &str,
    claim: &CompletionClaim,
) -> CompletionVerificationProjection {
    CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::Blocked,
        scope: CompletionVerificationScope::Task,
        claim: Some(claim.clone()),
        completion_blocked_claims: vec![claim.kind],
        completion_evidence_refs: Vec::new(),
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::MissingProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: format!(
                "no proving command exists for the inferred claim `{}`",
                claim.summary
            ),
            proof_ref: None,
            task_id: Some(task_id.to_string()),
            changed_paths: Vec::new(),
            required_action: CompletionRequiredAction::ClarifyClaim,
        }],
        child_summary: None,
    }
}

pub(super) fn evaluate_completion_closeout(
    input: CompletionCloseoutEvaluationInput<'_>,
    proof_rules: &[crate::domain::completion_verification::ProofCommandRule],
) -> Result<CompletionCloseoutEvaluation, SessionRuntimeError> {
    if let Some(conflict_projection) = conflicting_claim_projection(
        input.task_id,
        input.goal,
        input.expected_outcome,
        input.changed_files,
    )? {
        return Ok(CompletionCloseoutEvaluation::ClaimConflict(conflict_projection));
    }
    if proof_rules.is_empty() {
        return Ok(CompletionCloseoutEvaluation::SkipVerification);
    }

    let claim = match input.existing_claim {
        Some(claim) => claim,
        None => match infer_completion_claim(
            format!("claim-{}", input.task_id),
            input.goal,
            input.expected_outcome,
            input.changed_files,
            input.validation_command,
        ) {
            Ok(claim) => claim,
            Err(_) => return Ok(CompletionCloseoutEvaluation::SkipVerification),
        },
    };
    let proof_selection = select_proof_command(&claim, proof_rules)
        .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;

    if let Some(projection) =
        confirmation_required_projection(input.task_id, &claim, proof_selection.as_ref())
    {
        return Ok(match proof_selection {
            Some(proof_selection) => CompletionCloseoutEvaluation::ConfirmationRequired {
                claim,
                proof_selection,
                projection,
            },
            None => CompletionCloseoutEvaluation::MissingProof {
                claim: claim.clone(),
                projection: missing_proof_projection(input.task_id, &claim),
            },
        });
    }

    Ok(match proof_selection {
        Some(proof_selection) => {
            CompletionCloseoutEvaluation::ExecuteProof { claim, proof_selection }
        }
        None => CompletionCloseoutEvaluation::MissingProof {
            claim: claim.clone(),
            projection: missing_proof_projection(input.task_id, &claim),
        },
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::{SessionRuntime, confirmation_required_projection, conflicting_claim_projection};
    use crate::domain::completion_verification::{
        ClaimInferenceConfidence, CompletionClaim, CompletionClaimKind, CompletionClaimSource,
        CompletionRequiredAction, CompletionVerificationFinding, CompletionVerificationFindingKind,
        CompletionVerificationFindingSeverity, CompletionVerificationProjection,
        CompletionVerificationScope, CompletionVerificationState,
    };
    use crate::domain::execution::{ExecutionCommand, WorkspaceExecutionProfile};
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::Plan;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::step::Step;
    use crate::domain::task::Task;
    use crate::domain::trace::ExecutionTrace;

    const SESSION_ID: &str = "session-finalization";
    const STEP_ID: &str = "step-finalization";
    const UPDATED_AT: u64 = 111;

    #[test]
    fn helper_projections_cover_conflict_and_confirmation_paths() -> Result<(), Box<dyn Error>> {
        let conflict = conflicting_claim_projection(
            "task-1",
            "fix the failing add test",
            Some("build stays clean"),
            &["src/lib.rs".to_string()],
        )?;
        assert!(conflict.is_some());
        assert!(conflict.as_ref().is_some_and(|projection| {
            projection.completion_verification_findings.iter().any(|finding| {
                finding.kind == CompletionVerificationFindingKind::ClaimConflict
                    && finding.required_action == CompletionRequiredAction::ResolveConflict
            })
        }));

        let confirmation = confirmation_required_projection(
            "task-2",
            &CompletionClaim {
                claim_id: "claim-2".to_string(),
                kind: CompletionClaimKind::BuildClean,
                scope: CompletionVerificationScope::Task,
                source: CompletionClaimSource::ExplicitMetadata,
                confidence: Some(ClaimInferenceConfidence::Medium),
                summary: "build stays clean".to_string(),
                supporting_signals: vec!["goal_text".to_string()],
            },
            Some(&crate::domain::completion_verification::ProofCommandSelection {
                claim_id: "claim-2".to_string(),
                command_ref: "proof.cargo-test".to_string(),
                command_line: "cargo test --quiet".to_string(),
                selection_reason: "best available proof".to_string(),
                coverage_note: Some("proves only tests_pass".to_string()),
                documentation_relevant: false,
            }),
        );
        assert!(confirmation.is_some());
        assert!(confirmation.as_ref().is_some_and(|projection| {
            projection
                .claim
                .as_ref()
                .is_some_and(|claim| claim.source == CompletionClaimSource::ExplicitMetadata)
                && projection.completion_verification_findings.iter().any(|finding| {
                    finding.required_action == CompletionRequiredAction::ConfirmClaim
                        && finding.message.contains("selected proof command")
                })
        }));

        Ok(())
    }

    #[test]
    fn block_completion_closeout_covers_parent_conflict_confirmation_missing_and_skip_paths()
    -> Result<(), Box<dyn Error>> {
        let parent_workspace = TestWorkspace::new("boundline-finalization-parent-block")?;
        let runtime = SessionRuntime::for_workspace(parent_workspace.as_path());
        let mut parent_session =
            sample_session(parent_workspace.as_path(), "fix the failing add test");
        let mut parent_task = sample_task(
            parent_workspace.as_path(),
            "parent-block-task",
            "fix the failing add test",
        )?;
        parent_task.set_completion_verification_projection(&CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::Blocked,
            scope: CompletionVerificationScope::Stage,
            claim: Some(bug_fix_claim()),
            completion_blocked_claims: vec![CompletionClaimKind::BugFixed],
            completion_evidence_refs: Vec::new(),
            completion_verification_findings: vec![CompletionVerificationFinding {
                kind: CompletionVerificationFindingKind::MissingChildProof,
                severity: CompletionVerificationFindingSeverity::Blocking,
                message: "parent scope is still blocked".to_string(),
                proof_ref: None,
                task_id: Some("T-1".to_string()),
                changed_paths: Vec::new(),
                required_action: CompletionRequiredAction::RunProof,
            }],
            child_summary: None,
        })?;
        let mut parent_trace = ExecutionTrace::new(&parent_task.id, SESSION_ID, "parent block");
        let parent_response = runtime.block_completion_closeout(
            &mut parent_session,
            &mut parent_task,
            &mut parent_trace,
        )?;
        assert!(parent_response.is_some());
        assert_eq!(parent_session.latest_status, SessionStatus::Blocked);

        let conflict_workspace = TestWorkspace::new("boundline-finalization-conflict")?;
        let runtime = SessionRuntime::for_workspace(conflict_workspace.as_path());
        let mut conflict_session =
            sample_session(conflict_workspace.as_path(), "fix the failing add test");
        conflict_session.goal_plan = Some(goal_plan_with_expected_outcome("build stays clean")?);
        let mut conflict_task =
            sample_task(conflict_workspace.as_path(), "conflict-task", "fix the failing add test")?;
        let mut conflict_trace = ExecutionTrace::new(&conflict_task.id, SESSION_ID, "conflict");
        let conflict_response = runtime.block_completion_closeout(
            &mut conflict_session,
            &mut conflict_task,
            &mut conflict_trace,
        )?;
        assert!(conflict_response.is_some());
        assert!(conflict_task.context.completion_verification_projection()?.is_some_and(
            |projection| {
                projection.completion_verification_findings.iter().any(|finding| {
                    finding.required_action == CompletionRequiredAction::ResolveConflict
                })
            }
        ));

        let skip_workspace = TestWorkspace::new("boundline-finalization-skip")?;
        fs::create_dir_all(skip_workspace.as_path().join(".boundline"))?;
        fs::write(
            skip_workspace.as_path().join(".boundline/execution.json"),
            serde_json::to_string_pretty(&WorkspaceExecutionProfile {
                name: "skip".to_string(),
                read_targets: vec!["src/lib.rs".to_string()],
                validation_command: ExecutionCommand {
                    program: "custom-verify".to_string(),
                    args: vec!["workspace".to_string()],
                },
                attempts: Vec::new(),
                adaptive: None,
                limits: RunLimits::default(),
                governance: None,
                review: None,
                legacy_source: None,
            })?,
        )?;
        let runtime = SessionRuntime::for_workspace(skip_workspace.as_path());
        let mut skip_session = sample_session(skip_workspace.as_path(), "fix the failing add test");
        skip_session.goal_plan = Some(goal_plan_with_expected_outcome("tests are passing")?);
        let mut skip_task =
            sample_task(skip_workspace.as_path(), "skip-task", "fix the failing add test")?;
        let mut skip_trace = ExecutionTrace::new(&skip_task.id, SESSION_ID, "skip");
        let skip_response = runtime.block_completion_closeout(
            &mut skip_session,
            &mut skip_task,
            &mut skip_trace,
        )?;
        assert!(skip_response.is_none());

        Ok(())
    }

    fn sample_session(workspace: &Path, goal: &str) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: SESSION_ID.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some(goal.to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: UPDATED_AT,
            updated_at: UPDATED_AT,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        }
    }

    fn sample_task(workspace: &Path, task_id: &str, goal: &str) -> Result<Task, Box<dyn Error>> {
        let request = crate::domain::task::TaskRunRequest {
            goal: goal.to_string(),
            input: json!({}),
            session_id: SESSION_ID.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let plan = Plan::new(vec![Step::agent(STEP_ID, "planner", json!({"goal": goal}))?])?;
        Ok(Task::new(task_id, &request, plan)?)
    }

    fn goal_plan_with_expected_outcome(expected_outcome: &str) -> Result<GoalPlan, Box<dyn Error>> {
        Ok(GoalPlan::new(
            "fix the failing add test",
            vec![PlannedTask {
                task_id: "planned-task".to_string(),
                description: "verify completion".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some(expected_outcome.to_string()),
                decision_type_hint: None,
                depends_on: None,
            }],
        )?)
    }

    fn bug_fix_claim() -> CompletionClaim {
        CompletionClaim {
            claim_id: "claim-bug-fix".to_string(),
            kind: CompletionClaimKind::BugFixed,
            scope: CompletionVerificationScope::Task,
            source: CompletionClaimSource::RuntimeInference,
            confidence: Some(ClaimInferenceConfidence::High),
            summary: "bug fix is complete".to_string(),
            supporting_signals: vec!["goal_text".to_string(), "changed_files".to_string()],
        }
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
