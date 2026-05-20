#[path = "../../../src/cli/assistant_assets.rs"]
pub mod assistant_assets;
#[path = "../../../src/cli/checkpoint.rs"]
pub mod checkpoint;
#[path = "../../../src/cli/cluster.rs"]
pub mod cluster;
#[path = "../../../src/cli/config.rs"]
pub mod config;
#[path = "../../../src/cli/dashboard.rs"]
pub mod dashboard;
#[path = "../../../src/cli/diagnostics.rs"]
pub mod diagnostics;
#[path = "../../../src/cli/govern.rs"]
pub mod govern;
#[path = "../../../src/cli/init.rs"]
pub mod init;
#[path = "../../../src/cli/inspect.rs"]
pub mod inspect;
#[path = "../../../src/cli/output.rs"]
pub mod output;
#[path = "../../../src/cli/run.rs"]
pub mod run;
#[path = "../../../src/cli/session.rs"]
pub mod session;
#[path = "../../../src/cli/workflow.rs"]
pub mod workflow;
#[path = "../../../src/cli/workspace.rs"]
pub mod workspace;

#[path = "../../../src/cli.rs"]
mod cli_impl;

pub use cli_impl::*;
