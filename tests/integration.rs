#[path = "integration/sequential_task_run.rs"]
mod sequential_task_run;

#[path = "integration/retry_and_replan.rs"]
mod retry_and_replan;

#[path = "integration/trace_capture.rs"]
mod trace_capture;

#[path = "integration/cli_demo_flow.rs"]
mod cli_demo_flow;

#[path = "integration/cli_custom_run.rs"]
mod cli_custom_run;

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

#[path = "integration/session_recovery.rs"]
mod session_recovery;

#[path = "integration/run_demo_flow.rs"]
mod run_demo_flow;

#[path = "integration/run_demo_edge_cases.rs"]
mod run_demo_edge_cases;
