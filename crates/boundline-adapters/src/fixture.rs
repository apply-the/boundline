#[path = "../../../src/fixture.rs"]
mod fixture_impl;

pub mod framework_adapter_protocol;

pub use fixture_impl::{
    FilePatch, FixtureCommand, FixtureRuntime, FixtureRuntimeError, FixtureValidationError,
    ReasoningProfileFixtureScenario, WorkspaceFixture, build_fixture_plan,
    build_fixture_plan_for_flow, build_fixture_plan_for_goal, build_fixture_runtime,
    build_fixture_runtime_for_flow, build_fixture_runtime_for_goal_plan, build_task_request,
    execution_manifest_path, load_workspace_execution_profile, local_reasoning_posture_fixture,
    local_reasoning_posture_fixture_for_profile, reasoning_profile_fixture,
};
pub use framework_adapter_protocol::{
    pretty_fixture_json, round_trip_fixture, sample_framework_adapter_config_value,
    sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_failed_response,
    sample_framework_adapter_execute_stage_request,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_field_definition, sample_framework_adapter_hook_emission_request,
    sample_framework_adapter_hook_emission_response,
    sample_framework_adapter_preflight_blocked_response,
    sample_framework_adapter_preflight_ready_response, sample_framework_adapter_preflight_request,
    sample_framework_adapter_success_envelope,
};
