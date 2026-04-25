#[path = "support/workspace_fixture.rs"]
mod workspace_fixture;

#[path = "contract/orchestrator_run.rs"]
mod orchestrator_run;

#[path = "contract/endpoint_execution.rs"]
mod endpoint_execution;

#[path = "contract/trace_record.rs"]
mod trace_record;

#[path = "contract/cli_command_contract.rs"]
mod cli_command_contract;

#[path = "contract/diagnostics_report_contract.rs"]
mod diagnostics_report_contract;

#[path = "contract/trace_summary_contract.rs"]
mod trace_summary_contract;

#[path = "contract/assistant_command_pack_contract.rs"]
mod assistant_command_pack_contract;

#[path = "contract/assistant_command_definition_contract.rs"]
mod assistant_command_definition_contract;

#[path = "contract/session_record_contract.rs"]
mod session_record_contract;

#[path = "contract/session_command_contract.rs"]
mod session_command_contract;

#[path = "contract/assistant_session_continuity_contract.rs"]
mod assistant_session_continuity_contract;

#[path = "contract/flow_command_contract.rs"]
mod flow_command_contract;

#[path = "contract/flow_session_contract.rs"]
mod flow_session_contract;

#[path = "contract/flow_status_contract.rs"]
mod flow_status_contract;
