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
use crate::domain::configuration::PersistedCapabilityProviderConfiguration;
use crate::domain::execution::StageRoutingDecisionRecord;
use crate::domain::framework_adapter::{
    AdapterConfigCompletenessState, FrameworkAdapterStageOutcomeDetails,
    StoredAdapterConfigValueState,
};
use crate::domain::session::{
    CapabilityProviderExecutionRecord, FrameworkAdapterStageFailureDetails,
};
use crate::domain::trace::{CapabilityProviderTraceRecord, HookEventDispatchRecord};
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
const PROVIDER_STATUS_UNCONFIGURED: &str = "unconfigured";
const PROVIDER_STATUS_ACTIVE: &str = "active";
const PROVIDER_STATUS_BLOCKED: &str = "blocked";
const PROVIDER_STATUS_INACTIVE: &str = "inactive";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapabilityProviderOutputProjection {
    pub status: String,
    pub provider_id: Option<String>,
    pub activation_state: Option<String>,
    pub capability_ids: Option<String>,
    pub setup_requirements: Option<String>,
    pub summary: Option<String>,
}

impl CapabilityProviderOutputProjection {
    fn status_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("capability_provider_status: {}", self.status)];
        if let Some(provider_id) = &self.provider_id {
            lines.push(format!("capability_provider_id: {provider_id}"));
        }
        if let Some(activation_state) = &self.activation_state {
            lines.push(format!("capability_provider_activation_state: {activation_state}"));
        }
        if let Some(capability_ids) = &self.capability_ids {
            lines.push(format!("capability_provider_capability_ids: {capability_ids}"));
        }
        if let Some(setup_requirements) = &self.setup_requirements {
            lines.push(format!("capability_provider_setup_requirements: {setup_requirements}"));
        }
        if let Some(summary) = &self.summary {
            lines.push(format!("capability_provider_summary: {summary}"));
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

pub(crate) fn capability_provider_status_lines(
    workspace_ref: &str,
    execution: Option<&CapabilityProviderExecutionRecord>,
) -> Vec<String> {
    if let Some(execution) = execution {
        return capability_provider_execution_lines(Some(execution));
    }
    capability_provider_output_projection(workspace_ref).status_lines()
}

pub(crate) fn capability_provider_execution_lines(
    execution: Option<&CapabilityProviderExecutionRecord>,
) -> Vec<String> {
    let Some(execution) = execution else {
        return Vec::new();
    };
    let projection = &execution.projection;
    let mut lines = vec![
        format!("capability_provider_id: {}", projection.provider_id),
        format!("capability_provider_activation_state: {}", projection.activation_state.as_str()),
        format!("capability_provider_summary: {}", projection.summary),
    ];
    if let Some(capability_id) = &projection.capability_id {
        lines.push(format!("capability_provider_capability_id: {capability_id}"));
    }
    if let Some(readiness_state) = projection.readiness_state {
        lines.push(format!(
            "capability_provider_readiness_state: {}",
            match readiness_state {
                crate::domain::capability_provider::ProviderReadinessState::Ready => "ready",
                crate::domain::capability_provider::ProviderReadinessState::Degraded => "degraded",
                crate::domain::capability_provider::ProviderReadinessState::Unavailable => {
                    "unavailable"
                }
            }
        ));
    }
    if let Some(validation_disposition) = projection.validation_disposition {
        lines.push(format!(
            "capability_provider_validation_disposition: {}",
            validation_disposition.as_str()
        ));
    }
    if let Some(failure_class) = projection.failure_class {
        lines.push(format!("capability_provider_failure_class: {}", failure_class.as_str()));
    }
    if !projection.accepted_evidence_refs.is_empty() {
        lines.push(format!(
            "capability_provider_accepted_evidence_refs: {}",
            projection.accepted_evidence_refs.join(", ")
        ));
    }
    if !projection.rejected_evidence_refs.is_empty() {
        lines.push(format!(
            "capability_provider_rejected_evidence_refs: {}",
            projection.rejected_evidence_refs.join(", ")
        ));
    }
    if !projection.limitations.is_empty() {
        lines.push(format!(
            "capability_provider_limitations: {}",
            projection.limitations.join(", ")
        ));
    }
    if !projection.setup_requirements.is_empty() {
        lines.push(format!(
            "capability_provider_setup_requirements: {}",
            render_provider_setup_requirements(&projection.setup_requirements)
        ));
    }
    lines
}

pub(crate) fn capability_provider_trace_lines(
    trace_record: Option<&CapabilityProviderTraceRecord>,
) -> Vec<String> {
    let Some(trace_record) = trace_record else {
        return Vec::new();
    };
    capability_provider_execution_lines(Some(&CapabilityProviderExecutionRecord {
        request_id: trace_record.request_id.clone(),
        projection: trace_record.projection.clone(),
    }))
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
    append_framework_adapter_stage_detail_lines(&mut lines, failure.execution.details.as_ref());

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

    append_framework_adapter_stage_detail_lines(&mut lines, routing.details.as_ref());

    lines
}

fn append_framework_adapter_stage_detail_lines(
    lines: &mut Vec<String>,
    details: Option<&FrameworkAdapterStageOutcomeDetails>,
) {
    let Some(details) = details else {
        return;
    };

    if let Some(workflow_id) = details.workflow_id.as_deref() {
        lines.push(format!("framework_adapter_workflow_id: {workflow_id}"));
    }
    if !details.executed_commands.is_empty() {
        lines.push(format!(
            "framework_adapter_executed_commands: {}",
            details.executed_commands.join(", ")
        ));
    }
    if let Some(readiness_status) = details.final_planning_readiness_status {
        lines.push(format!("framework_adapter_planning_readiness: {}", readiness_status.as_str()));
    }
    if let Some(analyze_pass_count) = details.analyze_pass_count {
        lines.push(format!("framework_adapter_analyze_pass_count: {analyze_pass_count}"));
    }
    if let Some(remediation_cycles_used) = details.remediation_cycles_used {
        lines.push(format!("framework_adapter_remediation_cycles_used: {remediation_cycles_used}"));
    }
    if let Some(implementation_status) = details.implementation_status {
        lines.push(format!(
            "framework_adapter_implementation_status: {}",
            implementation_status.as_str()
        ));
    }
    if !details.planning_findings.is_empty() {
        lines.push(format!(
            "framework_adapter_planning_findings: {}",
            format_planning_findings(&details.planning_findings)
        ));
    }
    if !details.remaining_blocking_findings.is_empty() {
        lines.push(format!(
            "framework_adapter_remaining_blocking_findings: {}",
            format_planning_findings(&details.remaining_blocking_findings)
        ));
    }
    if !details.remediation_tasks_attempted.is_empty() {
        lines.push(format!(
            "framework_adapter_remediation_attempted: {}",
            format_remediation_tasks(&details.remediation_tasks_attempted)
        ));
    }
    if !details.remediation_tasks_completed.is_empty() {
        lines.push(format!(
            "framework_adapter_remediation_completed: {}",
            format_remediation_tasks(&details.remediation_tasks_completed)
        ));
    }
    if !details.remediation_tasks_skipped.is_empty() {
        lines.push(format!(
            "framework_adapter_remediation_skipped: {}",
            format_remediation_tasks(&details.remediation_tasks_skipped)
        ));
    }
    if !details.validation_refs.is_empty() {
        lines.push(format!(
            "framework_adapter_validation_refs: {}",
            details.validation_refs.join(", ")
        ));
    }
}

fn format_planning_findings(
    findings: &[crate::domain::framework_adapter::PlanningFinding],
) -> String {
    findings
        .iter()
        .map(|finding| format!("{}:{}", finding.finding_id, finding.severity.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_remediation_tasks(
    tasks: &[crate::domain::framework_adapter::PlanningRemediationTaskOutcome],
) -> String {
    tasks
        .iter()
        .map(|task| match task.skip_reason {
            Some(skip_reason) => format!("{}:{}", task.task_id, skip_reason.as_str()),
            None => task.task_id.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
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

pub(crate) fn capability_provider_output_projection(
    workspace_ref: &str,
) -> CapabilityProviderOutputProjection {
    let workspace = Path::new(workspace_ref);
    if let Ok(Some(configuration)) =
        FileConfigStore::for_workspace(workspace).local_capability_provider()
    {
        return projection_from_provider_configuration(configuration);
    }
    CapabilityProviderOutputProjection {
        status: PROVIDER_STATUS_UNCONFIGURED.to_string(),
        provider_id: None,
        activation_state: None,
        capability_ids: None,
        setup_requirements: None,
        summary: None,
    }
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

fn projection_from_provider_configuration(
    configuration: PersistedCapabilityProviderConfiguration,
) -> CapabilityProviderOutputProjection {
    let selected = configuration
        .active_provider_id
        .as_deref()
        .and_then(|provider_id| {
            configuration.registrations.iter().find(|item| item.provider_id == provider_id)
        })
        .or_else(|| configuration.registrations.first());
    let Some(selected) = selected else {
        return CapabilityProviderOutputProjection {
            status: PROVIDER_STATUS_UNCONFIGURED.to_string(),
            provider_id: None,
            activation_state: None,
            capability_ids: None,
            setup_requirements: None,
            summary: None,
        };
    };
    let status = match selected.activation_state {
        crate::domain::capability_provider::CapabilityProviderActivationState::Active => {
            PROVIDER_STATUS_ACTIVE
        }
        crate::domain::capability_provider::CapabilityProviderActivationState::Blocked => {
            PROVIDER_STATUS_BLOCKED
        }
        _ => PROVIDER_STATUS_INACTIVE,
    };
    CapabilityProviderOutputProjection {
        status: status.to_string(),
        provider_id: Some(selected.provider_id.clone()),
        activation_state: Some(selected.activation_state.as_str().to_string()),
        capability_ids: Some(join_or_none(&selected.capability_ids)),
        setup_requirements: Some(render_provider_setup_requirements(&selected.setup_requirements)),
        summary: Some(if selected.has_blocking_setup_requirements() {
            "provider activation is blocked by setup requirements".to_string()
        } else {
            "provider registration is available".to_string()
        }),
    }
}

fn render_provider_setup_requirements(
    requirements: &[crate::domain::capability_provider::ProviderSetupRequirement],
) -> String {
    if requirements.is_empty() {
        return "none".to_string();
    }
    requirements
        .iter()
        .map(|item| {
            format!(
                "{}={}",
                item.display_label,
                match item.resolution_state {
                    crate::domain::capability_provider::ProviderSetupResolutionState::Present => {
                        "present"
                    }
                    crate::domain::capability_provider::ProviderSetupResolutionState::Missing => {
                        "missing"
                    }
                    crate::domain::capability_provider::ProviderSetupResolutionState::Invalid => {
                        "invalid"
                    }
                    crate::domain::capability_provider::ProviderSetupResolutionState::Unchecked => {
                        "unchecked"
                    }
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn join_or_none(items: &[String]) -> String {
    if items.is_empty() { "none".to_string() } else { items.join(", ") }
}

#[cfg(test)]
mod tests {
    use super::{
        capability_provider_execution_lines, capability_provider_trace_lines,
        framework_adapter_stage_failure_lines, framework_adapter_stage_routing_lines,
        projection_from_provider_configuration,
    };
    use crate::domain::capability_provider::{
        CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
        CapabilityProviderProjection, CapabilityProviderRegistration,
        CapabilityProviderRegistrationSource, CommandProviderTransport, ProviderFailureClass,
        ProviderReadinessState, ProviderSetupRequiredState, ProviderSetupRequirement,
        ProviderSetupRequirementKind, ProviderSetupResolutionState, ProviderTransportDescriptor,
        ProviderValidationOutcome,
    };
    use crate::domain::configuration::PersistedCapabilityProviderConfiguration;
    use crate::domain::execution::StageRoutingDecisionRecord;
    use crate::domain::framework_adapter::{
        AdapterExecutionSource, AdapterFailureClass, AdapterLifecycleStageKey,
        FrameworkAdapterStageOutcomeDetails, ImplementationStatus, LifecycleStageExecutionStatus,
        PlanningFinding, PlanningFindingSeverity, PlanningReadinessStatus,
        PlanningRemediationSkipReason, PlanningRemediationTaskOutcome, StageClaimState,
        StageRoutingDecisionReason,
    };
    use crate::domain::session::{
        CapabilityProviderExecutionRecord, FrameworkAdapterStageFailureDetails,
        LifecycleStageExecutionRecord,
    };
    use crate::domain::trace::CapabilityProviderTraceRecord;

    #[test]
    fn framework_adapter_stage_routing_lines_surface_planning_details() {
        let lines = framework_adapter_stage_routing_lines(Some(&StageRoutingDecisionRecord {
            run_id: "run-1".to_string(),
            stage_key: AdapterLifecycleStageKey::Plan,
            execution_source: AdapterExecutionSource::Adapter,
            decision_reason: StageRoutingDecisionReason::DeclaredOverride,
            claim_state: StageClaimState::Completed,
            adapter_id: Some("speckit".to_string()),
            stage_status: Some(LifecycleStageExecutionStatus::Blocked),
            produced_artifacts: vec!["specs/066-agentic-framework-integration/plan.md".to_string()],
            details: Some(sample_planning_details()),
            recorded_at: 1,
        }));

        assert!(lines.iter().any(|line| line == "framework_adapter_workflow_id: speckit-planning"));
        assert!(lines.iter().any(|line| line == "framework_adapter_planning_readiness: blocked"));
        assert!(lines.iter().any(|line| line == "framework_adapter_analyze_pass_count: 2"));
        assert!(lines.iter().any(|line| line == "framework_adapter_remediation_cycles_used: 1"));
        assert!(lines.iter().any(|line| {
            line == "framework_adapter_remediation_skipped: R-002:requires_operator_input"
        }));
    }

    #[test]
    fn framework_adapter_stage_failure_lines_surface_validation_refs() {
        let lines =
            framework_adapter_stage_failure_lines(Some(&FrameworkAdapterStageFailureDetails {
                execution: LifecycleStageExecutionRecord {
                    run_id: "run-2".to_string(),
                    stage_key: AdapterLifecycleStageKey::Run,
                    execution_source: AdapterExecutionSource::Adapter,
                    adapter_id: Some("speckit".to_string()),
                    status: LifecycleStageExecutionStatus::Blocked,
                    intervention_required: true,
                    failure_class: Some(AdapterFailureClass::AdapterRuntime),
                    produced_artifacts: vec!["artifacts/run-brief.md".to_string()],
                    details: Some(FrameworkAdapterStageOutcomeDetails {
                        workflow_id: Some("speckit-implementation".to_string()),
                        implementation_status: Some(ImplementationStatus::Blocked),
                        validation_refs: vec!["validation/run.md".to_string()],
                        ..FrameworkAdapterStageOutcomeDetails::default()
                    }),
                    started_at: Some(1),
                    finished_at: Some(2),
                },
                claim_state: StageClaimState::Claimed,
                summary: "run blocked".to_string(),
                detail: Some("resume run".to_string()),
                protocol_error_code: None,
            }));

        assert!(
            lines
                .iter()
                .any(|line| line == "framework_adapter_workflow_id: speckit-implementation")
        );
        assert!(
            lines.iter().any(|line| line == "framework_adapter_implementation_status: blocked")
        );
        assert!(
            lines.iter().any(|line| line == "framework_adapter_validation_refs: validation/run.md")
        );
    }

    #[test]
    fn capability_provider_lines_surface_projection_and_trace_details() {
        let projection = CapabilityProviderProjection {
            provider_id: "demo-provider".to_string(),
            activation_state: CapabilityProviderActivationState::Active,
            capability_id: Some("capability.demo".to_string()),
            readiness_state: Some(ProviderReadinessState::Degraded),
            validation_disposition: Some(ProviderValidationOutcome::Rejected),
            failure_class: Some(ProviderFailureClass::PostExecutionValidation),
            setup_requirements: vec![ProviderSetupRequirement {
                requirement_id: "config-token".to_string(),
                kind: ProviderSetupRequirementKind::ConfigValue,
                required_state: ProviderSetupRequiredState::Required,
                resolution_state: ProviderSetupResolutionState::Present,
                display_label: "token".to_string(),
                source_ref: Some("config/token".to_string()),
            }],
            accepted_evidence_refs: vec!["artifact://accepted".to_string()],
            rejected_evidence_refs: vec!["artifact://rejected".to_string()],
            limitations: vec!["bounded".to_string()],
            summary: "provider summary".to_string(),
        };
        let execution_lines =
            capability_provider_execution_lines(Some(&CapabilityProviderExecutionRecord {
                request_id: "req-1".to_string(),
                projection: projection.clone(),
            }));
        assert!(
            execution_lines
                .iter()
                .any(|line| line == "capability_provider_capability_id: capability.demo")
        );
        assert!(
            execution_lines
                .iter()
                .any(|line| line == "capability_provider_readiness_state: degraded")
        );
        assert!(
            execution_lines
                .iter()
                .any(|line| { line == "capability_provider_validation_disposition: rejected" })
        );
        assert!(execution_lines.iter().any(|line| {
            line == "capability_provider_failure_class: post_execution_validation"
        }));
        assert!(execution_lines.iter().any(|line| {
            line == "capability_provider_accepted_evidence_refs: artifact://accepted"
        }));
        assert!(execution_lines.iter().any(|line| {
            line == "capability_provider_rejected_evidence_refs: artifact://rejected"
        }));
        assert!(
            execution_lines.iter().any(|line| line == "capability_provider_limitations: bounded")
        );
        assert!(
            execution_lines
                .iter()
                .any(|line| { line == "capability_provider_setup_requirements: token=present" })
        );

        let trace_lines = capability_provider_trace_lines(Some(&CapabilityProviderTraceRecord {
            request_id: "req-1".to_string(),
            projection,
            validation: crate::domain::capability_provider::ProviderValidationDisposition {
                request_id: "req-1".to_string(),
                disposition: ProviderValidationOutcome::Rejected,
                failure_class: Some(ProviderFailureClass::PostExecutionValidation),
                accepted_evidence_refs: vec!["artifact://accepted".to_string()],
                rejected_evidence_refs: vec!["artifact://rejected".to_string()],
                reason: "provider summary".to_string(),
            },
        }));
        assert!(
            trace_lines.iter().any(|line| line == "capability_provider_summary: provider summary")
        );
    }

    #[test]
    fn capability_provider_projection_prefers_active_registration_and_surfaces_setup_summary() {
        let projection =
            projection_from_provider_configuration(PersistedCapabilityProviderConfiguration {
                registrations: vec![CapabilityProviderRegistration {
                    provider_id: "demo-provider".to_string(),
                    display_name: "Demo Provider".to_string(),
                    transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
                        command_ref: "/bin/echo".to_string(),
                        args: Vec::new(),
                        working_directory_ref: None,
                        environment_ref_names: Vec::new(),
                    }),
                    registration_source: CapabilityProviderRegistrationSource::OperatorCli,
                    discovery_state: CapabilityProviderDiscoveryState::Explicit,
                    activation_state: CapabilityProviderActivationState::Blocked,
                    config_refs: Vec::new(),
                    secret_handle_refs: Vec::new(),
                    setup_requirements: vec![ProviderSetupRequirement {
                        requirement_id: "secret-demo".to_string(),
                        kind: ProviderSetupRequirementKind::SecretHandle,
                        required_state: ProviderSetupRequiredState::Required,
                        resolution_state: ProviderSetupResolutionState::Missing,
                        display_label: "demo-secret".to_string(),
                        source_ref: None,
                    }],
                    capability_ids: vec!["capability.demo".to_string()],
                    active_profile_id: None,
                }],
                active_provider_id: Some("demo-provider".to_string()),
                last_validated_at: Some(7),
            });

        assert_eq!(projection.status, "blocked");
        assert_eq!(projection.provider_id.as_deref(), Some("demo-provider"));
        assert_eq!(projection.setup_requirements.as_deref(), Some("demo-secret=missing"));
        assert_eq!(
            projection.summary.as_deref(),
            Some("provider activation is blocked by setup requirements")
        );
    }

    #[test]
    fn capability_provider_output_helpers_cover_unconfigured_and_optional_branches() {
        let unconfigured =
            projection_from_provider_configuration(PersistedCapabilityProviderConfiguration {
                registrations: Vec::new(),
                active_provider_id: None,
                last_validated_at: None,
            });
        assert_eq!(unconfigured.status, "unconfigured");
        assert!(unconfigured.provider_id.is_none());

        let inactive_projection =
            projection_from_provider_configuration(PersistedCapabilityProviderConfiguration {
                registrations: vec![CapabilityProviderRegistration {
                    provider_id: "inactive-provider".to_string(),
                    display_name: "Inactive Provider".to_string(),
                    transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
                        command_ref: "/bin/echo".to_string(),
                        args: Vec::new(),
                        working_directory_ref: None,
                        environment_ref_names: Vec::new(),
                    }),
                    registration_source: CapabilityProviderRegistrationSource::OperatorCli,
                    discovery_state: CapabilityProviderDiscoveryState::Explicit,
                    activation_state: CapabilityProviderActivationState::Inactive,
                    config_refs: Vec::new(),
                    secret_handle_refs: Vec::new(),
                    setup_requirements: vec![ProviderSetupRequirement {
                        requirement_id: "config-demo".to_string(),
                        kind: ProviderSetupRequirementKind::ConfigValue,
                        required_state: ProviderSetupRequiredState::Required,
                        resolution_state: ProviderSetupResolutionState::Unchecked,
                        display_label: "demo-config".to_string(),
                        source_ref: None,
                    }],
                    capability_ids: Vec::new(),
                    active_profile_id: None,
                }],
                active_provider_id: Some("inactive-provider".to_string()),
                last_validated_at: Some(9),
            });
        assert_eq!(inactive_projection.status, "inactive");
        assert_eq!(inactive_projection.capability_ids.as_deref(), Some("none"));
        assert_eq!(
            inactive_projection.setup_requirements.as_deref(),
            Some("demo-config=unchecked")
        );
        assert_eq!(
            inactive_projection.summary.as_deref(),
            Some("provider activation is blocked by setup requirements")
        );
        let status_lines = inactive_projection.status_lines();
        assert!(status_lines.iter().any(|line| line == "capability_provider_status: inactive"));
        assert!(status_lines.iter().any(|line| {
            line == "capability_provider_setup_requirements: demo-config=unchecked"
        }));
    }

    #[test]
    fn capability_provider_execution_lines_cover_missing_optional_fields() {
        let lines = capability_provider_execution_lines(Some(&CapabilityProviderExecutionRecord {
            request_id: "req-empty".to_string(),
            projection: CapabilityProviderProjection {
                provider_id: "demo-provider".to_string(),
                activation_state: CapabilityProviderActivationState::Blocked,
                capability_id: None,
                readiness_state: Some(ProviderReadinessState::Unavailable),
                validation_disposition: Some(ProviderValidationOutcome::Blocked),
                failure_class: None,
                setup_requirements: Vec::new(),
                accepted_evidence_refs: Vec::new(),
                rejected_evidence_refs: Vec::new(),
                limitations: Vec::new(),
                summary: "blocked summary".to_string(),
            },
        }));
        assert!(
            lines.iter().any(|line| line == "capability_provider_readiness_state: unavailable")
        );
        assert!(lines.iter().any(|line| line == "capability_provider_summary: blocked summary"));
        assert!(lines.iter().all(|line| !line.starts_with("capability_provider_capability_id:")));
        assert!(lines.iter().all(|line| !line.starts_with("capability_provider_failure_class:")));
    }

    fn sample_planning_details() -> FrameworkAdapterStageOutcomeDetails {
        FrameworkAdapterStageOutcomeDetails {
            workflow_id: Some("speckit-planning".to_string()),
            executed_commands: vec!["speckit.analyze".to_string()],
            planning_findings: vec![PlanningFinding {
                finding_id: "F-001".to_string(),
                summary: "Blocking planning finding".to_string(),
                severity: PlanningFindingSeverity::Blocking,
            }],
            remediation_tasks_attempted: vec![PlanningRemediationTaskOutcome {
                task_id: "R-001".to_string(),
                summary: "Attempt remediation".to_string(),
                finding_ids: vec!["F-001".to_string()],
                skip_reason: None,
            }],
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: vec![PlanningRemediationTaskOutcome {
                task_id: "R-002".to_string(),
                summary: "Needs operator input".to_string(),
                finding_ids: vec!["F-001".to_string()],
                skip_reason: Some(PlanningRemediationSkipReason::RequiresOperatorInput),
            }],
            remaining_blocking_findings: vec![PlanningFinding {
                finding_id: "F-001".to_string(),
                summary: "Blocking planning finding".to_string(),
                severity: PlanningFindingSeverity::Blocking,
            }],
            final_planning_readiness_status: Some(PlanningReadinessStatus::Blocked),
            analyze_pass_count: Some(2),
            remediation_cycles_used: Some(1),
            implementation_status: None,
            validation_refs: Vec::new(),
        }
    }
}
