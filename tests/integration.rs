#[path = "support/workspace_fixture.rs"]
mod workspace_fixture;

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

#[path = "integration/flow_cli_run.rs"]
mod flow_cli_run;

#[path = "integration/session_governance_flow.rs"]
mod session_governance_flow;

#[path = "integration/canon_governance_flow.rs"]
mod canon_governance_flow;

#[path = "integration/governance_autopilot_flow.rs"]
mod governance_autopilot_flow;
