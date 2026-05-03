#[path = "support/workspace_fixture.rs"]
mod workspace_fixture;

#[path = "support/runtime_refoundation.rs"]
mod runtime_refoundation;

#[path = "integration/sequential_task_run.rs"]
mod sequential_task_run;

#[path = "integration/retry_and_replan.rs"]
mod retry_and_replan;

#[path = "integration/trace_capture.rs"]
mod trace_capture;

#[path = "integration/fixture_vertical_slice.rs"]
mod fixture_vertical_slice;

#[path = "integration/cli_custom_run.rs"]
mod cli_custom_run;

#[path = "integration/cli_adaptive_execution.rs"]
mod cli_adaptive_execution;

#[path = "integration/cli_diagnostics.rs"]
mod cli_diagnostics;

#[path = "integration/distribution_doctor_flow.rs"]
mod distribution_doctor_flow;

#[path = "integration/distribution_doctor_blocked_flow.rs"]
mod distribution_doctor_blocked_flow;

#[path = "integration/release_metadata_flow.rs"]
mod release_metadata_flow;

#[path = "integration/cli_trace_inspection.rs"]
mod cli_trace_inspection;

#[path = "integration/assistant_shell_enabled_flow.rs"]
mod assistant_shell_enabled_flow;

#[path = "integration/assistant_chat_fallback.rs"]
mod assistant_chat_fallback;

#[path = "integration/session_cli_flow.rs"]
mod session_cli_flow;

#[path = "integration/session_adaptive_flow.rs"]
mod session_adaptive_flow;

#[path = "integration/session_compatibility_continuity.rs"]
mod session_compatibility_continuity;

#[path = "integration/flow_cli_run.rs"]
mod flow_cli_run;

#[path = "integration/session_governance_flow.rs"]
mod session_governance_flow;

#[path = "integration/canon_governance_flow.rs"]
mod canon_governance_flow;

#[path = "integration/governance_autopilot_flow.rs"]
mod governance_autopilot_flow;

#[path = "integration/human_input_capture_flow.rs"]
mod human_input_capture_flow;

#[path = "integration/human_input_multi_source_flow.rs"]
mod human_input_multi_source_flow;

#[path = "integration/human_input_governance_flow.rs"]
mod human_input_governance_flow;

#[path = "integration/init_bootstrap_flow.rs"]
mod init_bootstrap_flow;

#[path = "integration/config_workspace_flow.rs"]
mod config_workspace_flow;

#[path = "integration/cluster_bootstrap_flow.rs"]
mod cluster_bootstrap_flow;

#[path = "integration/cluster_status_flow.rs"]
mod cluster_status_flow;

#[path = "integration/cluster_config_flow.rs"]
mod cluster_config_flow;

#[path = "integration/cluster_delivery_flow.rs"]
mod cluster_delivery_flow;

#[path = "integration/cluster_delivery_blocked.rs"]
mod cluster_delivery_blocked;

#[path = "integration/session_native_flow.rs"]
mod session_native_flow;

#[path = "integration/fixture_compat_flow.rs"]
mod fixture_compat_flow;

#[path = "integration/runtime_refoundation_flow.rs"]
mod runtime_refoundation_flow;

#[path = "integration/runtime_refoundation_failure.rs"]
mod runtime_refoundation_failure;

#[path = "integration/runtime_refoundation_compat.rs"]
mod runtime_refoundation_compat;

#[path = "integration/runtime_refoundation_governance.rs"]
mod runtime_refoundation_governance;

#[path = "integration/workflow_layer_run.rs"]
mod workflow_layer_run;

#[path = "integration/workflow_layer_resume.rs"]
mod workflow_layer_resume;

#[path = "integration/workflow_layer_compat.rs"]
mod workflow_layer_compat;

#[path = "integration/workflow_follow_through.rs"]
mod workflow_follow_through;

#[path = "integration/workflow_discovery.rs"]
mod workflow_discovery;

#[path = "integration/workflow_follow_through_compat.rs"]
mod workflow_follow_through_compat;

#[path = "integration/workflow_follow_through_blocked.rs"]
mod workflow_follow_through_blocked;

#[path = "integration/governed_stage_depth_workflow.rs"]
mod governed_stage_depth_workflow;
