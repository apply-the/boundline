//! Browser validation CLI support for Boundline.
//!
//! Provides the programmatic dispatch surface for `boundline validate browser`.
//! The clap subcommand wire-up is deferred to a follow-up PR to avoid needing
//! changes across 30+ match arms in the top-level CLI dispatcher.

use boundline_adapters::browser_artifact_store::BrowserArtifactStore;
use boundline_adapters::browser_provider_runtime::BrowserProviderRuntime;
use boundline_core::domain::browser_provider::{
    BrowserValidationStep, ReadinessLocator, ValidationTimeouts,
};
use std::path::Path;
use uuid::Uuid;

/// Parameters for a browser validation run.
pub struct ValidateBrowserParams<'a> {
    pub provider_command: &'a str,
    pub provider_args: &'a [String],
    pub url: &'a str,
    pub readiness_selector: Option<&'a str>,
    pub readiness_state: Option<&'a str>,
    pub readiness_timeout: Option<u32>,
    pub session_id: &'a str,
    pub workspace_root: &'a Path,
}

/// Run a browser validation step against a target URL.
///
/// Spawns the configured browser provider, dispatches the validation
/// step, writes artifacts to the session-scoped directory, and returns
/// the evidence packet.
///
/// # Errors
///
/// Returns an error message string suitable for terminal output if the
/// provider cannot be started, the handshake fails, or the dispatch fails.
pub fn run_validate_browser(params: &ValidateBrowserParams<'_>) -> Result<String, String> {
    let run_id = Uuid::new_v4().to_string();
    let artifact_dir = params
        .workspace_root
        .join(".boundline")
        .join("sessions")
        .join(params.session_id)
        .join("browser")
        .join(&run_id);

    let readiness = params.readiness_selector.map(|selector| {
        let state = match params.readiness_state.unwrap_or("visible") {
            "attached" => boundline_core::domain::browser_provider::LocatorState::Attached,
            "hidden" => boundline_core::domain::browser_provider::LocatorState::Hidden,
            "detached" => boundline_core::domain::browser_provider::LocatorState::Detached,
            _ => boundline_core::domain::browser_provider::LocatorState::Visible,
        };
        ReadinessLocator {
            locator_type: boundline_core::domain::browser_provider::LocatorType::CssSelector,
            value: selector.to_string(),
            state,
            timeout_seconds: params.readiness_timeout.unwrap_or(20),
            stabilization_delay_ms: Some(250),
        }
    });

    let step = BrowserValidationStep {
        validation_run_id: run_id.clone(),
        url: params.url.to_string(),
        readiness,
        interaction_script: None,
        accessibility_enabled: false,
        dom_inspection: None,
        baseline_ref: None,
        timeouts: ValidationTimeouts::default(),
        network_allowlist: None,
        artifact_dir: artifact_dir.to_string_lossy().to_string(),
        session_id: params.session_id.to_string(),
    };

    let mut runtime =
        BrowserProviderRuntime::new(params.provider_command, params.provider_args, 10);
    runtime.start().map_err(|e| e.to_string())?;

    let packet = runtime.dispatch(&step).map_err(|e| e.to_string())?;

    let store = BrowserArtifactStore::new(&artifact_dir).map_err(|e| e.to_string())?;
    let _evidence_ref = store.write_evidence_packet(&packet, &run_id).map_err(|e| e.to_string())?;

    let mut output =
        format!("validation run {} completed with status {:?}\n", run_id, packet.status);
    output
        .push_str(&format!("  page title: {}\n", packet.page_title.as_deref().unwrap_or("(none)")));
    output.push_str(&format!(
        "  http status: {}\n",
        packet.http_status.map_or("(none)".into(), |s| s.to_string())
    ));
    output.push_str(&format!("  artifacts: {} files\n", packet.artifacts.len()));
    output.push_str(&format!("  findings:  {}\n", packet.findings.len()));
    output.push_str(&format!("  duration:  {}ms\n", packet.timing.total_ms));

    for finding in &packet.findings {
        output.push_str(&format!(
            "    [{:?}] {:?}: {}\n",
            finding.severity, finding.kind, finding.message
        ));
    }

    Ok(output)
}
