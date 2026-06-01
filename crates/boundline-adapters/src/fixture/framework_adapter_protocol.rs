//! Golden fixtures for the framework-adapter subprocess protocol.

use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::adapters::{
    FrameworkAdapterConfigFieldDefinition, FrameworkAdapterConfigValue,
    FrameworkAdapterDescribeResponse, FrameworkAdapterExecuteStageRequest,
    FrameworkAdapterExecuteStageResponse, FrameworkAdapterFailureClass,
    FrameworkAdapterFieldValueKind, FrameworkAdapterHookDeliveryStatus,
    FrameworkAdapterPreflightBlockReason, FrameworkAdapterPreflightRequest,
    FrameworkAdapterPreflightResponse, FrameworkAdapterPreflightStatus,
    FrameworkAdapterRequiredFieldPolicy, FrameworkAdapterResponseEnvelope,
    FrameworkAdapterStageExecutionStatus, FrameworkAdapterTransportDescriptor, HookEmissionRequest,
    HookEmissionResponse,
};
use crate::orchestrator::{
    FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, FrameworkHookKey, FrameworkStageKey,
};

const SAMPLE_ADAPTER_ID: &str = "speckit";
const SAMPLE_ADAPTER_VERSION: &str = "0.1.0";
const SAMPLE_BOUNDLINE_RANGE: &str = ">=0.66.0,<0.67.0";
const SAMPLE_BOUNDLINE_VERSION: &str = "0.66.0";
const SAMPLE_TEMPLATE_REPO: &str = "../boundline-framework-template";
const SAMPLE_WORKSPACE_REF: &str = "../tmp/example-workspace";
const SAMPLE_PAYLOAD_REF: &str = ".boundline/traces/example-trace.json";
const SAMPLE_RECOVERY_COMMAND: &str = "boundline adapter add speckit --workspace <workspace>";
const SAMPLE_SUMMARY: &str = "Plan artifacts refreshed through the Speckit profile";
const SAMPLE_FAILURE_SUMMARY: &str = "Speckit could not complete the claimed stage";
const SAMPLE_HOOK_SUMMARY: &str = "Hook processed successfully";
const TEMPLATE_REPO_FIELD_KEY: &str = "template_repo";
const TEMPLATE_REPO_LABEL: &str = "Template repository";
const TEMPLATE_REPO_PROMPT: &str = "Path to the reusable template repo";
const TEMPLATE_REPO_HELP: &str =
    "Point this at ../boundline-framework-template or another checked-out template repo";
const PRODUCED_PLAN_ARTIFACT: &str = "specs/066-agentic-framework-integration/plan.md";
const PRODUCED_TASKS_ARTIFACT: &str = "specs/066-agentic-framework-integration/tasks.md";
const CONTEXT_SPEC_ARTIFACT: &str = "specs/066-agentic-framework-integration/spec.md";
const FAILURE_NEXT_ACTION: &str = "Inspect the adapter log and retry after correction";

/// Builds the canonical required field fixture for the sample Speckit profile.
pub fn sample_framework_adapter_field_definition() -> FrameworkAdapterConfigFieldDefinition {
    FrameworkAdapterConfigFieldDefinition {
        field_key: TEMPLATE_REPO_FIELD_KEY.to_string(),
        display_label: TEMPLATE_REPO_LABEL.to_string(),
        value_kind: FrameworkAdapterFieldValueKind::Path,
        required: true,
        secret: false,
        default_value_text: None,
        prompt_text: TEMPLATE_REPO_PROMPT.to_string(),
        help_text: TEMPLATE_REPO_HELP.to_string(),
        non_interactive_policy: FrameworkAdapterRequiredFieldPolicy::Fail,
    }
}

/// Builds the canonical config-value fixture for the sample Speckit profile.
pub fn sample_framework_adapter_config_value() -> FrameworkAdapterConfigValue {
    FrameworkAdapterConfigValue {
        field_key: TEMPLATE_REPO_FIELD_KEY.to_string(),
        value_kind: FrameworkAdapterFieldValueKind::Path,
        string_value: None,
        path_value: Some(SAMPLE_TEMPLATE_REPO.to_string()),
        bool_value: None,
        int_value: None,
    }
}

/// Builds the canonical `describe` response fixture.
pub fn sample_framework_adapter_describe_response() -> FrameworkAdapterDescribeResponse {
    FrameworkAdapterDescribeResponse {
        protocol_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
        adapter_id: SAMPLE_ADAPTER_ID.to_string(),
        adapter_version: SAMPLE_ADAPTER_VERSION.to_string(),
        supported_boundline_range: SAMPLE_BOUNDLINE_RANGE.to_string(),
        supported_transports: vec![FrameworkAdapterTransportDescriptor::stdio_json()],
        declared_stage_overrides: vec![FrameworkStageKey::Plan, FrameworkStageKey::Run],
        declared_hook_subscriptions: vec![
            FrameworkHookKey::StageCompleted,
            FrameworkHookKey::StageFailed,
        ],
        required_config_fields: vec![sample_framework_adapter_field_definition()],
    }
}

/// Builds the canonical `preflight` request fixture.
pub fn sample_framework_adapter_preflight_request() -> FrameworkAdapterPreflightRequest {
    FrameworkAdapterPreflightRequest {
        boundline_version: SAMPLE_BOUNDLINE_VERSION.to_string(),
        workspace_ref: SAMPLE_WORKSPACE_REF.to_string(),
        non_interactive: false,
        config_values: vec![sample_framework_adapter_config_value()],
    }
}

/// Builds the canonical ready `preflight` response fixture.
pub fn sample_framework_adapter_preflight_ready_response() -> FrameworkAdapterPreflightResponse {
    FrameworkAdapterPreflightResponse {
        status: FrameworkAdapterPreflightStatus::Ready,
        normalized_config_values: vec![sample_framework_adapter_config_value()],
        warnings: Vec::new(),
        reason: None,
        missing_fields: Vec::new(),
        recovery: None,
    }
}

/// Builds the canonical blocked `preflight` response fixture.
pub fn sample_framework_adapter_preflight_blocked_response() -> FrameworkAdapterPreflightResponse {
    FrameworkAdapterPreflightResponse {
        status: FrameworkAdapterPreflightStatus::Blocked,
        normalized_config_values: Vec::new(),
        warnings: Vec::new(),
        reason: Some(FrameworkAdapterPreflightBlockReason::MissingRequiredConfig),
        missing_fields: vec![TEMPLATE_REPO_FIELD_KEY.to_string()],
        recovery: Some(SAMPLE_RECOVERY_COMMAND.to_string()),
    }
}

/// Builds the canonical `execute-stage` request fixture.
pub fn sample_framework_adapter_execute_stage_request() -> FrameworkAdapterExecuteStageRequest {
    FrameworkAdapterExecuteStageRequest {
        run_id: Uuid::from_u128(0xb1d1d3c27f6d4d8c9f576e57fd2d1d02),
        stage_key: FrameworkStageKey::Plan,
        stage_attempt: 1,
        workspace_ref: SAMPLE_WORKSPACE_REF.to_string(),
        adapter_id: SAMPLE_ADAPTER_ID.to_string(),
        config_values: vec![sample_framework_adapter_config_value()],
        context_artifacts: vec![CONTEXT_SPEC_ARTIFACT.to_string()],
    }
}

/// Builds the canonical successful `execute-stage` response fixture.
pub fn sample_framework_adapter_execute_stage_success_response()
-> FrameworkAdapterExecuteStageResponse {
    FrameworkAdapterExecuteStageResponse {
        status: FrameworkAdapterStageExecutionStatus::Succeeded,
        summary: SAMPLE_SUMMARY.to_string(),
        produced_artifacts: vec![
            PRODUCED_PLAN_ARTIFACT.to_string(),
            PRODUCED_TASKS_ARTIFACT.to_string(),
        ],
        next_action: None,
        failure_class: None,
    }
}

/// Builds the canonical failed `execute-stage` response fixture.
pub fn sample_framework_adapter_execute_stage_failed_response()
-> FrameworkAdapterExecuteStageResponse {
    FrameworkAdapterExecuteStageResponse {
        status: FrameworkAdapterStageExecutionStatus::Failed,
        summary: SAMPLE_FAILURE_SUMMARY.to_string(),
        produced_artifacts: Vec::new(),
        next_action: Some(FAILURE_NEXT_ACTION.to_string()),
        failure_class: Some(FrameworkAdapterFailureClass::AdapterRuntime),
    }
}

/// Builds the canonical `emit-hook` request fixture.
pub fn sample_framework_adapter_hook_emission_request() -> HookEmissionRequest {
    HookEmissionRequest {
        run_id: Uuid::from_u128(0xb1d1d3c27f6d4d8c9f576e57fd2d1d02),
        hook_key: FrameworkHookKey::StageCompleted,
        stage_key: FrameworkStageKey::Plan,
        stage_claimed: true,
        workspace_ref: SAMPLE_WORKSPACE_REF.to_string(),
        payload_ref: SAMPLE_PAYLOAD_REF.to_string(),
    }
}

/// Builds the canonical successful `emit-hook` response fixture.
pub fn sample_framework_adapter_hook_emission_response() -> HookEmissionResponse {
    HookEmissionResponse {
        status: FrameworkAdapterHookDeliveryStatus::Delivered,
        summary: SAMPLE_HOOK_SUMMARY.to_string(),
    }
}

/// Wraps a command payload in the standard success envelope used on stdout.
pub fn sample_framework_adapter_success_envelope<T>(
    data: T,
) -> FrameworkAdapterResponseEnvelope<T> {
    FrameworkAdapterResponseEnvelope::success(data)
}

/// Serializes a typed fixture into stable pretty JSON for golden comparisons.
pub fn pretty_fixture_json<T>(value: &T) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value)
}

/// Round-trips a typed fixture through JSON and returns the decoded value.
pub fn round_trip_fixture<T>(value: &T) -> Result<T, serde_json::Error>
where
    T: Serialize + DeserializeOwned,
{
    serde_json::from_str(&pretty_fixture_json(value)?)
}
