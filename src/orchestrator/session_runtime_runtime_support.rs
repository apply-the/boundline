use super::*;

pub(super) fn effective_assistant_runtimes(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> Vec<RuntimeKind> {
    workspace
        .filter(|config| !config.assistant_runtimes.is_empty())
        .map(|config| config.assistant_runtimes.clone())
        .or_else(|| {
            cluster
                .filter(|config| !config.assistant_runtimes.is_empty())
                .map(|config| config.assistant_runtimes.clone())
        })
        .or_else(|| {
            global
                .filter(|config| !config.assistant_runtimes.is_empty())
                .map(|config| config.assistant_runtimes.clone())
        })
        .unwrap_or_default()
}

pub(super) fn cluster_task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Exhausted => "exhausted",
        TaskStatus::Aborted => "aborted",
    }
}

pub(super) fn cluster_workspace_is_blocked(workspace_ref: &str) -> bool {
    let workspace = Path::new(workspace_ref);
    let Ok(profile) = load_workspace_execution_profile(workspace) else {
        return true;
    };

    !profile.attempts.iter().any(|attempt| {
        attempt.changes.iter().all(|change| {
            let Ok(contents) = std::fs::read_to_string(workspace.join(&change.path)) else {
                return false;
            };
            contents.contains(&change.find) || contents.contains(&change.replace)
        })
    })
}

pub(super) fn canon_workspace_scope_mismatch_reason(workspace: &Path) -> Option<String> {
    let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());
    let git_root = nearest_git_root(&workspace)?;
    if git_root == workspace {
        return None;
    }

    Some(format!(
        "planning governance requires a Canon workspace root, but Canon would target git root {} instead of workspace {}; use the repository root as the Boundline workspace or initialize a dedicated nested repository first",
        git_root.display(),
        workspace.display()
    ))
}

fn nearest_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub(super) fn git_config_value(workspace: &Path, key: &str) -> Option<String> {
    let output =
        Command::new("git").current_dir(workspace).args(["config", "--get", key]).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

pub(super) fn session_audit_outcome_for_status(status: SessionStatus) -> SessionAuditOutcome {
    match status {
        SessionStatus::Initialized => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Recorded, "session initialized")
        }
        SessionStatus::GoalCaptured => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Recorded,
            "goal captured for active session",
        ),
        SessionStatus::Planned => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "session planned")
        }
        SessionStatus::Blocked => {
            let mut outcome =
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Blocked, "session blocked");
            outcome.blocking = true;
            outcome
        }
        SessionStatus::Running => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session running")
        }
        SessionStatus::Succeeded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Succeeded, "session succeeded")
        }
        SessionStatus::Failed => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session failed")
        }
        SessionStatus::Exhausted => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Failed,
            "session exhausted its execution budget",
        ),
        SessionStatus::Aborted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session aborted")
        }
        SessionStatus::Invalid => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session invalid")
        }
    }
}

pub(super) fn trace_event_audit_algorithm(event_type: TraceEventType) -> SessionAuditAlgorithm {
    match event_type {
        TraceEventType::GoalPlanCreated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Plan,
            "goal_planner",
            "build_goal_plan_with_sources",
        ),
        TraceEventType::FlowInferred => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Plan, "session_runtime", "plan_goal_plan")
        }
        TraceEventType::ProjectScalePathProposed
        | TraceEventType::ProjectScaleStageTransitioned => SessionAuditAlgorithm::new(
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
        ),
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
        ),
        TraceEventType::ReviewCouncilAssembled => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_council",
            "resolve_council_assembly",
        ),
        TraceEventType::ReviewStopSemanticsRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_governance",
            "active_review_stop_semantics",
        ),
        TraceEventType::ReviewVoteResolved | TraceEventType::VotingDecisionRecorded => {
            SessionAuditAlgorithm::new(
                SessionAuditPhase::Review,
                "review_vote",
                "VoteRuleDefinition::resolve",
            )
        }
        TraceEventType::ReviewStarted
        | TraceEventType::ReviewerStarted
        | TraceEventType::ReviewerCompleted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewAdjudicated
        | TraceEventType::ReviewTerminalRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
        ),
        TraceEventType::ReasoningProfileActivated
        | TraceEventType::ReasoningParticipantStarted
        | TraceEventType::ReasoningParticipantCompleted
        | TraceEventType::ReasoningDisagreementRecorded
        | TraceEventType::ReasoningDebateRoundCompleted
        | TraceEventType::ReasoningReflexionRevisionCompleted
        | TraceEventType::ReasoningAdjudicationRecorded
        | TraceEventType::ReasoningConfidenceRecorded
        | TraceEventType::ReasoningProfileBlocked
        | TraceEventType::ReasoningProfileInterrupted
        | TraceEventType::ReasoningProfileEscalated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        TraceEventType::GovernanceDecisionRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Governance,
            "governance",
            "build_autopilot_decision",
        ),
        TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::GovernanceCompleted
        | TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected => SessionAuditAlgorithm::new(
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
        ),
        TraceEventType::RetryScheduled
        | TraceEventType::StageRetryScheduled
        | TraceEventType::Replanned
        | TraceEventType::StageReplanned
        | TraceEventType::StageFailed => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Recovery, "recovery", "decide_recovery")
        }
        TraceEventType::CheckpointCreated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "checkpoint",
            "prepare_checkpoint_for_mutation",
        ),
        TraceEventType::TerminalRecorded => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Run, "session_runtime", "finalize_task")
        }
        TraceEventType::TaskStarted
        | TraceEventType::FlowSelected
        | TraceEventType::StageTransitioned
        | TraceEventType::StepStarted
        | TraceEventType::StepCompleted => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Run, "session_runtime", "advance_task")
        }
    }
}

pub(super) fn trace_event_audit_outcome(event: &TraceEvent) -> SessionAuditOutcome {
    match event.event_type {
        TraceEventType::TaskStarted
        | TraceEventType::FlowSelected
        | TraceEventType::GoalPlanCreated
        | TraceEventType::FlowInferred
        | TraceEventType::ProjectScalePathProposed
        | TraceEventType::StageTransitioned
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceDecisionRecorded
        | TraceEventType::ReviewStarted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewerStarted
        | TraceEventType::ReviewStopSemanticsRecorded
        | TraceEventType::StepStarted
        | TraceEventType::DecisionCreated
        | TraceEventType::ReasoningProfileActivated
        | TraceEventType::ReasoningParticipantStarted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "activity started")
        }
        TraceEventType::DecisionDispatched
        | TraceEventType::CheckpointCreated
        | TraceEventType::VotingDecisionRecorded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Recorded, "activity recorded")
        }
        TraceEventType::DecisionVerified | TraceEventType::GovernanceCompleted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Succeeded, "activity succeeded")
        }
        TraceEventType::StepCompleted
        | TraceEventType::ReviewerCompleted
        | TraceEventType::ReviewCouncilAssembled
        | TraceEventType::ReviewVoteResolved
        | TraceEventType::ReviewAdjudicated
        | TraceEventType::ReviewTerminalRecorded
        | TraceEventType::DecisionRecovered
        | TraceEventType::ReasoningParticipantCompleted
        | TraceEventType::ReasoningDisagreementRecorded
        | TraceEventType::ReasoningDebateRoundCompleted
        | TraceEventType::ReasoningReflexionRevisionCompleted
        | TraceEventType::ReasoningAdjudicationRecorded
        | TraceEventType::ReasoningConfidenceRecorded
        | TraceEventType::ProjectScaleStageTransitioned
        | TraceEventType::TerminalRecorded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "activity completed")
        }
        TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::ReasoningProfileInterrupted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Awaiting, "awaiting follow-up")
        }
        TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected
        | TraceEventType::ReasoningProfileBlocked => {
            let mut outcome = SessionAuditOutcome::new(
                SessionAuditOutcomeStatus::Blocked,
                trace_event_summary(event),
            );
            outcome.blocking = true;
            outcome
        }
        TraceEventType::DecisionFailed | TraceEventType::ReasoningProfileEscalated => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, trace_event_summary(event))
        }
        TraceEventType::RetryScheduled | TraceEventType::StageRetryScheduled => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Retried, trace_event_summary(event))
        }
        TraceEventType::Replanned | TraceEventType::StageReplanned => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Replanned,
            trace_event_summary(event),
        ),
        TraceEventType::StageFailed => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, trace_event_summary(event))
        }
    }
}

pub(super) fn trace_event_audit_message(event: &TraceEvent) -> String {
    let event_label = trace_event_type_text(event.event_type).replace('_', " ");
    let summary = trace_event_summary(event);
    if summary == event_label { event_label } else { format!("{event_label}: {summary}") }
}

fn trace_event_summary(event: &TraceEvent) -> String {
    payload_string(event.payload.get("summary"))
        .or_else(|| payload_string(event.payload.get("reason")))
        .or_else(|| payload_string(event.payload.get("message")))
        .or_else(|| payload_string(event.payload.get("headline")))
        .or_else(|| payload_string(event.payload.get("selection_summary")))
        .or_else(|| {
            payload_string(event.payload.get("stop_semantics"))
                .map(|stop_semantics| format!("stop semantics {stop_semantics}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("target")).map(|target| format!("target {target}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("stage_key"))
                .map(|stage_key| format!("stage {stage_key}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("reviewer_id"))
                .map(|reviewer_id| format!("reviewer {reviewer_id}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("participant_id"))
                .map(|participant_id| format!("participant {participant_id}"))
        })
        .unwrap_or_else(|| trace_event_type_text(event.event_type).replace('_', " "))
}

pub(super) fn trace_event_audit_actor(event: &TraceEvent) -> SessionAuditActor {
    match event.event_type {
        TraceEventType::ReviewerStarted | TraceEventType::ReviewerCompleted => {
            reviewer_audit_actor(&event.payload)
        }
        TraceEventType::ReviewAdjudicated => reviewer_audit_actor(&event.payload),
        TraceEventType::ReviewCouncilAssembled
        | TraceEventType::ReviewStopSemanticsRecorded
        | TraceEventType::ReviewVoteResolved => review_council_audit_actor(&event.payload),
        TraceEventType::ReasoningParticipantStarted
        | TraceEventType::ReasoningParticipantCompleted => {
            reasoning_participant_audit_actor(&event.payload)
        }
        TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceDecisionRecorded
        | TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::GovernanceCompleted
        | TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected => governance_audit_actor(&event.payload),
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => SessionAuditActor {
            kind: SessionAuditActorKind::Agent,
            id: "boundline-decision-loop".to_string(),
            display_name: Some("Boundline Decision Loop".to_string()),
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        },
        TraceEventType::ReviewStarted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewTerminalRecorded
        | TraceEventType::VotingDecisionRecorded => SessionAuditActor {
            kind: SessionAuditActorKind::Reviewer,
            id: "review-council".to_string(),
            display_name: Some("Review Council".to_string()),
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        },
        _ => SessionAuditActor::system("boundline"),
    }
}

fn reviewer_audit_actor(payload: &Value) -> SessionAuditActor {
    let reviewer_id = payload_string(payload.get("reviewer_id"))
        .unwrap_or_else(|| "unknown-reviewer".to_string());
    let reviewer_role = payload_string(payload.get("reviewer_role"));
    let reviewer_source = payload_string(payload.get("reviewer_source"));
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::Reviewer,
        id: reviewer_id.clone(),
        display_name: Some(reviewer_id),
        role: reviewer_role,
        runtime_kind: None,
        provider: None,
        route_slot: None,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };
    if let Some(source) = reviewer_source.as_deref() {
        apply_route_text_to_actor(&mut actor, source);
    }
    actor
}

fn review_council_audit_actor(payload: &Value) -> SessionAuditActor {
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::Reviewer,
        id: "review-council".to_string(),
        display_name: Some("Review Council".to_string()),
        role: None,
        runtime_kind: None,
        provider: None,
        route_slot: Some("review".to_string()),
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };

    let participants = payload
        .get("vote_resolution")
        .and_then(|value| value.get("participants"))
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerParticipation>>(value).ok())
        .unwrap_or_default();

    let completed_routes = participants
        .iter()
        .filter(|participant| participant.status == ReviewerParticipationStatus::Completed)
        .filter_map(|participant| participant.effective_route.as_deref())
        .map(str::trim)
        .filter(|route| !route.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    actor.participant_routes = completed_routes.clone();
    actor.mixed_routes = completed_routes.len() > 1;

    if let Some(route) = completed_routes.first() {
        apply_route_text_to_actor(&mut actor, route);
    }

    if actor.mixed_routes {
        actor.role = Some("multi-reviewer".to_string());
    }

    actor
}

fn reasoning_participant_audit_actor(payload: &Value) -> SessionAuditActor {
    let participant_id = payload_string(payload.get("participant_id"))
        .unwrap_or_else(|| "unknown-participant".to_string());
    let role = payload_string(payload.get("role"));
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::ReasoningParticipant,
        id: participant_id.clone(),
        display_name: Some(participant_id.clone()),
        role,
        runtime_kind: None,
        provider: None,
        route_slot: None,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };

    if let Some(record) = payload
        .get("reasoning_profile_record")
        .cloned()
        .and_then(|value| serde_json::from_value::<ProfileActivationRecord>(value).ok())
        && let Some(participant) = record
            .participants
            .iter()
            .find(|participant| participant.participant_id == participant_id)
    {
        actor.provider = participant.provider_family.clone();
        apply_route_text_to_actor(&mut actor, &participant.effective_route);
    }

    actor
}

fn governance_audit_actor(payload: &Value) -> SessionAuditActor {
    let runtime = payload_string(payload.get("runtime"))
        .or_else(|| payload_string(payload.get("selected_runtime")))
        .unwrap_or_else(|| "governance".to_string());
    let route_slot = payload_string(payload.get("stage_key"))
        .as_deref()
        .and_then(governance_route_slot_for_stage_key)
        .map(str::to_string);
    SessionAuditActor {
        kind: SessionAuditActorKind::GovernanceRuntime,
        id: runtime.clone(),
        display_name: Some(runtime.clone()),
        role: payload_string(payload.get("stage_key")),
        runtime_kind: Some(runtime),
        provider: payload_string(payload.get("runtime"))
            .or_else(|| payload_string(payload.get("selected_runtime"))),
        route_slot,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    }
}

fn governance_route_slot_for_stage_key(stage_key: &str) -> Option<&'static str> {
    let stage_key = stage_key.trim();
    if stage_key.is_empty() {
        return None;
    }

    if stage_key.starts_with("plan:") {
        return Some("planning");
    }

    Some("implementation")
}

fn apply_route_text_to_actor(actor: &mut SessionAuditActor, route_text: &str) {
    if let Some((route_slot, runtime, model)) = parse_three_segment_route(route_text) {
        actor.route_slot = Some(route_slot);
        actor.runtime_kind = Some(runtime.clone());
        actor.provider.get_or_insert(runtime);
        actor.model_name = Some(model);
        return;
    }

    if let Some((runtime, model)) = route_text.split_once('/') {
        let runtime = runtime.trim();
        let model = model.trim();
        if !runtime.is_empty() {
            actor.runtime_kind = Some(runtime.to_string());
            actor.provider.get_or_insert(runtime.to_string());
        }
        if !model.is_empty() {
            actor.model_name = Some(model.to_string());
        }
    }
}

fn parse_three_segment_route(route_text: &str) -> Option<(String, String, String)> {
    let mut parts = route_text.splitn(3, ':');
    let route_slot = parts.next()?.trim();
    let runtime = parts.next()?.trim();
    let model = parts.next()?.trim();
    if route_slot.is_empty() || runtime.is_empty() || model.is_empty() {
        return None;
    }
    Some((route_slot.to_string(), runtime.to_string(), model.to_string()))
}

fn payload_string(value: Option<&Value>) -> Option<String> {
    let value = value?;
    match value {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => serde_json::to_string(value).ok(),
    }
}

pub(super) fn trace_event_type_text(event_type: TraceEventType) -> String {
    serde_json::to_value(event_type)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

pub(super) fn default_planning_system_context(mode: CanonMode) -> SystemContextBinding {
    if mode.requires_existing_context() {
        SystemContextBinding::Existing
    } else {
        SystemContextBinding::New
    }
}

pub(super) fn parse_planning_system_context(raw: &str) -> Option<SystemContextBinding> {
    match raw.trim() {
        SYSTEM_CONTEXT_NEW_TEXT => Some(SystemContextBinding::New),
        SYSTEM_CONTEXT_EXISTING_TEXT => Some(SystemContextBinding::Existing),
        _ => None,
    }
}

pub(super) fn read_upstream_artifact_capped(packet_dir: &Path, file_name: &str) -> Option<String> {
    let path = packet_dir.join(file_name);
    let content = fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() <= UPSTREAM_EVIDENCE_MAX_CHARS {
        return Some(trimmed.to_string());
    }
    Some(truncate_with_ellipsis_marker(trimmed, UPSTREAM_EVIDENCE_MAX_CHARS))
}

fn truncate_with_ellipsis_marker(text: &str, max_chars: usize) -> String {
    let Some((end_index, _)) = text.char_indices().nth(max_chars) else {
        return text.to_string();
    };
    let mut truncated = text[..end_index].to_string();
    truncated.push_str("\n\n[truncated]");
    truncated
}

pub(super) fn execution_governance_read_targets(
    native_context: &TaskContext,
    fallback_targets: &[String],
) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for state_key in [LATEST_CHANGED_FILES_KEY, "changed_files"] {
        if let Some(changed_files) = native_context.state.get(state_key).and_then(Value::as_array) {
            for changed_file in changed_files.iter().filter_map(Value::as_str) {
                if !changed_file.trim().is_empty() {
                    targets.insert(changed_file.to_string());
                }
            }
        }
    }

    if targets.is_empty() {
        for target in fallback_targets {
            if !target.trim().is_empty() {
                targets.insert(target.clone());
            }
        }
    }

    targets.into_iter().collect()
}

pub(super) fn missing_planning_governance_field(
    mode: CanonMode,
    field: &'static str,
) -> SessionRuntimeError {
    SessionRuntimeError::GoalPlan(format!(
        "planning governance for Canon mode {} requires field '{field}'",
        mode.as_str()
    ))
}

pub(super) fn session_status_for_task_status(status: TaskStatus) -> SessionStatus {
    match status {
        TaskStatus::Planned => SessionStatus::Planned,
        TaskStatus::Running => SessionStatus::Running,
        TaskStatus::Succeeded => SessionStatus::Succeeded,
        TaskStatus::Failed => SessionStatus::Failed,
        TaskStatus::Exhausted => SessionStatus::Exhausted,
        TaskStatus::Aborted => SessionStatus::Aborted,
    }
}
