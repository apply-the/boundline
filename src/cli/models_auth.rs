use thiserror::Error;

use crate::adapters::auth_profile_store::{
    self, AuthProfileStoreError, load_auth_profiles, save_auth_profiles,
};
use crate::adapters::github_device_flow::{self, DeviceFlowError};
use crate::cli::CommandExitStatus;
use crate::domain::trace::current_timestamp_millis;

const GITHUB_COPILOT_PROVIDER_KEY: &str = "github-copilot";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelsAuthReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

#[derive(Debug, Error)]
pub enum ModelsAuthError {
    #[error("device flow failed: {0}")]
    DeviceFlow(#[from] DeviceFlowError),
    #[error("auth profile store error: {0}")]
    ProfileStore(#[from] AuthProfileStoreError),
    #[error("unsupported provider: {provider}")]
    UnsupportedProvider { provider: String },
}

pub fn execute_login(provider: &str) -> Result<ModelsAuthReport, ModelsAuthError> {
    if provider != GITHUB_COPILOT_PROVIDER_KEY {
        return Err(ModelsAuthError::UnsupportedProvider { provider: provider.to_string() });
    }

    let login_result = github_device_flow::execute_device_login()?;

    let mut store = load_auth_profiles()?;
    let now_millis = current_timestamp_millis();
    let obtained_at = format!("{now_millis}");
    store.set_token(provider, login_result.token, obtained_at);
    let profile_path = save_auth_profiles(&store)?;

    Ok(ModelsAuthReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!(
            "Authenticated with {provider}. Token stored at {}.",
            profile_path.display()
        ),
    })
}

pub fn execute_status() -> Result<ModelsAuthReport, ModelsAuthError> {
    let store = load_auth_profiles()?;
    let providers = store.list_providers();

    let output = if providers.is_empty() {
        "No providers authenticated. Run `boundline models auth login` to authenticate.".to_string()
    } else {
        let mut lines = vec!["Authenticated providers:".to_string()];
        for provider in providers {
            lines.push(format!("  - {provider}"));
        }
        lines.push(String::new());
        lines.push(format!(
            "Profile stored at: {}",
            auth_profile_store::auth_profiles_path().display()
        ));
        lines.join("\n")
    };

    Ok(ModelsAuthReport { exit_status: CommandExitStatus::Succeeded, terminal_output: output })
}

pub fn execute_remove(provider: &str) -> Result<ModelsAuthReport, ModelsAuthError> {
    let mut store = load_auth_profiles()?;
    let removed = store.remove_provider(provider);

    if removed {
        save_auth_profiles(&store)?;
        Ok(ModelsAuthReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!("Removed authentication for {provider}."),
        })
    } else {
        Ok(ModelsAuthReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: format!("No stored authentication found for {provider}."),
        })
    }
}
