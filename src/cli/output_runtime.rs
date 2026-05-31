use std::path::Path;

use crate::adapters::agent::{FrameworkAdapterHost, SubprocessFrameworkAdapterHost};
use crate::adapters::config_store::FileConfigStore;
use crate::adapters::{
    FrameworkAdapterDescribeResponse, format_framework_adapter_transports,
    framework_adapter_supports_v1_transport,
};
use crate::cli::adapter::{
    command_exists_on_path, config_schema_fingerprint, discovery_state_label,
};
use crate::domain::configuration::PersistedAdapterConfiguration;
use crate::domain::execution::StageRoutingDecisionRecord;
use crate::domain::framework_adapter::{
    AdapterConfigCompletenessState, StoredAdapterConfigValueState,
};
use crate::domain::session::FrameworkAdapterStageFailureDetails;
use crate::domain::trace::HookEventDispatchRecord;
use crate::registry::agent_registry::FrameworkAdapterProfileRegistry;
use serde_json::Value;

use super::{
    EXPLANATION_LABEL_REASONING_CONTRIBUTION, EXPLANATION_LABEL_REASONING_FALLBACK_DISCLOSURE,
    EXPLANATION_LABEL_REASONING_SELECTION_REASON,
};
use crate::domain::reasoning::ProfileActivationRecord;

const STATUS_BUILT_IN_DEFAULT: &str = "built_in_default";
const STATUS_CONFIGURED: &str = "configured";
const STATUS_BLOCKED: &str = "blocked";
const EXECUTION_SOURCE_ADAPTER: &str = "adapter";
const EXECUTION_SOURCE_BUILT_IN: &str = "built_in";
const COMPATIBILITY_GATE_V1_STDIO_JSON: &str = "v1_json_over_stdin_stdout_only";
const BLOCKED_REASON_UNSUPPORTED_TRANSPORT: &str = "unsupported_transport";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrameworkAdapterOutputProjection {
    pub status: String,
    pub execution_source: String,
    pub adapter_id: Option<String>,
    pub config_state: Option<String>,
    pub interactive_resolution: Option<bool>,
    pub value_count: Option<usize>,
    pub discovery_state: Option<String>,
    pub discovery_hint: Option<String>,
    pub activation_required: Option<String>,
    pub supported_transports: Option<String>,
    pub compatibility_gate: Option<String>,
    pub blocked_reason: Option<String>,
}

impl FrameworkAdapterOutputProjection {
    fn status_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("framework_adapter_status: {}", self.status),
            format!("framework_adapter_execution_source: {}", self.execution_source),
        ];

        if let Some(adapter_id) = &self.adapter_id {
            lines.push(format!("framework_adapter_id: {adapter_id}"));
        }
        if let Some(config_state) = &self.config_state {
            lines.push(format!("framework_adapter_config_state: {config_state}"));
        }
        if let Some(interactive_resolution) = self.interactive_resolution {
            lines.push(format!(
                "framework_adapter_interactive_resolution: {interactive_resolution}"
            ));
        }
        if let Some(value_count) = self.value_count {
            lines.push(format!("framework_adapter_value_count: {value_count}"));
        }
        if let Some(discovery_state) = &self.discovery_state {
            lines.push(format!("framework_adapter_discovery_state: {discovery_state}"));
        }
        if let Some(discovery_hint) = &self.discovery_hint {
            lines.push(format!("framework_adapter_discovery_hint: {discovery_hint}"));
        }
        if let Some(activation_required) = &self.activation_required {
            lines.push(format!("framework_adapter_activation_required: {activation_required}"));
        }
        if let Some(supported_transports) = &self.supported_transports {
            lines.push(format!("framework_adapter_supported_transports: {supported_transports}"));
        }
        if let Some(compatibility_gate) = &self.compatibility_gate {
            lines.push(format!("framework_adapter_compatibility_gate: {compatibility_gate}"));
        }
        if let Some(blocked_reason) = &self.blocked_reason {
            lines.push(format!("framework_adapter_blocked_reason: {blocked_reason}"));
        }

        lines
    }
}

pub(crate) fn append_reasoning_profile_lines(
    lines: &mut Vec<String>,
    label_prefix: &str,
    reasoning_profile: &ProfileActivationRecord,
) {
    lines.push(format!("{label_prefix}reasoning_profile_id: {}", reasoning_profile.profile_id));
    lines.push(format!("{label_prefix}reasoning_profile_stage: {}", reasoning_profile.stage_key));
    lines.push(format!(
        "{label_prefix}reasoning_profile_status: {}",
        reasoning_profile.status.as_str()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_profile_trigger: {}",
        reasoning_profile.trigger.as_str()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_profile_reason: {}",
        reasoning_profile.activation_reason
    ));
    lines.push(format!(
        "{label_prefix}{EXPLANATION_LABEL_REASONING_SELECTION_REASON}: {}",
        reasoning_profile.disclosure_selection_reason()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_budget: participants={} branches={} calls={} adjudication_steps={}",
        reasoning_profile.budget.max_participants,
        reasoning_profile.budget.max_branches,
        reasoning_profile.budget.max_calls,
        reasoning_profile.budget.max_adjudication_steps,
    ));
    if !reasoning_profile.participants.is_empty() {
        lines.push(format!(
            "{label_prefix}reasoning_participants: {}",
            reasoning_profile
                .participants
                .iter()
                .map(|participant| format!(
                    "{}={}",
                    participant.role_id, participant.effective_route
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(independence) = &reasoning_profile.independence {
        lines.push(format!(
            "{label_prefix}reasoning_independence_result: {}",
            independence.result.as_str()
        ));
        lines.push(format!("{label_prefix}reasoning_independence_reason: {}", independence.reason));
    }
    if let Some(posture) = &reasoning_profile.posture {
        lines.push(format!("{label_prefix}reasoning_posture_contract: {}", posture.contract_line));
        lines.push(format!(
            "{label_prefix}reasoning_posture_admission_priority: {}",
            posture.admission_priority.as_str()
        ));
        lines.push(format!(
            "{label_prefix}reasoning_posture_confidence_handoff: {}",
            posture.confidence_handoff_required
        ));
        lines.push(format!(
            "{label_prefix}reasoning_posture_provenance_ref: {}",
            posture.provenance_ref
        ));
    }
    if let Some(outcome) = &reasoning_profile.outcome {
        lines.push(format!("{label_prefix}reasoning_outcome: {}", outcome.outcome_kind.as_str()));
        lines.push(format!("{label_prefix}reasoning_outcome_headline: {}", outcome.headline));
        if let Some(disagreement_summary) = &outcome.disagreement_summary {
            lines.push(format!(
                "{label_prefix}reasoning_disagreement_summary: {disagreement_summary}"
            ));
        }
        if let Some(next_action) = &outcome.next_action {
            lines.push(format!("{label_prefix}reasoning_next_action: {next_action}"));
        }
    }
    if let Some(confidence) = &reasoning_profile.confidence {
        lines.push(format!(
            "{label_prefix}reasoning_confidence_level: {}",
            confidence.confidence_level.as_str()
        ));
        lines.push(format!(
            "{label_prefix}reasoning_confidence_effect: {}",
            confidence.admission_effect.as_str()
        ));
        lines.push(format!("{label_prefix}reasoning_confidence_summary: {}", confidence.summary));
    }
    if let Some(contribution_summary) = reasoning_profile.disclosure_contribution_summary() {
        lines.push(format!(
            "{label_prefix}{EXPLANATION_LABEL_REASONING_CONTRIBUTION}: {contribution_summary}"
        ));
    }
    if let Some(fallback_disclosure) = reasoning_profile.disclosure_fallback_disclosure() {
        lines.push(format!(
            "{label_prefix}{EXPLANATION_LABEL_REASONING_FALLBACK_DISCLOSURE}: {fallback_disclosure}"
        ));
    }
}

pub(crate) fn adaptive_workspace_slice_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    let slice = state.get("latest_workspace_slice")?;
    let targets = slice.get("selected_targets")?.as_array()?;
    let targets = targets.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if targets.is_empty() { None } else { Some(targets.join(", ")) }
}

pub(crate) fn adaptive_attempt_lineage_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    let lineage = state.get("latest_attempt_lineage")?;
    let current = lineage.get("current_attempt_id")?.as_str()?;
    let transition = lineage.get("transition_kind")?.as_str()?;
    let previous = lineage.get("previous_attempt_id").and_then(Value::as_str);
    previous.map_or_else(
        || Some(format!("{current} ({transition})")),
        |previous| Some(format!("{current} {transition} {previous}")),
    )
}

pub(crate) fn adaptive_candidate_family_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    state.get("latest_candidate_family")?.as_str().map(str::to_string)
}

pub(crate) fn adaptive_selection_reason_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    state.get("latest_selection_reason")?.as_str().map(str::to_string)
}

pub(crate) fn adaptive_rejected_candidates_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    let rejected = state.get("latest_rejected_candidates")?.as_array()?;
    let rejected = rejected.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if rejected.is_empty() { None } else { Some(rejected.join(" | ")) }
}

pub(crate) fn adaptive_exhaustion_reason_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    state.get("latest_exhaustion_reason")?.as_str().map(str::to_string)
}

pub(crate) fn framework_adapter_output_projection(
    workspace_ref: &str,
) -> FrameworkAdapterOutputProjection {
    let workspace = Path::new(workspace_ref);
    if let Ok(Some(adapter)) = FileConfigStore::for_workspace(workspace).local_adapter() {
        let mut projection = FrameworkAdapterOutputProjection {
            status: STATUS_CONFIGURED.to_string(),
            execution_source: EXECUTION_SOURCE_ADAPTER.to_string(),
            adapter_id: Some(adapter.selection.adapter_id.clone()),
            config_state: Some(adapter_config_state_text(adapter.completeness_state).to_string()),
            interactive_resolution: Some(adapter.interactive_resolution),
            value_count: Some(adapter.value_count),
            discovery_state: Some(
                discovery_state_label(adapter.selection.discovery_state).to_string(),
            ),
            discovery_hint: None,
            activation_required: None,
            supported_transports: None,
            compatibility_gate: Some(COMPATIBILITY_GATE_V1_STDIO_JSON.to_string()),
            blocked_reason: None,
        };

        if let Some(describe) = configured_adapter_describe(workspace, &adapter) {
            projection.config_state = Some(
                adapter_config_state_text(revalidated_adapter_config_state(&adapter, &describe))
                    .to_string(),
            );
            if !describe.supported_transports.is_empty() {
                projection.supported_transports =
                    Some(format_framework_adapter_transports(&describe.supported_transports));
            }
            if !framework_adapter_supports_v1_transport(&describe.supported_transports) {
                projection.status = STATUS_BLOCKED.to_string();
                projection.execution_source = EXECUTION_SOURCE_BUILT_IN.to_string();
                projection.blocked_reason = Some(BLOCKED_REASON_UNSUPPORTED_TRANSPORT.to_string());
            }
        }

        return projection;
    }

    let mut projection = FrameworkAdapterOutputProjection {
        status: STATUS_BUILT_IN_DEFAULT.to_string(),
        execution_source: EXECUTION_SOURCE_BUILT_IN.to_string(),
        adapter_id: None,
        config_state: None,
        interactive_resolution: None,
        value_count: None,
        discovery_state: None,
        discovery_hint: None,
        activation_required: None,
        supported_transports: None,
        compatibility_gate: None,
        blocked_reason: None,
    };

    if let Ok(registry) = FrameworkAdapterProfileRegistry::boundline_known_profiles()
        && let Some(profile) =
            registry.profiles().find(|profile| command_exists_on_path(&profile.default_command))
    {
        projection.discovery_hint = Some(format!("{} available on PATH", profile.adapter_id));
        projection.activation_required =
            Some(format!("boundline adapter add {}", profile.registration_alias));
    }

    projection
}

fn revalidated_adapter_config_state(
    adapter: &PersistedAdapterConfiguration,
    describe: &FrameworkAdapterDescribeResponse,
) -> AdapterConfigCompletenessState {
    if config_schema_fingerprint(describe) != adapter.schema_fingerprint {
        return AdapterConfigCompletenessState::Invalid;
    }

    for field in describe.required_config_fields.iter().filter(|field| field.required) {
        let Some(value) = adapter.values.iter().find(|value| value.field_key == field.field_key)
        else {
            return AdapterConfigCompletenessState::MissingRequired;
        };

        match value.resolution_state {
            StoredAdapterConfigValueState::Present => {}
            StoredAdapterConfigValueState::Missing => {
                return AdapterConfigCompletenessState::MissingRequired;
            }
            StoredAdapterConfigValueState::Invalid => {
                return AdapterConfigCompletenessState::Invalid;
            }
        }
    }

    adapter.completeness_state
}

fn adapter_config_state_text(
    completeness_state: crate::domain::framework_adapter::AdapterConfigCompletenessState,
) -> &'static str {
    match completeness_state {
        crate::domain::framework_adapter::AdapterConfigCompletenessState::Complete => "complete",
        crate::domain::framework_adapter::AdapterConfigCompletenessState::MissingRequired => {
            "missing_required"
        }
        crate::domain::framework_adapter::AdapterConfigCompletenessState::Invalid => "invalid",
    }
}

pub(crate) fn framework_adapter_status_lines(workspace_ref: &str) -> Vec<String> {
    framework_adapter_output_projection(workspace_ref).status_lines()
}

pub(crate) fn framework_adapter_stage_failure_lines(
    failure: Option<&FrameworkAdapterStageFailureDetails>,
) -> Vec<String> {
    let Some(failure) = failure else {
        return Vec::new();
    };

    let mut lines = vec![
        format!(
            "framework_adapter_execution_source: {}",
            failure.execution.execution_source.as_str()
        ),
        format!("framework_adapter_stage: {}", failure.execution.stage_key.as_str()),
        format!("framework_adapter_stage_claim: {}", failure.claim_state.as_str()),
        format!("framework_adapter_stage_status: {}", failure.execution.status.as_str()),
        format!(
            "framework_adapter_intervention_required: {}",
            failure.execution.intervention_required
        ),
    ];

    if let Some(adapter_id) = failure.execution.adapter_id.as_deref() {
        lines.push(format!("framework_adapter_stage_adapter_id: {adapter_id}"));
    }
    if let Some(failure_class) = failure.execution.failure_class {
        lines.push(format!("framework_adapter_failure_class: {}", failure_class.as_str()));
    }
    lines.push(format!("framework_adapter_failure_summary: {}", failure.summary));
    if let Some(detail) = failure.detail.as_deref() {
        lines.push(format!("framework_adapter_failure_detail: {detail}"));
    }
    if let Some(protocol_error_code) = failure.protocol_error_code.as_deref() {
        lines.push(format!("framework_adapter_protocol_error_code: {protocol_error_code}"));
    }

    lines
}

pub(crate) fn framework_adapter_stage_routing_lines(
    routing: Option<&StageRoutingDecisionRecord>,
) -> Vec<String> {
    let Some(routing) = routing else {
        return Vec::new();
    };

    let mut lines = vec![
        format!("framework_adapter_execution_source: {}", routing.execution_source.as_str()),
        format!("framework_adapter_stage: {}", routing.stage_key.as_str()),
        format!("framework_adapter_stage_claim: {}", routing.claim_state.as_str()),
        format!("framework_adapter_routing_reason: {}", routing.decision_reason.as_str()),
    ];

    if let Some(stage_status) = routing.stage_status {
        lines.push(format!("framework_adapter_stage_status: {}", stage_status.as_str()));
    }

    if let Some(adapter_id) = routing.adapter_id.as_deref() {
        lines.push(format!("framework_adapter_stage_adapter_id: {adapter_id}"));
    }

    if !routing.produced_artifacts.is_empty() {
        lines.push(format!(
            "framework_adapter_produced_artifacts: {}",
            routing.produced_artifacts.join(", ")
        ));
    }

    lines
}

pub(crate) fn framework_adapter_hook_dispatch_lines(
    dispatch: Option<&HookEventDispatchRecord>,
) -> Vec<String> {
    let Some(dispatch) = dispatch else {
        return Vec::new();
    };

    vec![
        format!("framework_adapter_hook: {}", dispatch.hook_key.as_str()),
        format!("framework_adapter_hook_stage: {}", dispatch.stage_key.as_str()),
        format!("framework_adapter_hook_delivery_status: {}", dispatch.dispatch_status.as_str()),
        format!("framework_adapter_hook_stage_claimed: {}", dispatch.stage_claimed),
        format!("framework_adapter_hook_adapter_id: {}", dispatch.adapter_id),
        format!("framework_adapter_hook_summary: {}", dispatch.summary),
    ]
}

fn configured_adapter_describe(
    workspace: &Path,
    adapter: &PersistedAdapterConfiguration,
) -> Option<FrameworkAdapterDescribeResponse> {
    let mut host = SubprocessFrameworkAdapterHost::new(adapter.selection.command.clone())
        .ok()?
        .with_args(adapter.selection.args.clone());
    if workspace.is_dir() {
        host = host.with_working_directory(workspace.to_path_buf());
    }
    host.describe().ok()
}
