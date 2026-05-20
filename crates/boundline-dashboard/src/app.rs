use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use boundline_adapters::adapters::dashboard_state::DashboardStateAssembler;

use crate::render::{RenderMode, RenderOptions, render_snapshot};
use crate::state::TerminalCapabilities;

#[derive(Debug, Parser)]
#[command(name = "boundline-dashboard")]
#[command(about = "Interactive Boundline delivery dashboard")]
#[command(version)]
pub struct DashboardCli {
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    #[arg(long)]
    pub no_color: bool,
    #[arg(long)]
    pub snapshot_json: bool,
}

pub fn run_from_cli() -> ExitCode {
    let cli = DashboardCli::parse();
    match run(cli) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

pub fn run(cli: DashboardCli) -> Result<String, String> {
    let capabilities = TerminalCapabilities::detect(cli.no_color);
    let workspace = match cli.workspace {
        Some(path) => path,
        None => std::env::current_dir().map_err(|error| error.to_string())?,
    };
    let snapshot = DashboardStateAssembler::for_workspace(&workspace)
        .snapshot(cli.no_color)
        .map_err(|error| error.to_string())?;
    if cli.snapshot_json {
        return serde_json::to_string_pretty(&snapshot)
            .map(|json| format!("{json}\n"))
            .map_err(|error| error.to_string());
    }
    let mode = if snapshot.degraded_state.is_some() {
        RenderMode::Degraded
    } else if capabilities.color {
        RenderMode::Interactive
    } else {
        RenderMode::Monochrome
    };
    Ok(render_snapshot(
        &snapshot,
        RenderOptions { mode, width: 80, height: 24, color: capabilities.color },
    ))
}
