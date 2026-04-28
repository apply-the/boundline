use crate::adapters::governance_runtime::{
    GovernanceBoundedContext, GovernanceInputDocument, ReusedPacketInput,
};
use crate::domain::brief::{AuthoredBriefBundle, GovernanceIntent};
use crate::domain::flow::{FlowStepMetadata, built_in_flow};
use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::domain::governance::{
    ApprovalState, AutopilotAction, AutopilotDecisionRecord, CanonMode, GovernanceLifecycleState,
    GovernanceProfile, GovernanceRuntimeKind, GovernedStagePacket, GovernedStageRecord,
    PacketReadiness, PacketReuseBinding, StageGovernancePolicy, candidate_canon_modes,
    resolved_canon_mode, supported_canon_modes_for_stage,
};
use crate::domain::task_context::{
    LATEST_GOVERNANCE_DECISION_KEY, LATEST_GOVERNANCE_PACKET_KEY,
    LATEST_GOVERNANCE_PACKET_REUSE_KEY, LATEST_GOVERNANCE_STAGE_KEY, TaskContext, TaskContextError,
};

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

pub fn governance_input_documents(task_input: &Value) -> Vec<GovernanceInputDocument> {
    let Some(bundle) = task_input
        .get("authored_brief")
        .cloned()
        .and_then(|value| serde_json::from_value::<AuthoredBriefBundle>(value).ok())
    else {
        return Vec::new();
    };

    let mut documents = Vec::new();
    let mut stage_brief_assigned = false;
    for source in bundle.sources {
        let Some(path) = source.workspace_path else {
            continue;
        };
        let kind = if stage_brief_assigned {
            "authored-brief"
        } else {
            stage_brief_assigned = true;
            "stage-brief"
        };
        documents.push(GovernanceInputDocument { kind: kind.to_string(), path });
    }

    documents
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
            binding_reason: "same_stage_rerun".to_string(),
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
        binding_reason: "upstream_stage_context".to_string(),
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
        if metadata.stage_id == "implement" {
            candidate_actions.push(AutopilotAction::EscalateVerification);
        } else if metadata.stage_id == "verify"
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
        } else if metadata.stage_id == "implement" {
            let target =
                escalation_target_stage_key(metadata, AutopilotAction::EscalateVerification);
            (
                Some(AutopilotAction::EscalateVerification),
                resolved_mode,
                target,
                format!("autopilot escalated {stage_key} toward verification governance"),
                None,
            )
        } else if metadata.stage_id == "verify"
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

pub fn governance_state_patch(
    record: &GovernedStageRecord,
    packet: Option<&GovernedStagePacket>,
    packet_reuse: Option<&PacketReuseBinding>,
    decision: Option<&AutopilotDecisionRecord>,
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

    Ok(patch)
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
