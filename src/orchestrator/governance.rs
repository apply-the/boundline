use crate::adapters::governance_runtime::{
    GovernanceBoundedContext, GovernanceInputDocument, GovernanceRuntimeResponse, ReusedPacketInput,
};
use crate::domain::brief::{AuthoredBriefBundle, GovernanceIntent};
use crate::domain::flow::{FlowStepMetadata, built_in_flow};
use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::domain::governance::{
    ApprovalState, AutopilotAction, AutopilotDecisionRecord, CanonEvidenceInspectSummary,
    CanonMode, CanonPossibleActionSummary, CanonRecommendedActionSummary, CompactedCanonMemory,
    GovernanceLifecycleState, GovernanceProfile, GovernanceRolloutProfile, GovernanceRuntimeKind,
    GovernanceRuntimeState, GovernanceStartupContext, GovernedStagePacket, GovernedStageRecord,
    MemoryCredibilityState, PacketReadiness, PacketReuseBinding, PacketReuseBindingReason,
    StageGovernancePolicy, candidate_canon_modes, resolve_governance_startup_posture,
    resolved_canon_mode, supported_canon_modes_for_stage,
};
use crate::domain::task_context::{
    LATEST_COMPACTED_CANON_MEMORY_KEY, LATEST_GOVERNANCE_APPROVAL_PROVENANCE_KEY,
    LATEST_GOVERNANCE_CONTRACT_LINES_KEY, LATEST_GOVERNANCE_DECISION_KEY,
    LATEST_GOVERNANCE_PACKET_KEY, LATEST_GOVERNANCE_PACKET_REUSE_KEY, LATEST_GOVERNANCE_REASON_KEY,
    LATEST_GOVERNANCE_ROLLOUT_PROFILE_KEY, LATEST_GOVERNANCE_RUNTIME_STATE_KEY,
    LATEST_GOVERNANCE_STAGE_KEY, TaskContext, TaskContextError,
};

const GOVERNANCE_DOCUMENT_KIND_AUTHORED_BRIEF: &str = "authored-brief";
const GOVERNANCE_DOCUMENT_KIND_CLARIFICATION_ANSWER: &str = "clarification-answer";
const GOVERNANCE_DOCUMENT_KIND_PROJECT_MEMORY_EVIDENCE: &str = "project-memory-evidence";
const GOVERNANCE_DOCUMENT_KIND_PROJECT_MEMORY_SURFACE: &str = "project-memory-surface";
const GOVERNANCE_DOCUMENT_KIND_STAGE_BRIEF: &str = "stage-brief";
const GOVERNANCE_STAGE_IMPLEMENT: &str = "implement";
const GOVERNANCE_STAGE_VERIFY: &str = "verify";
const AUTHORITY_CONTRACT_UNAVAILABLE_REASON_CODE: &str = "authority_contract_unavailable";
const ADAPTIVE_CONTRACT_UNAVAILABLE_REASON_CODE: &str = "adaptive_contract_unavailable";
const AUTHORITY_HARD_STOP_REASON_CODE: &str = "authority_hard_stop";

pub fn governance_stage_key(flow_name: &str, stage_id: &str) -> String {
    format!("{}:{}", flow_name, stage_id)
}

pub fn selected_stage_policy(
    governance: Option<&GovernanceProfile>,
    flow_name: &str,
    stage_id: &str,
) -> Option<StageGovernancePolicy> {
    governance.and_then(|profile| profile.stage_policy(flow_name, stage_id).cloned())
}

pub fn requested_governance_intent(task_input: &Value) -> Option<GovernanceIntent> {
    task_input
        .get("governance_intent")
        .cloned()
        .or_else(|| {
            task_input
                .get("authored_brief")
                .and_then(|bundle| bundle.get("governance_intent"))
                .cloned()
        })
        .and_then(|value| serde_json::from_value(value).ok())
}

pub fn overlay_stage_policy_with_intent(
    policy: &StageGovernancePolicy,
    intent: Option<&GovernanceIntent>,
) -> StageGovernancePolicy {
    let Some(intent) = intent else {
        return policy.clone();
    };

    let mut policy = policy.clone();
    policy.enabled = true;

    if let Some(runtime_preference) = intent.runtime_preference {
        policy.runtime = Some(runtime_preference);
        if runtime_preference == GovernanceRuntimeKind::Canon {
            policy.required = true;
        }
    }
    if let Some(risk) = intent.risk.as_ref() {
        policy.risk = Some(risk.clone());
    }
    if let Some(zone) = intent.zone.as_ref() {
        policy.zone = Some(zone.clone());
    }
    if let Some(owner) = intent.owner.as_ref() {
        policy.owner = Some(owner.clone());
    }

    policy
}

pub fn governance_input_documents(
    task_input: &Value,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
) -> Vec<GovernanceInputDocument> {
    let mut documents = Vec::new();
    if let Some(bundle) = task_input
        .get("authored_brief")
        .cloned()
        .and_then(|value| serde_json::from_value::<AuthoredBriefBundle>(value).ok())
    {
        let mut stage_brief_assigned = false;
        for source in bundle.sources {
            let Some(path) = source.workspace_path else {
                continue;
            };
            let kind = if stage_brief_assigned {
                GOVERNANCE_DOCUMENT_KIND_AUTHORED_BRIEF
            } else {
                stage_brief_assigned = true;
                GOVERNANCE_DOCUMENT_KIND_STAGE_BRIEF
            };
            documents.push(GovernanceInputDocument { kind: kind.to_string(), path });
        }

        // T040: Include clarification answers as input documents.
        if let Some(clarification) = bundle
            .clarification
            .as_ref()
            .filter(|c| c.status == crate::domain::task::ClarificationStatus::Answered)
        {
            documents.push(GovernanceInputDocument {
                kind: GOVERNANCE_DOCUMENT_KIND_CLARIFICATION_ANSWER.to_string(),
                path: format!("clarification-{}", clarification.clarification_id),
            });
        }
    }

    append_project_memory_input_documents(&mut documents, compacted_canon_memory);

    documents
}

pub fn planning_governance_input_documents(
    authored_brief: Option<&AuthoredBriefBundle>,
    stage_brief_ref: &str,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
) -> Vec<GovernanceInputDocument> {
    let mut documents = vec![GovernanceInputDocument {
        kind: GOVERNANCE_DOCUMENT_KIND_STAGE_BRIEF.to_string(),
        path: stage_brief_ref.to_string(),
    }];

    if let Some(bundle) = authored_brief {
        for source in &bundle.sources {
            let Some(path) = source.workspace_path.clone() else {
                continue;
            };
            documents.push(GovernanceInputDocument {
                kind: GOVERNANCE_DOCUMENT_KIND_AUTHORED_BRIEF.to_string(),
                path,
            });
        }

        if let Some(clarification) = bundle.clarification.as_ref().filter(|clarification| {
            clarification.status == crate::domain::task::ClarificationStatus::Answered
        }) {
            documents.push(GovernanceInputDocument {
                kind: GOVERNANCE_DOCUMENT_KIND_CLARIFICATION_ANSWER.to_string(),
                path: format!("clarification-{}", clarification.clarification_id),
            });
        }
    }

    append_project_memory_input_documents(&mut documents, compacted_canon_memory);
    documents
}

fn append_project_memory_input_documents(
    documents: &mut Vec<GovernanceInputDocument>,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
) {
    let Some(memory) = compacted_canon_memory else {
        return;
    };

    let provenance_links = memory
        .evidence_summary
        .as_ref()
        .filter(|summary| !summary.artifact_provenance_links.is_empty())
        .map(|summary| summary.artifact_provenance_links.as_slice())
        .unwrap_or(memory.artifact_refs.as_slice());

    for path in provenance_links {
        let kind = if path.starts_with("docs/project/") {
            GOVERNANCE_DOCUMENT_KIND_PROJECT_MEMORY_SURFACE
        } else if path.starts_with("docs/evidence/") {
            GOVERNANCE_DOCUMENT_KIND_PROJECT_MEMORY_EVIDENCE
        } else {
            continue;
        };

        if documents.iter().any(|document| document.path == *path) {
            continue;
        }

        documents.push(GovernanceInputDocument { kind: kind.to_string(), path: path.clone() });
    }
}

pub fn select_packet_reuse_binding(
    context: &TaskContext,
    metadata: &FlowStepMetadata,
) -> Result<Option<PacketReuseBinding>, GovernanceStateSelectionError> {
    let Some(stage_record) = context
        .latest_governance_stage()
        .map_err(GovernanceStateSelectionError::from_task_context)?
    else {
        return Ok(None);
    };
    let Some(packet) = context
        .latest_governance_packet()
        .map_err(GovernanceStateSelectionError::from_task_context)?
    else {
        return Ok(None);
    };
    if packet.readiness != PacketReadiness::Reusable {
        return Ok(None);
    }

    let downstream_stage_key = governance_stage_key(&metadata.flow_name, &metadata.stage_id);
    if stage_record.stage_key == downstream_stage_key {
        return Ok(Some(PacketReuseBinding {
            upstream_stage_key: stage_record.stage_key,
            downstream_stage_key,
            packet_ref: packet.packet_ref,
            binding_reason: PacketReuseBindingReason::SameStageRerun,
        }));
    }

    if metadata.stage_index == 0 {
        return Ok(None);
    }

    let Some(previous_stage) = built_in_flow(&metadata.flow_name)
        .and_then(|flow| flow.stage(metadata.stage_index.saturating_sub(1)))
    else {
        return Ok(None);
    };
    let upstream_stage_key = governance_stage_key(&metadata.flow_name, previous_stage.id);
    if stage_record.stage_key != upstream_stage_key {
        return Ok(None);
    }

    Ok(Some(PacketReuseBinding {
        upstream_stage_key,
        downstream_stage_key,
        packet_ref: packet.packet_ref,
        binding_reason: PacketReuseBindingReason::UpstreamStageContext,
    }))
}

pub fn bounded_reused_packets(
    context: &TaskContext,
    metadata: &FlowStepMetadata,
) -> Result<Vec<ReusedPacketInput>, GovernanceStateSelectionError> {
    let Some(binding) = select_packet_reuse_binding(context, metadata)? else {
        return Ok(Vec::new());
    };
    let Some(packet) = context
        .latest_governance_packet()
        .map_err(GovernanceStateSelectionError::from_task_context)?
    else {
        return Ok(Vec::new());
    };

    Ok(vec![ReusedPacketInput {
        stage_key: binding.upstream_stage_key,
        packet_ref: binding.packet_ref,
        headline: packet.headline,
    }])
}

pub fn bounded_governance_context(
    context: &TaskContext,
    metadata: &FlowStepMetadata,
    read_targets: &[String],
) -> Result<(GovernanceBoundedContext, Option<PacketReuseBinding>), GovernanceStateSelectionError> {
    let packet_reuse = select_packet_reuse_binding(context, metadata)?;
    let reused_packets = bounded_reused_packets(context, metadata)?;

    Ok((
        GovernanceBoundedContext {
            read_targets: read_targets.to_vec(),
            stage_brief_ref: None,
            reused_packets,
        },
        packet_reuse,
    ))
}

/// T041: Enrich a bounded governance context with reused_packets from
/// `GovernedSessionLifecycle.accumulated_context` entries not already present.
pub fn enrich_bounded_context_with_accumulated(
    bounded_context: &mut GovernanceBoundedContext,
    accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
) {
    use std::collections::HashSet;
    let existing_refs: HashSet<String> =
        bounded_context.reused_packets.iter().map(|p| p.packet_ref.clone()).collect();

    for doc_ref in accumulated_context {
        if doc_ref.readiness != PacketReadiness::Reusable {
            continue;
        }
        if existing_refs.contains(&doc_ref.packet_ref) {
            continue;
        }
        bounded_context.reused_packets.push(ReusedPacketInput {
            stage_key: doc_ref.stage_key.clone(),
            packet_ref: doc_ref.packet_ref.clone(),
            headline: format!("governed {:?} from {}", doc_ref.canon_mode, doc_ref.stage_key),
        });
    }
}

/// T042: Create a `GovernedDocumentRef` from a Canon governance response.
///
/// Returns `Some` when the response contains a reusable packet; otherwise `None`.
pub fn governed_document_ref_from_response(
    stage_key: &str,
    canon_mode: CanonMode,
    response: &GovernanceRuntimeResponse,
) -> Option<crate::domain::governance::GovernedDocumentRef> {
    let packet = response.packet.as_ref()?;
    if packet.readiness != PacketReadiness::Reusable {
        return None;
    }
    Some(crate::domain::governance::GovernedDocumentRef {
        stage_key: stage_key.to_string(),
        canon_mode,
        packet_ref: packet.packet_ref.clone(),
        document_path: packet.document_refs.first().cloned(),
        readiness: packet.readiness,
    })
}

/// T042: Append a governed document reference to the session's governance lifecycle.
///
/// No-op if the session has no lifecycle or the doc_ref is `None`.
pub fn append_governed_document_to_lifecycle(
    session: &mut crate::domain::session::ActiveSessionRecord,
    doc_ref: Option<crate::domain::governance::GovernedDocumentRef>,
) {
    let Some(doc_ref) = doc_ref else { return };
    let Some(lifecycle) = session.governance_lifecycle.as_mut() else { return };
    lifecycle.accumulated_context.push(doc_ref);
}

/// T043: Extract a clarification prompt from a Canon response with non-success state.
///
/// Returns `Some(prompt_text)` when the Canon response indicates incomplete
/// input or pending mode selection that the operator should resolve.
pub fn clarification_prompt_from_response(response: &GovernanceRuntimeResponse) -> Option<String> {
    match response.status {
        GovernanceLifecycleState::PendingSelection => {
            Some(format!("Canon requires mode selection: {}", response.message))
        }
        GovernanceLifecycleState::Incomplete => {
            let missing_sections = response
                .packet
                .as_ref()
                .map(|p| &p.missing_sections)
                .filter(|sections| !sections.is_empty());

            Some(match missing_sections {
                Some(sections) => format!(
                    "Canon document is incomplete. Missing sections: {}. {}",
                    sections.join(", "),
                    response.message
                ),
                None => format!("Canon document is incomplete. {}", response.message),
            })
        }
        GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed => {
            let missing_sections = response
                .packet
                .as_ref()
                .filter(|p| p.readiness == PacketReadiness::Incomplete)
                .map(|p| &p.missing_sections)
                .filter(|sections| !sections.is_empty());

            missing_sections.map(|sections| {
                format!(
                    "Canon document is incomplete. Missing sections: {}. {}",
                    sections.join(", "),
                    response.message
                )
            })
        }
        _ => None,
    }
}

/// T044: Determine if a Canon response indicates awaiting approval.
///
/// Returns `true` when the session lifecycle should be updated to reflect
/// an approval-pending state.
pub fn is_awaiting_approval_response(response: &GovernanceRuntimeResponse) -> bool {
    response.status == GovernanceLifecycleState::AwaitingApproval
}

/// T044: Update session governance lifecycle to approval-pending state.
pub fn set_lifecycle_awaiting_approval(
    session: &mut crate::domain::session::ActiveSessionRecord,
    response: &GovernanceRuntimeResponse,
) {
    let Some(lifecycle) = session.governance_lifecycle.as_mut() else { return };
    lifecycle.terminal_reason = Some(format!("awaiting approval: {}", response.message));
}

/// T045: Determine if a session lifecycle is in awaiting-approval state
/// and should use `refresh` instead of `start`.
pub fn lifecycle_requires_refresh(session: &crate::domain::session::ActiveSessionRecord) -> bool {
    session
        .governance_lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.terminal_reason.as_deref())
        .is_some_and(|reason| reason.starts_with("awaiting approval"))
}

pub fn runtime_command_available(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains('/') {
        return std::path::Path::new(trimmed).is_file();
    }

    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|directory| directory.join(trimmed).is_file())
    })
}

pub fn narrowed_bounded_context(
    bounded_context: &GovernanceBoundedContext,
) -> Option<GovernanceBoundedContext> {
    if bounded_context.read_targets.len() > 1 {
        let mut narrowed = bounded_context.clone();
        narrowed.read_targets.pop();
        return Some(narrowed);
    }

    if !bounded_context.reused_packets.is_empty() {
        let mut narrowed = bounded_context.clone();
        narrowed.reused_packets.pop();
        return Some(narrowed);
    }

    None
}

pub fn escalation_target_stage_key(
    metadata: &FlowStepMetadata,
    action: AutopilotAction,
) -> Option<String> {
    match action {
        AutopilotAction::EscalateVerification => {
            let flow = built_in_flow(&metadata.flow_name)?;
            let next_stage = flow.stage(metadata.stage_index.saturating_add(1))?;
            Some(governance_stage_key(&metadata.flow_name, next_stage.id))
        }
        AutopilotAction::EscalatePrReview => {
            Some(governance_stage_key(&metadata.flow_name, &metadata.stage_id))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_autopilot_decision(
    governance_attempt_id: &str,
    policy: &StageGovernancePolicy,
    default_runtime: GovernanceRuntimeKind,
    metadata: &FlowStepMetadata,
    bounded_context: &GovernanceBoundedContext,
    lifecycle_state: Option<GovernanceLifecycleState>,
    approval_state: Option<ApprovalState>,
    packet_readiness: Option<PacketReadiness>,
) -> Option<AutopilotDecisionRecord> {
    if !policy.autopilot {
        return None;
    }

    let stage_key = governance_stage_key(&metadata.flow_name, &metadata.stage_id);
    let candidate_modes = candidate_canon_modes(policy, default_runtime);
    let resolved_mode = resolved_canon_mode(policy, default_runtime);
    let approval_requested = matches!(approval_state, Some(ApprovalState::Requested));
    let packet_issue =
        matches!(packet_readiness, Some(PacketReadiness::Incomplete | PacketReadiness::Rejected))
            || matches!(
                lifecycle_state,
                Some(GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed)
            );
    let narrowed_context =
        packet_issue.then(|| narrowed_bounded_context(bounded_context)).flatten();
    let supports_pr_review =
        supported_canon_modes_for_stage(&metadata.flow_name, &metadata.stage_id)
            .contains(&CanonMode::PrReview);

    let mut candidate_actions = Vec::new();
    if approval_requested {
        candidate_actions.push(AutopilotAction::AwaitApproval);
    }
    if !candidate_modes.is_empty() {
        candidate_actions.push(AutopilotAction::SelectMode);
    }
    if narrowed_context.is_some() {
        candidate_actions.push(AutopilotAction::RetryStageWithNarrowedContext);
    }
    if candidate_modes.is_empty() && !approval_requested {
        if metadata.stage_id == GOVERNANCE_STAGE_IMPLEMENT {
            candidate_actions.push(AutopilotAction::EscalateVerification);
        } else if metadata.stage_id == GOVERNANCE_STAGE_VERIFY
            && supports_pr_review
            && resolved_mode != Some(CanonMode::PrReview)
        {
            candidate_actions.push(AutopilotAction::EscalatePrReview);
        }
    }

    let (selected_action, selected_mode, selected_target_stage_key, rationale, blocked_reason) =
        if approval_requested {
            (
                Some(AutopilotAction::AwaitApproval),
                resolved_mode,
                None,
                format!("autopilot is waiting for approval for {stage_key}"),
                None,
            )
        } else if let Some(mode) = candidate_modes.first().copied() {
            (
                Some(AutopilotAction::SelectMode),
                Some(mode),
                None,
                format!("autopilot selected Canon mode {:?} for {stage_key}", mode),
                None,
            )
        } else if narrowed_context.is_some() {
            (
                Some(AutopilotAction::RetryStageWithNarrowedContext),
                resolved_mode,
                None,
                format!("autopilot narrowed the bounded context for {stage_key}"),
                None,
            )
        } else if metadata.stage_id == GOVERNANCE_STAGE_IMPLEMENT {
            let target =
                escalation_target_stage_key(metadata, AutopilotAction::EscalateVerification);
            (
                Some(AutopilotAction::EscalateVerification),
                resolved_mode,
                target,
                format!("autopilot escalated {stage_key} toward verification governance"),
                None,
            )
        } else if metadata.stage_id == GOVERNANCE_STAGE_VERIFY
            && supports_pr_review
            && resolved_mode != Some(CanonMode::PrReview)
        {
            let target = escalation_target_stage_key(metadata, AutopilotAction::EscalatePrReview);
            (
                Some(AutopilotAction::EscalatePrReview),
                resolved_mode,
                target,
                format!("autopilot escalated {stage_key} toward pr-review governance"),
                None,
            )
        } else {
            if policy.required {
                candidate_actions.push(AutopilotAction::BlockStage);
            }
            let blocked_reason = policy
                .required
                .then(|| format!("no compliant governance continuation exists for {stage_key}"));
            (
                None,
                resolved_mode,
                None,
                blocked_reason.clone().unwrap_or_else(|| {
                    format!("autopilot found no additional governance action for {stage_key}")
                }),
                blocked_reason,
            )
        };

    Some(AutopilotDecisionRecord {
        decision_id: format!("{governance_attempt_id}-decision"),
        stage_key,
        candidate_actions,
        candidate_modes,
        selected_action,
        selected_mode,
        selected_target_stage_key,
        rationale,
        blocked_reason,
    })
}

pub fn default_stage_canon_mode(
    policy: &StageGovernancePolicy,
    default_runtime: GovernanceRuntimeKind,
) -> Option<CanonMode> {
    candidate_canon_modes(policy, default_runtime).into_iter().next()
}

pub fn governance_state_patch(
    record: &GovernedStageRecord,
    packet: Option<&GovernedStagePacket>,
    packet_reuse: Option<&PacketReuseBinding>,
    decision: Option<&AutopilotDecisionRecord>,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
    projection: &GovernanceProjectionSnapshot,
) -> Result<Map<String, Value>, GovernanceStatePatchError> {
    let mut patch = Map::new();
    patch.insert(
        LATEST_GOVERNANCE_STAGE_KEY.to_string(),
        serialize_to_value(LATEST_GOVERNANCE_STAGE_KEY, record)?,
    );
    patch.insert(
        LATEST_GOVERNANCE_PACKET_KEY.to_string(),
        optional_serialized_value(LATEST_GOVERNANCE_PACKET_KEY, packet)?,
    );
    patch.insert(
        LATEST_GOVERNANCE_PACKET_REUSE_KEY.to_string(),
        optional_serialized_value(LATEST_GOVERNANCE_PACKET_REUSE_KEY, packet_reuse)?,
    );
    patch.insert(
        LATEST_GOVERNANCE_DECISION_KEY.to_string(),
        optional_serialized_value(LATEST_GOVERNANCE_DECISION_KEY, decision)?,
    );
    patch.insert(
        LATEST_COMPACTED_CANON_MEMORY_KEY.to_string(),
        optional_serialized_value(LATEST_COMPACTED_CANON_MEMORY_KEY, compacted_canon_memory)?,
    );
    patch.insert(
        LATEST_GOVERNANCE_RUNTIME_STATE_KEY.to_string(),
        serialize_to_value(LATEST_GOVERNANCE_RUNTIME_STATE_KEY, &projection.runtime_state)?,
    );
    patch.insert(
        LATEST_GOVERNANCE_ROLLOUT_PROFILE_KEY.to_string(),
        serialize_to_value(LATEST_GOVERNANCE_ROLLOUT_PROFILE_KEY, &projection.rollout_profile)?,
    );
    patch
        .insert(LATEST_GOVERNANCE_REASON_KEY.to_string(), Value::String(projection.reason.clone()));
    patch.insert(
        LATEST_GOVERNANCE_CONTRACT_LINES_KEY.to_string(),
        serialize_to_value(LATEST_GOVERNANCE_CONTRACT_LINES_KEY, &projection.contract_lines)?,
    );
    patch.insert(
        LATEST_GOVERNANCE_APPROVAL_PROVENANCE_KEY.to_string(),
        Value::String(projection.approval_provenance.clone()),
    );

    Ok(patch)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceProjectionSnapshot {
    pub runtime_state: GovernanceRuntimeState,
    pub rollout_profile: GovernanceRolloutProfile,
    pub reason: String,
    pub contract_lines: Vec<String>,
    pub approval_provenance: String,
}

pub fn governance_projection_snapshot(
    context: &TaskContext,
    stage_key: &str,
    packet: Option<&GovernedStagePacket>,
    approval_state: ApprovalState,
) -> Result<GovernanceProjectionSnapshot, GovernanceStatePatchError> {
    let current_state = current_runtime_state_for_stage(context, stage_key)?;
    let adaptive = supported_adaptive_envelope(packet);
    let requested_profile =
        adaptive.map(|envelope| GovernanceRolloutProfile::from(envelope.rollout_profile));
    let operator_approved_profile = matches!(approval_state, ApprovalState::Granted)
        .then_some(requested_profile)
        .flatten()
        .filter(|profile| *profile != GovernanceRolloutProfile::Minimal);
    let posture = resolve_governance_startup_posture(GovernanceStartupContext {
        current_state: current_state.unwrap_or(GovernanceRuntimeState::Advisory),
        requested_profile,
        operator_approved_profile,
        low_trust_surface: current_state.is_none(),
    });
    let runtime_state = adaptive
        .filter(|envelope| {
            envelope.rollout_profile
                == crate::domain::governance::CanonAdaptiveRolloutProfile::Minimal
        })
        .map(|envelope| GovernanceRuntimeState::from(envelope.governance_state))
        .unwrap_or(posture.runtime_state);

    Ok(GovernanceProjectionSnapshot {
        runtime_state,
        rollout_profile: posture.rollout_profile,
        reason: governance_projection_reason(adaptive, posture.rollout_profile, approval_state),
        contract_lines: governance_contract_lines(packet),
        approval_provenance: governance_approval_provenance(
            adaptive,
            posture.rollout_profile,
            approval_state,
        ),
    })
}

fn current_runtime_state_for_stage(
    context: &TaskContext,
    stage_key: &str,
) -> Result<Option<GovernanceRuntimeState>, GovernanceStatePatchError> {
    let latest_stage = context
        .latest_governance_stage()
        .map_err(|error| GovernanceStatePatchError::TaskContext(error.to_string()))?;
    if latest_stage.as_ref().map(|record| record.stage_key.as_str()) != Some(stage_key) {
        return Ok(None);
    }

    Ok(context
        .state
        .get(LATEST_GOVERNANCE_RUNTIME_STATE_KEY)
        .and_then(Value::as_str)
        .and_then(parse_runtime_state))
}

fn parse_runtime_state(value: &str) -> Option<GovernanceRuntimeState> {
    match value {
        "advisory" => Some(GovernanceRuntimeState::Advisory),
        "catch" => Some(GovernanceRuntimeState::Catch),
        "rule" => Some(GovernanceRuntimeState::Rule),
        "hook" => Some(GovernanceRuntimeState::Hook),
        _ => None,
    }
}

fn supported_adaptive_envelope(
    packet: Option<&GovernedStagePacket>,
) -> Option<&crate::domain::governance::CanonAdaptiveGovernanceV1Envelope> {
    packet
        .and_then(|packet| packet.adaptive_governance.as_ref())
        .filter(|adaptive| adaptive.is_supported_contract_line())
}

fn governance_projection_reason(
    adaptive: Option<&crate::domain::governance::CanonAdaptiveGovernanceV1Envelope>,
    rollout_profile: GovernanceRolloutProfile,
    approval_state: ApprovalState,
) -> String {
    if let Some(adaptive) = adaptive {
        if let Some(rationale) =
            adaptive.state_rationale.as_deref().filter(|value| !value.trim().is_empty())
        {
            return rationale.to_string();
        }
        if let Some(rationale) =
            adaptive.profile_rationale.as_deref().filter(|value| !value.trim().is_empty())
        {
            return rationale.to_string();
        }
        if rollout_profile == GovernanceRolloutProfile::Minimal
            && adaptive.rollout_profile
                == crate::domain::governance::CanonAdaptiveRolloutProfile::Minimal
        {
            return "startup posture seeded from adaptive companion".to_string();
        }
    }

    if matches!(approval_state, ApprovalState::Granted)
        && rollout_profile != GovernanceRolloutProfile::Minimal
    {
        return "startup posture activated approved adaptive companion".to_string();
    }

    "startup posture defaulted locally for low-trust surface".to_string()
}

fn governance_contract_lines(packet: Option<&GovernedStagePacket>) -> Vec<String> {
    let authority_line = packet
        .and_then(|packet| packet.authority_governance.as_ref())
        .map(|authority| format!("authority_contract_line: {}", authority.contract_line))
        .unwrap_or_else(|| "authority_contract_line: unavailable".to_string());
    let adaptive_line = supported_adaptive_envelope(packet)
        .map(|adaptive| format!("adaptive_contract_line: {}", adaptive.contract_line))
        .unwrap_or_else(|| "adaptive_contract_line: unavailable".to_string());

    vec![authority_line, adaptive_line]
}

fn governance_approval_provenance(
    adaptive: Option<&crate::domain::governance::CanonAdaptiveGovernanceV1Envelope>,
    rollout_profile: GovernanceRolloutProfile,
    approval_state: ApprovalState,
) -> String {
    if matches!(approval_state, ApprovalState::Requested) {
        return "stronger posture remained inactive because operator approval is still requested"
            .to_string();
    }

    if matches!(approval_state, ApprovalState::Granted)
        && adaptive.is_some()
        && rollout_profile != GovernanceRolloutProfile::Minimal
    {
        return "operator approval activated the requested stronger posture".to_string();
    }

    "approval not required".to_string()
}

pub fn compacted_canon_memory_from_response(
    stage_key: &str,
    runtime_kind: GovernanceRuntimeKind,
    response: &GovernanceRuntimeResponse,
) -> Option<CompactedCanonMemory> {
    if runtime_kind != GovernanceRuntimeKind::Canon
        && response.packet.as_ref().and_then(|packet| packet.canon_mode).is_none()
    {
        return None;
    }

    let artifact_refs = response
        .packet
        .as_ref()
        .map(|packet| {
            if packet.document_refs.is_empty() {
                packet.expected_document_refs.clone()
            } else {
                packet.document_refs.clone()
            }
        })
        .unwrap_or_default();
    let credibility = canon_memory_credibility(response.status, response.packet.as_ref());
    let recommended_next_action = canon_memory_recommended_action(response, credibility);
    let possible_actions = canon_memory_possible_actions(response, credibility);
    let authority_provenance_lines = response
        .packet
        .as_ref()
        .and_then(|packet| packet.authority_governance.as_ref())
        .map(|authority| authority.projection_lines())
        .unwrap_or_default();
    let adaptive_provenance_lines = response
        .packet
        .as_ref()
        .map(|packet| {
            packet
                .adaptive_governance
                .as_ref()
                .map(|adaptive| adaptive.projection_lines())
                .unwrap_or_else(|| vec!["adaptive_contract_line: unavailable".to_string()])
        })
        .unwrap_or_default();
    let semantic_provenance_lines = response
        .packet
        .as_ref()
        .map(|packet| {
            packet
                .semantic_descriptor
                .as_ref()
                .map(|semantic| semantic.projection_lines())
                .unwrap_or_else(|| vec!["semantic_contract_line: unavailable".to_string()])
        })
        .unwrap_or_default();

    Some(CompactedCanonMemory {
        headline: response
            .packet
            .as_ref()
            .map(|packet| packet.headline.clone())
            .unwrap_or_else(|| response.message.clone()),
        credibility,
        stage_key: Some(stage_key.to_string()),
        run_ref: response.run_ref.clone(),
        packet_ref: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
        reason_code: response.reason_code.clone(),
        artifact_refs: artifact_refs.clone(),
        mode_summary: None,
        possible_actions,
        recommended_next_action,
        evidence_summary: (!artifact_refs.is_empty()).then_some(CanonEvidenceInspectSummary {
            execution_posture: None,
            carried_forward_items: Vec::new(),
            artifact_provenance_links: artifact_refs,
            closure_status: None,
            closure_findings: Vec::new(),
        }),
        authority_provenance_lines,
        adaptive_provenance_lines,
        semantic_provenance_lines,
    })
}

pub fn compacted_canon_memory_for_block(
    stage_key: &str,
    runtime_kind: GovernanceRuntimeKind,
    reason: &str,
) -> Option<CompactedCanonMemory> {
    (runtime_kind == GovernanceRuntimeKind::Canon).then(|| CompactedCanonMemory {
        headline: reason.to_string(),
        credibility: MemoryCredibilityState::Insufficient,
        stage_key: Some(stage_key.to_string()),
        run_ref: None,
        packet_ref: None,
        reason_code: Some("blocked_context".to_string()),
        artifact_refs: Vec::new(),
        mode_summary: None,
        possible_actions: vec![CanonPossibleActionSummary {
            action: "refresh".to_string(),
            text: "refresh Canon governance context before retrying".to_string(),
            target: None,
        }],
        recommended_next_action: Some(CanonRecommendedActionSummary {
            action: "refresh".to_string(),
            rationale: "refresh Canon governance context before retrying".to_string(),
            target: None,
        }),
        evidence_summary: None,
        authority_provenance_lines: Vec::new(),
        adaptive_provenance_lines: Vec::new(),
        semantic_provenance_lines: Vec::new(),
    })
}

pub fn fail_closed_required_authority_response(
    stage_key: &str,
    policy: &StageGovernancePolicy,
    runtime_kind: GovernanceRuntimeKind,
    response: &GovernanceRuntimeResponse,
) -> Option<GovernanceRuntimeResponse> {
    if runtime_kind != GovernanceRuntimeKind::Canon
        || response.status != GovernanceLifecycleState::GovernedReady
    {
        return None;
    }

    if policy.required {
        let Some(authority) =
            response.packet.as_ref().and_then(|packet| packet.authority_governance.as_ref())
        else {
            let mut blocked = response.clone();
            blocked.status = GovernanceLifecycleState::Blocked;
            blocked.reason_code = Some(AUTHORITY_CONTRACT_UNAVAILABLE_REASON_CODE.to_string());
            blocked.message = format!(
                "Canon authority-governance-v1 metadata is missing for required stage {stage_key}"
            );
            return Some(blocked);
        };

        let (reason_code, message) = if !authority.is_supported_contract_line() {
            (
                AUTHORITY_CONTRACT_UNAVAILABLE_REASON_CODE,
                format!(
                    "Canon authority contract `{}` is unsupported for required stage {stage_key}",
                    authority.contract_line
                ),
            )
        } else if authority.requires_hard_stop() {
            (
                AUTHORITY_HARD_STOP_REASON_CODE,
                authority.hard_stop_reason().unwrap_or_else(|| {
                    format!(
                        "Canon authority-governance-v1 requires a hard stop for stage {stage_key}"
                    )
                }),
            )
        } else {
            ("", String::new())
        };

        if !reason_code.is_empty() {
            let mut blocked = response.clone();
            blocked.status = GovernanceLifecycleState::Blocked;
            blocked.reason_code = Some(reason_code.to_string());
            blocked.message = message;
            return Some(blocked);
        }
    }

    if !policy.require_adaptive_companion {
        return None;
    }

    let Some(packet) = response.packet.as_ref() else {
        let mut blocked = response.clone();
        blocked.status = GovernanceLifecycleState::Blocked;
        blocked.reason_code = Some(ADAPTIVE_CONTRACT_UNAVAILABLE_REASON_CODE.to_string());
        blocked.message = format!(
            "Canon adaptive-governance-v1 metadata is missing for required stage {stage_key}"
        );
        return Some(blocked);
    };

    let Some(adaptive) = packet.adaptive_governance.as_ref() else {
        let mut blocked = response.clone();
        blocked.status = GovernanceLifecycleState::Blocked;
        blocked.reason_code = Some(ADAPTIVE_CONTRACT_UNAVAILABLE_REASON_CODE.to_string());
        blocked.message = format!(
            "Canon adaptive-governance-v1 metadata is unavailable for required stage {stage_key}"
        );
        return Some(blocked);
    };

    if adaptive.is_supported_contract_line() {
        return None;
    }

    let mut blocked = response.clone();
    blocked.status = GovernanceLifecycleState::Blocked;
    blocked.reason_code = Some(ADAPTIVE_CONTRACT_UNAVAILABLE_REASON_CODE.to_string());
    blocked.message = format!(
        "Canon adaptive contract `{}` is unsupported for required stage {stage_key}",
        adaptive.contract_line
    );
    Some(blocked)
}

fn canon_memory_credibility(
    lifecycle_state: GovernanceLifecycleState,
    packet: Option<&GovernedStagePacket>,
) -> MemoryCredibilityState {
    let packet_readiness = packet.map(|packet| packet.readiness);
    if matches!(lifecycle_state, GovernanceLifecycleState::Failed)
        || matches!(packet_readiness, Some(PacketReadiness::Rejected))
    {
        return MemoryCredibilityState::Contradicted;
    }
    if matches!(lifecycle_state, GovernanceLifecycleState::Blocked)
        || matches!(packet_readiness, Some(PacketReadiness::Incomplete))
    {
        return MemoryCredibilityState::Stale;
    }
    if matches!(packet_readiness, Some(PacketReadiness::Reusable))
        || matches!(lifecycle_state, GovernanceLifecycleState::AwaitingApproval)
    {
        return MemoryCredibilityState::Credible;
    }

    MemoryCredibilityState::Insufficient
}

fn canon_memory_recommended_action(
    response: &GovernanceRuntimeResponse,
    credibility: MemoryCredibilityState,
) -> Option<CanonRecommendedActionSummary> {
    match credibility {
        MemoryCredibilityState::Credible
            if response.status == GovernanceLifecycleState::AwaitingApproval =>
        {
            Some(CanonRecommendedActionSummary {
                action: "approve".to_string(),
                rationale: response.message.clone(),
                target: response.run_ref.clone(),
            })
        }
        MemoryCredibilityState::Credible => Some(CanonRecommendedActionSummary {
            action: "inspect".to_string(),
            rationale: response.message.clone(),
            target: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
        }),
        MemoryCredibilityState::Stale | MemoryCredibilityState::Insufficient => {
            Some(CanonRecommendedActionSummary {
                action: "refresh".to_string(),
                rationale: response.message.clone(),
                target: response
                    .run_ref
                    .clone()
                    .or_else(|| response.packet.as_ref().map(|packet| packet.packet_ref.clone())),
            })
        }
        MemoryCredibilityState::Contradicted => Some(CanonRecommendedActionSummary {
            action: "replan".to_string(),
            rationale: response.message.clone(),
            target: response
                .run_ref
                .clone()
                .or_else(|| response.packet.as_ref().map(|packet| packet.packet_ref.clone())),
        }),
    }
}

fn canon_memory_possible_actions(
    response: &GovernanceRuntimeResponse,
    credibility: MemoryCredibilityState,
) -> Vec<CanonPossibleActionSummary> {
    match credibility {
        MemoryCredibilityState::Credible
            if response.status == GovernanceLifecycleState::AwaitingApproval =>
        {
            vec![CanonPossibleActionSummary {
                action: "approve".to_string(),
                text: "record the required approval before continuing".to_string(),
                target: response.run_ref.clone(),
            }]
        }
        MemoryCredibilityState::Credible => vec![CanonPossibleActionSummary {
            action: "inspect".to_string(),
            text: "inspect the current Canon packet before continuing".to_string(),
            target: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
        }],
        MemoryCredibilityState::Stale | MemoryCredibilityState::Insufficient => {
            vec![CanonPossibleActionSummary {
                action: "refresh".to_string(),
                text: "refresh the governed packet and reassess its credibility".to_string(),
                target: response.run_ref.clone(),
            }]
        }
        MemoryCredibilityState::Contradicted => vec![CanonPossibleActionSummary {
            action: "replan".to_string(),
            text: "replan because the prior Canon-grounded memory is contradicted".to_string(),
            target: response.run_ref.clone(),
        }],
    }
}

fn optional_serialized_value<T: Serialize>(
    key: &str,
    value: Option<&T>,
) -> Result<Value, GovernanceStatePatchError> {
    value.map_or(Ok(Value::Null), |value| serialize_to_value(key, value))
}

fn serialize_to_value<T: Serialize>(
    key: &str,
    value: &T,
) -> Result<Value, GovernanceStatePatchError> {
    serde_json::to_value(value).map_err(|error| GovernanceStatePatchError::Serialization {
        key: key.to_string(),
        message: error.to_string(),
    })
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GovernanceStatePatchError {
    #[error("failed to serialize governance state '{key}': {message}")]
    Serialization { key: String, message: String },
    #[error("failed to read governance state from task context: {0}")]
    TaskContext(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GovernanceStateSelectionError {
    #[error("failed to read governance state from task context: {0}")]
    TaskContext(String),
}

impl GovernanceStateSelectionError {
    fn from_task_context(error: TaskContextError) -> Self {
        Self::TaskContext(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceStepDecision<T> {
    Continue,
    Halt,
    Terminal(T),
}

use crate::domain::governance::CanonModeSelectionPreference;

/// Result of mode-selection gate evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeSelectionOutcome {
    /// Mode was explicitly provided by the operator.
    ExplicitMode(CanonMode),
    /// Mode inferred from evidence; confirmation recommended.
    InferredMode { mode: CanonMode, confirmation_prompt: String },
    /// Mode inferred and auto-accepted.
    AutoSelectedMode(CanonMode),
    /// No mode could be determined; manual selection required.
    PendingSelection { message: String },
}

/// Evaluate the mode-selection gate based on preference and explicit mode.
///
/// - `Manual` + no explicit mode → `PendingSelection`
/// - `AutoConfirm` + no explicit mode → `InferredMode` (placeholder; real inference TBD)
/// - `Auto` + no explicit mode → `AutoSelectedMode` (placeholder; real inference TBD)
/// - Any preference + explicit mode → `ExplicitMode`
pub fn evaluate_mode_selection_gate(
    preference: CanonModeSelectionPreference,
    explicit_mode: Option<CanonMode>,
) -> ModeSelectionOutcome {
    if let Some(mode) = explicit_mode {
        return ModeSelectionOutcome::ExplicitMode(mode);
    }

    match preference {
        CanonModeSelectionPreference::Manual => ModeSelectionOutcome::PendingSelection {
            message: "Canon mode-selection is manual; specify --mode <mode>".to_string(),
        },
        CanonModeSelectionPreference::AutoConfirm => {
            // Placeholder: real inference from evidence would happen here.
            ModeSelectionOutcome::InferredMode {
                mode: CanonMode::Change,
                confirmation_prompt: "Inferred Canon mode: change. Confirm with --mode change or specify a different mode.".to_string(),
            }
        }
        CanonModeSelectionPreference::Auto => {
            // Placeholder: real inference from evidence would happen here.
            ModeSelectionOutcome::AutoSelectedMode(CanonMode::Change)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::{Map, Value, json};

    use super::{
        ADAPTIVE_CONTRACT_UNAVAILABLE_REASON_CODE, AUTHORITY_CONTRACT_UNAVAILABLE_REASON_CODE,
        GovernanceBoundedContext, GovernanceStateSelectionError, bounded_reused_packets,
        build_autopilot_decision, canon_memory_credibility, canon_memory_possible_actions,
        canon_memory_recommended_action, clarification_prompt_from_response,
        compacted_canon_memory_for_block, compacted_canon_memory_from_response,
        fail_closed_required_authority_response, governance_input_documents,
        lifecycle_requires_refresh, optional_serialized_value, serialize_to_value,
        set_lifecycle_awaiting_approval,
    };
    use crate::adapters::governance_runtime::GovernanceInputDocument;
    use crate::adapters::governance_runtime::GovernanceRuntimeResponse;
    use crate::domain::flow::FlowStepMetadata;
    use crate::domain::governance::{
        ADAPTIVE_GOVERNANCE_V1_CONTRACT_LINE, AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE, ApprovalState,
        AutopilotAction, CanonAdaptiveGovernanceState, CanonAdaptiveGovernanceV1Envelope,
        CanonAdaptiveRolloutProfile, CanonAuthorityGovernanceV1Envelope, CanonAuthorityZone,
        CanonChangeClass, CanonEvidenceInspectSummary, CanonIntendedPersona,
        CanonModeSelectionPreference, CanonRecommendedActionSummary, CanonRiskClass,
        CanonSemanticArtifactDescriptorV1Envelope, CanonSemanticEligibilityState,
        CanonSemanticProvenanceBoundary, CompactedCanonMemory, GovernanceLifecycleState,
        GovernanceRuntimeKind, GovernedSessionLifecycle, GovernedStagePacket, GovernedStageRecord,
        MemoryCredibilityState, PacketReadiness, SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE,
        StageGovernancePolicy,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::task_context::TaskContext;
    use crate::domain::task_context::TaskContextError;

    fn response(
        status: GovernanceLifecycleState,
        message: &str,
        packet: Option<GovernedStagePacket>,
    ) -> GovernanceRuntimeResponse {
        GovernanceRuntimeResponse {
            status,
            approval_state: ApprovalState::NotNeeded,
            run_ref: Some("canon-run-1".to_string()),
            packet,
            reason_code: Some("packet_ready".to_string()),
            message: message.to_string(),
        }
    }

    fn packet(readiness: PacketReadiness) -> GovernedStagePacket {
        GovernedStagePacket {
            packet_ref: ".canon/runs/canon-run-1".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: None,
            expected_document_refs: vec![".canon/runs/canon-run-1/verification.md".to_string()],
            document_refs: vec![".canon/runs/canon-run-1/verification.md".to_string()],
            readiness,
            missing_sections: Vec::new(),
            headline: "Verification packet ready".to_string(),
            reason_code: Some("packet_ready".to_string()),
            authority_governance: None,
            adaptive_governance: None,
            semantic_descriptor: None,
        }
    }

    fn required_canon_policy() -> StageGovernancePolicy {
        StageGovernancePolicy {
            flow_name: "bug-fix".to_string(),
            stage_id: "investigate".to_string(),
            enabled: true,
            required: true,
            autopilot: false,
            require_adaptive_companion: false,
            runtime: Some(GovernanceRuntimeKind::Canon),
            canon_mode: None,
            reasoning_profile: None,
            system_context: None,
            risk: Some("medium".to_string()),
            zone: Some("core".to_string()),
            owner: Some("team-boundline".to_string()),
        }
    }

    #[test]
    fn governance_input_documents_include_project_memory_provenance_without_authored_brief() {
        let memory = CompactedCanonMemory {
            headline: "Canon project memory available".to_string(),
            credibility: MemoryCredibilityState::Credible,
            stage_key: None,
            run_ref: Some("run-123".to_string()),
            packet_ref: None,
            reason_code: None,
            artifact_refs: vec![
                "docs/project/architecture-map.md".to_string(),
                "docs/evidence/architecture/run-123".to_string(),
                ".canon/runs/canon-run-1/verification.md".to_string(),
            ],
            mode_summary: None,
            possible_actions: Vec::new(),
            recommended_next_action: None,
            evidence_summary: Some(CanonEvidenceInspectSummary {
                execution_posture: None,
                carried_forward_items: Vec::new(),
                artifact_provenance_links: vec![
                    "docs/project/architecture-map.md".to_string(),
                    "docs/evidence/architecture/run-123".to_string(),
                    ".canon/runs/canon-run-1/verification.md".to_string(),
                ],
                closure_status: None,
                closure_findings: Vec::new(),
            }),
            authority_provenance_lines: Vec::new(),
            adaptive_provenance_lines: Vec::new(),
            semantic_provenance_lines: Vec::new(),
        };

        let documents = governance_input_documents(&json!({}), Some(&memory));

        assert_eq!(
            documents,
            vec![
                GovernanceInputDocument {
                    kind: "project-memory-surface".to_string(),
                    path: "docs/project/architecture-map.md".to_string(),
                },
                GovernanceInputDocument {
                    kind: "project-memory-evidence".to_string(),
                    path: "docs/evidence/architecture/run-123".to_string(),
                },
            ]
        );
    }

    #[test]
    fn compacted_canon_memory_from_response_surfaces_approval_guidance() {
        let response = GovernanceRuntimeResponse {
            status: GovernanceLifecycleState::AwaitingApproval,
            approval_state: ApprovalState::Requested,
            run_ref: Some("canon-run-2".to_string()),
            packet: Some(packet(PacketReadiness::Pending)),
            reason_code: Some("approval_requested".to_string()),
            message: "Canon is waiting for approval".to_string(),
        };

        let memory = compacted_canon_memory_from_response(
            "change:verify",
            GovernanceRuntimeKind::Canon,
            &response,
        )
        .expect("Canon responses should compact into memory");

        assert_eq!(memory.credibility, MemoryCredibilityState::Credible);
        assert_eq!(
            memory.recommended_next_action,
            Some(CanonRecommendedActionSummary {
                action: "approve".to_string(),
                rationale: "Canon is waiting for approval".to_string(),
                target: Some("canon-run-2".to_string()),
            })
        );
        assert_eq!(memory.possible_actions[0].action, "approve");
    }

    #[test]
    fn compacted_canon_memory_from_response_projects_canon_contract_provenance_lines() {
        let mut governed_packet = packet(PacketReadiness::Reusable);
        governed_packet.authority_governance = Some(CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Yellow,
            change_class: CanonChangeClass::SystemicImpact,
            intended_persona: CanonIntendedPersona::SystemArchitect,
            approval_state: ApprovalState::Granted,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: Some("verification.md".to_string()),
            artifact_order: vec!["verification.md".to_string()],
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        });
        governed_packet.adaptive_governance = Some(CanonAdaptiveGovernanceV1Envelope {
            contract_line: ADAPTIVE_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            governance_state: CanonAdaptiveGovernanceState::Rule,
            rollout_profile: CanonAdaptiveRolloutProfile::Governed,
            state_rationale: Some("approval required".to_string()),
            profile_rationale: Some("yellow zone packet".to_string()),
        });
        governed_packet.semantic_descriptor = Some(CanonSemanticArtifactDescriptorV1Envelope {
            semantic_contract_line: SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE.to_string(),
            semantic_eligibility: CanonSemanticEligibilityState::Eligible,
            semantic_provenance_boundary: Some(CanonSemanticProvenanceBoundary::Surface),
            semantic_provenance_ref: Some(".canon/runs/canon-run-1/verification.md".to_string()),
            semantic_labels: vec!["verification".to_string()],
            semantic_exclusion_reason: None,
        });
        let response = response(
            GovernanceLifecycleState::GovernedReady,
            "Canon packet ready",
            Some(governed_packet),
        );

        let memory = compacted_canon_memory_from_response(
            "change:verify",
            GovernanceRuntimeKind::Canon,
            &response,
        )
        .expect("Canon responses should compact into memory");

        assert!(
            memory
                .authority_provenance_lines
                .iter()
                .any(|line| { line == "authority_contract_line: authority-governance-v1" })
        );
        assert!(
            memory
                .adaptive_provenance_lines
                .iter()
                .any(|line| line == "adaptive_contract_line: adaptive-governance-v1")
        );
        assert!(
            memory
                .semantic_provenance_lines
                .iter()
                .any(|line| line == "semantic_contract_line: v1")
        );
        assert!(
            memory
                .provenance_lines()
                .iter()
                .any(|line| line == "authority_control_class: council_review")
        );
        assert!(
            memory
                .provenance_lines()
                .iter()
                .any(|line| line == "adaptive_rollout_profile: governed")
        );
        assert!(
            memory
                .provenance_lines()
                .iter()
                .any(|line| line
                    == "semantic_provenance_ref: .canon/runs/canon-run-1/verification.md")
        );
    }

    #[test]
    fn fail_closed_required_authority_response_blocks_missing_metadata() {
        let policy = required_canon_policy();
        let response = response(
            GovernanceLifecycleState::GovernedReady,
            "Canon packet ready",
            Some(packet(PacketReadiness::Reusable)),
        );

        let result = fail_closed_required_authority_response(
            "bug-fix:investigate",
            &policy,
            GovernanceRuntimeKind::Canon,
            &response,
        )
        .expect("missing authority metadata should fail closed");

        assert_eq!(result.status, GovernanceLifecycleState::Blocked);
        assert_eq!(result.reason_code.as_deref(), Some(AUTHORITY_CONTRACT_UNAVAILABLE_REASON_CODE));
        assert!(result.message.contains("missing"));
    }

    #[test]
    fn fail_closed_required_authority_response_blocks_unsupported_contract_line() {
        let policy = required_canon_policy();
        let mut governed_packet = packet(PacketReadiness::Reusable);
        governed_packet.authority_governance = Some(CanonAuthorityGovernanceV1Envelope {
            contract_line: format!("{}-next", AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE),
            authority_zone: CanonAuthorityZone::Yellow,
            change_class: CanonChangeClass::SystemicImpact,
            intended_persona: CanonIntendedPersona::SystemArchitect,
            approval_state: ApprovalState::NotNeeded,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        });
        let response = response(
            GovernanceLifecycleState::GovernedReady,
            "Canon packet ready",
            Some(governed_packet),
        );

        let blocked = fail_closed_required_authority_response(
            "bug-fix:investigate",
            &policy,
            GovernanceRuntimeKind::Canon,
            &response,
        )
        .expect("unsupported contracts should fail closed");

        assert_eq!(blocked.status, GovernanceLifecycleState::Blocked);
        assert_eq!(
            blocked.reason_code.as_deref(),
            Some(AUTHORITY_CONTRACT_UNAVAILABLE_REASON_CODE)
        );
        assert!(blocked.message.contains("unsupported"));
    }

    #[test]
    fn fail_closed_required_authority_response_blocks_missing_adaptive_companion_when_required() {
        let mut policy = required_canon_policy();
        policy.require_adaptive_companion = true;
        let mut governed_packet = packet(PacketReadiness::Reusable);
        governed_packet.authority_governance = Some(CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Yellow,
            change_class: CanonChangeClass::SystemicImpact,
            intended_persona: CanonIntendedPersona::SystemArchitect,
            approval_state: ApprovalState::Granted,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        });
        let response = response(
            GovernanceLifecycleState::GovernedReady,
            "Canon packet ready",
            Some(governed_packet),
        );

        let blocked = fail_closed_required_authority_response(
            "bug-fix:investigate",
            &policy,
            GovernanceRuntimeKind::Canon,
            &response,
        )
        .expect("missing adaptive companion should fail closed when required");

        assert_eq!(blocked.status, GovernanceLifecycleState::Blocked);
        assert_eq!(blocked.reason_code.as_deref(), Some(ADAPTIVE_CONTRACT_UNAVAILABLE_REASON_CODE));
        assert!(blocked.message.contains("adaptive-governance-v1"));
    }

    #[test]
    fn compacted_canon_memory_from_response_marks_rejected_packets_as_contradicted() {
        let response = response(
            GovernanceLifecycleState::Failed,
            "Canon rejected the packet",
            Some(packet(PacketReadiness::Rejected)),
        );

        let memory = compacted_canon_memory_from_response(
            "change:verify",
            GovernanceRuntimeKind::Canon,
            &response,
        )
        .expect("Canon responses should compact into memory");

        assert_eq!(
            canon_memory_credibility(response.status, response.packet.as_ref()),
            MemoryCredibilityState::Contradicted
        );
        assert_eq!(memory.credibility, MemoryCredibilityState::Contradicted);
        assert_eq!(memory.recommended_next_action.as_ref().unwrap().action, "replan");
        assert_eq!(
            canon_memory_possible_actions(&response, MemoryCredibilityState::Contradicted)[0]
                .action,
            "replan"
        );
        assert_eq!(
            canon_memory_recommended_action(&response, MemoryCredibilityState::Stale)
                .as_ref()
                .unwrap()
                .action,
            "refresh"
        );
    }

    #[test]
    fn compacted_canon_memory_for_block_is_only_created_for_canon_runtime() {
        assert!(
            compacted_canon_memory_for_block(
                "bug-fix:investigate",
                GovernanceRuntimeKind::Local,
                "local governance stayed optional"
            )
            .is_none()
        );

        let memory = compacted_canon_memory_for_block(
            "bug-fix:investigate",
            GovernanceRuntimeKind::Canon,
            "canon unavailable",
        )
        .expect("Canon block should create compact memory");

        assert_eq!(memory.credibility, MemoryCredibilityState::Insufficient);
        assert_eq!(memory.reason_code.as_deref(), Some("blocked_context"));
        assert_eq!(memory.recommended_next_action.as_ref().unwrap().action, "refresh");
    }

    #[test]
    fn canon_memory_possible_actions_returns_refresh_for_stale_memory() {
        let response = response(
            GovernanceLifecycleState::Blocked,
            "Canon packet needs to be refreshed",
            Some(packet(PacketReadiness::Incomplete)),
        );

        let actions = canon_memory_possible_actions(&response, MemoryCredibilityState::Stale);

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, "refresh");
        assert_eq!(actions[0].target.as_deref(), Some("canon-run-1"));
    }

    #[test]
    fn serialization_helpers_cover_null_and_error_paths() {
        struct FailingValue;

        impl Serialize for FailingValue {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("forced failure"))
            }
        }

        assert_eq!(optional_serialized_value::<f64>("none", None).unwrap(), Value::Null);

        let error = serialize_to_value("bad-number", &FailingValue).unwrap_err();
        assert!(matches!(
            error,
            super::GovernanceStatePatchError::Serialization { ref key, .. }
                if key == "bad-number"
        ));
    }

    #[test]
    fn governance_state_selection_error_wraps_task_context_error() {
        let error = GovernanceStateSelectionError::from_task_context(
            TaskContextError::StateDeserializationFailed {
                key: "latest_governance_stage".to_string(),
                message: "broken".to_string(),
            },
        );

        assert!(
            matches!(error, GovernanceStateSelectionError::TaskContext(message) if message.contains("latest_governance_stage"))
        );
    }

    #[test]
    fn mode_selection_gate_explicit_mode_overrides_preference() {
        use super::{ModeSelectionOutcome, evaluate_mode_selection_gate};
        use crate::domain::governance::{CanonMode, CanonModeSelectionPreference};

        let outcome = evaluate_mode_selection_gate(
            CanonModeSelectionPreference::Manual,
            Some(CanonMode::Requirements),
        );
        assert_eq!(outcome, ModeSelectionOutcome::ExplicitMode(CanonMode::Requirements));
    }

    #[test]
    fn mode_selection_gate_manual_without_mode_returns_pending() {
        use super::{ModeSelectionOutcome, evaluate_mode_selection_gate};
        use crate::domain::governance::CanonModeSelectionPreference;

        let outcome = evaluate_mode_selection_gate(CanonModeSelectionPreference::Manual, None);
        assert!(
            matches!(outcome, ModeSelectionOutcome::PendingSelection { message } if message.contains("--mode"))
        );
    }

    #[test]
    fn mode_selection_gate_auto_confirm_returns_inferred() {
        use super::{ModeSelectionOutcome, evaluate_mode_selection_gate};
        use crate::domain::governance::CanonModeSelectionPreference;

        let outcome = evaluate_mode_selection_gate(CanonModeSelectionPreference::AutoConfirm, None);
        assert!(matches!(outcome, ModeSelectionOutcome::InferredMode { .. }));
    }

    #[test]
    fn mode_selection_gate_auto_returns_auto_selected() {
        use super::{ModeSelectionOutcome, evaluate_mode_selection_gate};
        use crate::domain::governance::CanonModeSelectionPreference;

        let outcome = evaluate_mode_selection_gate(CanonModeSelectionPreference::Auto, None);
        assert!(matches!(outcome, ModeSelectionOutcome::AutoSelectedMode(_)));
    }

    #[test]
    fn clarification_and_refresh_helpers_cover_pending_incomplete_and_blocked_paths() {
        let pending_selection = clarification_prompt_from_response(&GovernanceRuntimeResponse {
            status: GovernanceLifecycleState::PendingSelection,
            approval_state: ApprovalState::NotNeeded,
            run_ref: Some("canon-run-3".to_string()),
            packet: None,
            reason_code: Some("mode_selection_required".to_string()),
            message: "select a Canon mode".to_string(),
        });
        assert_eq!(
            pending_selection.as_deref(),
            Some("Canon requires mode selection: select a Canon mode"),
        );

        let incomplete = clarification_prompt_from_response(&GovernanceRuntimeResponse {
            status: GovernanceLifecycleState::Incomplete,
            approval_state: ApprovalState::NotNeeded,
            run_ref: Some("canon-run-4".to_string()),
            packet: Some(GovernedStagePacket {
                missing_sections: vec!["risk".to_string(), "evidence".to_string()],
                ..packet(PacketReadiness::Incomplete)
            }),
            reason_code: Some("incomplete_packet".to_string()),
            message: "complete the missing sections".to_string(),
        });
        assert_eq!(
            incomplete.as_deref(),
            Some(
                "Canon document is incomplete. Missing sections: risk, evidence. complete the missing sections",
            ),
        );

        let blocked = clarification_prompt_from_response(&GovernanceRuntimeResponse {
            status: GovernanceLifecycleState::Blocked,
            approval_state: ApprovalState::NotNeeded,
            run_ref: Some("canon-run-5".to_string()),
            packet: Some(GovernedStagePacket {
                missing_sections: vec!["approval".to_string()],
                ..packet(PacketReadiness::Incomplete)
            }),
            reason_code: Some("blocked_packet".to_string()),
            message: "approval data is still missing".to_string(),
        });
        assert_eq!(
            blocked.as_deref(),
            Some(
                "Canon document is incomplete. Missing sections: approval. approval data is still missing",
            ),
        );

        assert!(
            clarification_prompt_from_response(&response(
                GovernanceLifecycleState::GovernedReady,
                "ready",
                Some(packet(PacketReadiness::Reusable)),
            ))
            .is_none()
        );

        let mut session = ActiveSessionRecord {
            session_id: "session-governance-refresh".to_string(),
            workspace_ref: "/tmp/governance-refresh".to_string(),
            goal: Some("refresh governance".to_string()),
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
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            delight_feedback: None,
            latest_voting: None,
        };
        set_lifecycle_awaiting_approval(
            &mut session,
            &response(
                GovernanceLifecycleState::AwaitingApproval,
                "waiting for approval",
                Some(packet(PacketReadiness::Pending)),
            ),
        );
        assert!(!lifecycle_requires_refresh(&session));

        session.governance_lifecycle = Some(GovernedSessionLifecycle {
            governance_runtime: GovernanceRuntimeKind::Canon,
            explicit_opt_out: false,
            mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
            selected_mode: Some(crate::domain::governance::CanonMode::Verification),
            selected_mode_sequence: vec![crate::domain::governance::CanonMode::Verification],
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: Vec::new(),
            accumulated_context: Vec::new(),
            terminal_reason: None,
            planning_input_fingerprint: None,
        });
        set_lifecycle_awaiting_approval(
            &mut session,
            &response(
                GovernanceLifecycleState::AwaitingApproval,
                "still waiting",
                Some(packet(PacketReadiness::Pending)),
            ),
        );
        assert!(lifecycle_requires_refresh(&session));
    }

    #[test]
    fn packet_reuse_and_autopilot_helpers_cover_remaining_branches() -> Result<(), String> {
        let mut context = TaskContext::new(
            "session-governance",
            "/tmp/governance",
            RunLimits::default(),
            Map::new(),
        );
        let stage_result = context.set_latest_governance_stage(&GovernedStageRecord {
            stage_key: "bug-fix:investigate".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::GovernedReady,
            required: false,
            autopilot_enabled: true,
            approval_state: ApprovalState::Granted,
            canon_run_ref: Some("canon-run-6".to_string()),
            governance_attempt_id: "attempt-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: Some(".canon/runs/canon-run-6".to_string()),
            decision_ref: None,
            stage_council: None,
            blocked_reason: None,
        });
        stage_result.map_err(|error| format!("unexpected stage persistence error: {error:?}"))?;
        let packet_result =
            context.set_latest_governance_packet(&packet(PacketReadiness::Reusable));
        packet_result.map_err(|error| format!("unexpected packet persistence error: {error:?}"))?;

        let same_stage_packets = bounded_reused_packets(
            &context,
            &FlowStepMetadata {
                flow_name: "bug-fix".to_string(),
                stage_id: "investigate".to_string(),
                stage_index: 0,
                total_stages: 3,
            },
        );
        let same_stage_packets = same_stage_packets
            .map_err(|error| format!("unexpected packet reuse error: {error:?}"))?;
        assert_eq!(same_stage_packets.len(), 1);
        assert_eq!(same_stage_packets[0].stage_key, "bug-fix:investigate");

        let downstream_packets = bounded_reused_packets(
            &context,
            &FlowStepMetadata {
                flow_name: "bug-fix".to_string(),
                stage_id: "implement".to_string(),
                stage_index: 1,
                total_stages: 3,
            },
        );
        let downstream_packets = downstream_packets
            .map_err(|error| format!("unexpected upstream reuse error: {error:?}"))?;
        assert_eq!(downstream_packets.len(), 1);
        assert_eq!(downstream_packets[0].stage_key, "bug-fix:investigate");

        let approval_policy = StageGovernancePolicy {
            flow_name: "bug-fix".to_string(),
            stage_id: "verify".to_string(),
            enabled: true,
            required: false,
            autopilot: true,
            require_adaptive_companion: false,
            runtime: Some(GovernanceRuntimeKind::Canon),
            canon_mode: None,
            reasoning_profile: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
        };
        let approval_decision = build_autopilot_decision(
            "attempt-approval",
            &approval_policy,
            GovernanceRuntimeKind::Canon,
            &FlowStepMetadata {
                flow_name: "bug-fix".to_string(),
                stage_id: "verify".to_string(),
                stage_index: 2,
                total_stages: 3,
            },
            &GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            Some(GovernanceLifecycleState::AwaitingApproval),
            Some(ApprovalState::Requested),
            Some(PacketReadiness::Pending),
        );
        let approval_decision =
            approval_decision.ok_or_else(|| "approval decision should exist".to_string())?;
        assert_eq!(approval_decision.selected_action, Some(AutopilotAction::AwaitApproval));

        let narrowed_decision = build_autopilot_decision(
            "attempt-narrowed",
            &StageGovernancePolicy {
                canon_mode: Some(crate::domain::governance::CanonMode::Verification),
                ..approval_policy.clone()
            },
            GovernanceRuntimeKind::Canon,
            &FlowStepMetadata {
                flow_name: "bug-fix".to_string(),
                stage_id: "verify".to_string(),
                stage_index: 2,
                total_stages: 3,
            },
            &GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string(), "tests/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            Some(GovernanceLifecycleState::Blocked),
            Some(ApprovalState::Granted),
            Some(PacketReadiness::Rejected),
        );
        let narrowed_decision =
            narrowed_decision.ok_or_else(|| "narrowed decision should exist".to_string())?;
        assert_eq!(
            narrowed_decision.selected_action,
            Some(AutopilotAction::RetryStageWithNarrowedContext),
        );

        let implement_decision = build_autopilot_decision(
            "attempt-escalate-verify",
            &StageGovernancePolicy {
                stage_id: "implement".to_string(),
                runtime: Some(GovernanceRuntimeKind::Local),
                canon_mode: None,
                ..approval_policy.clone()
            },
            GovernanceRuntimeKind::Local,
            &FlowStepMetadata {
                flow_name: "bug-fix".to_string(),
                stage_id: "implement".to_string(),
                stage_index: 1,
                total_stages: 3,
            },
            &GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            Some(GovernanceLifecycleState::GovernedReady),
            Some(ApprovalState::Granted),
            Some(PacketReadiness::Reusable),
        );
        let implement_decision =
            implement_decision.ok_or_else(|| "implement decision should exist".to_string())?;
        assert_eq!(implement_decision.selected_action, Some(AutopilotAction::EscalateVerification),);

        let blocked_decision = build_autopilot_decision(
            "attempt-blocked",
            &StageGovernancePolicy {
                required: true,
                canon_mode: Some(crate::domain::governance::CanonMode::PrReview),
                ..approval_policy
            },
            GovernanceRuntimeKind::Canon,
            &FlowStepMetadata {
                flow_name: "bug-fix".to_string(),
                stage_id: "verify".to_string(),
                stage_index: 2,
                total_stages: 3,
            },
            &GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            Some(GovernanceLifecycleState::GovernedReady),
            Some(ApprovalState::Granted),
            Some(PacketReadiness::Reusable),
        );
        let blocked_decision =
            blocked_decision.ok_or_else(|| "blocked decision should exist".to_string())?;
        assert!(blocked_decision.selected_action.is_none());
        assert!(blocked_decision.blocked_reason.is_some());
        assert!(blocked_decision.candidate_actions.contains(&AutopilotAction::BlockStage));

        Ok(())
    }
}
