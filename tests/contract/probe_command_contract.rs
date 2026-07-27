use std::fs;
use std::sync::Mutex;

use boundline::adapters::env_layer::OPENAI_API_KEY_ENV;
use boundline::cli::{probe::execute_probe, workspace::resolve_workspace};
use serde_json::Value;

use crate::workspace_fixture::{PROVIDER_ENV_KEYS, temp_empty_workspace, temp_fixture_workspace};

static PROCESS_STATE: Mutex<()> = Mutex::new(());

fn report(workspace: &std::path::Path) -> Result<Value, String> {
    serde_json::to_value(execute_probe(workspace)).map_err(|error| error.to_string())
}

struct ProviderEnvGuard(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl ProviderEnvGuard {
    fn cleared() -> Self {
        let previous =
            PROVIDER_ENV_KEYS.iter().map(|key| (*key, std::env::var_os(key))).collect::<Vec<_>>();
        for key in PROVIDER_ENV_KEYS {
            unsafe {
                std::env::remove_var(key);
            }
        }
        Self(previous)
    }
}

impl Drop for ProviderEnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.0 {
            unsafe {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

#[test]
fn probe_plain_output_reports_bootstrap_state_without_assistant_routes() -> Result<(), String> {
    let value = report(&temp_empty_workspace("boundline-probe-contract-bootstrap"))?;
    if value["workspace"]["initialized"] != Value::Bool(false)
        || value["recommended_next"]["command"] != Value::String("boundline init".into())
    {
        return Err(format!("internal probe bootstrap projection changed: {value}"));
    }
    Ok(())
}

#[test]
fn probe_host_json_output_wraps_the_rendered_probe_report() -> Result<(), String> {
    let value = report(&temp_empty_workspace("boundline-probe-contract-json"))?;
    if !value.is_object() || value["recommended_next"]["command"] != "boundline init" {
        return Err(format!("internal probe serialization changed: {value}"));
    }
    Ok(())
}

#[test]
fn probe_can_resolve_the_current_workspace_when_flag_is_omitted() -> Result<(), String> {
    let _lock = PROCESS_STATE.lock().map_err(|error| error.to_string())?;
    let workspace = temp_fixture_workspace("boundline-probe-contract-auto-workspace");
    let original = std::env::current_dir().map_err(|error| error.to_string())?;
    std::env::set_current_dir(&workspace).map_err(|error| error.to_string())?;
    let resolved = resolve_workspace(None).map_err(|error| error.to_string());
    std::env::set_current_dir(original).map_err(|error| error.to_string())?;
    let resolved = resolved?;
    let value = report(&resolved)?;
    if value["workspace"]["initialized"] != Value::Bool(true) {
        return Err(format!("internal probe cwd resolution changed: {value}"));
    }
    Ok(())
}

#[test]
fn probe_initialized_workspace_without_provider_credentials_routes_to_doctor() -> Result<(), String>
{
    let _lock = PROCESS_STATE.lock().map_err(|error| error.to_string())?;
    let _environment = ProviderEnvGuard::cleared();
    let value = report(&temp_fixture_workspace("boundline-probe-contract-doctor"))?;
    if value["providers"]["healthy"] != Value::Bool(false)
        || value["recommended_next"]["command"] != "boundline doctor"
    {
        return Err(format!("internal probe provider fallback changed: {value}"));
    }
    Ok(())
}

#[test]
fn probe_initialized_workspace_with_provider_credentials_routes_to_goal() -> Result<(), String> {
    let _lock = PROCESS_STATE.lock().map_err(|error| error.to_string())?;
    let _environment = ProviderEnvGuard::cleared();
    let workspace = temp_fixture_workspace("boundline-probe-contract-goal");
    unsafe {
        std::env::set_var(OPENAI_API_KEY_ENV, "probe-test");
    }
    let value = report(&workspace)?;
    if value["providers"]["healthy"] != Value::Bool(true)
        || value["recommended_next"]["command"] != "boundline goal"
    {
        return Err(format!("internal probe healthy-provider routing changed: {value}"));
    }
    Ok(())
}

#[test]
fn probe_reports_semantic_index_health_for_corrupt_index() -> Result<(), String> {
    let workspace = temp_empty_workspace("boundline-probe-contract-semantic-health");
    let index_dir = workspace.join(".boundline/context-intelligence");
    fs::create_dir_all(&index_dir).map_err(|error| error.to_string())?;
    fs::write(index_dir.join("retrieval-index.sqlite3"), b"fake-db")
        .map_err(|error| error.to_string())?;
    let value = report(&workspace)?;
    if value["capabilities"]["semantic_index"] != Value::Bool(true)
        || value["capabilities"]["semantic_index_health"] != "failed"
    {
        return Err(format!("internal probe semantic-health projection changed: {value}"));
    }
    Ok(())
}
