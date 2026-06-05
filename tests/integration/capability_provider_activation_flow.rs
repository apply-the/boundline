use std::fs;
use std::os::unix::fs::PermissionsExt;

use boundline::cli::provider::{
    AddProviderRequest, ShowProviderRequest, execute_add, execute_show,
};
use boundline::domain::configuration::PersistedCapabilityProviderConfiguration;

use crate::workspace_fixture::temp_git_workspace;

const PROVIDER_SCRIPT: &str = concat!(
    "#!/usr/bin/env python3\n",
    "import json, sys\n",
    "op = sys.argv[1]\n",
    "payload = json.load(sys.stdin)\n",
    "if op == 'capabilities':\n",
    "  print(json.dumps({'declarations':[{'provider_id':'demo-provider','protocol_line':'capability-provider-v1','protocol_version':'1.0.0','capability_id':'demo.fetch','supported_lifecycle_phases':['plan','run'],'supported_inputs':['context_pack'],'supported_outputs':['artifact'],'mutation_support':'proposal_only','required_permissions':['read_files'],'evidence_formats':['ref']}]}))\n",
    "elif op == 'health':\n",
    "  print(json.dumps({'provider_id':'demo-provider','readiness_state':'ready','missing_dependencies':[],'warnings':[],'runtime_environment':['local'],'checked_at':1}))\n",
    "else:\n",
    "  print(json.dumps({'request_id':'noop','required_context_refs':[],'optional_context_refs':[],'missing_evidence_refs':[],'expected_artifacts':[],'risk_observations':[],'estimated_cost_or_runtime':None}))\n",
);

fn write_provider_script() -> std::path::PathBuf {
    let root =
        std::env::temp_dir().join(format!("boundline-provider-script-{}", uuid::Uuid::new_v4()));
    let create_dir = fs::create_dir_all(&root);
    assert!(create_dir.is_ok());
    let script_path = root.join("provider.py");
    let write_result = fs::write(&script_path, PROVIDER_SCRIPT);
    assert!(write_result.is_ok());
    let metadata_result = fs::metadata(&script_path);
    assert!(metadata_result.is_ok());
    if let Ok(metadata) = metadata_result {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        let permission_result = fs::set_permissions(&script_path, permissions);
        assert!(permission_result.is_ok());
    }
    script_path
}

fn load_provider_config(
    workspace: &std::path::Path,
) -> Option<PersistedCapabilityProviderConfiguration> {
    let store = boundline::FileConfigStore::for_workspace(workspace);
    store.local_capability_provider().unwrap_or_default()
}

#[test]
fn provider_add_activates_when_setup_is_complete() {
    let workspace = temp_git_workspace("boundline-provider-activation-ready");
    let script_path = write_provider_script();

    let report = execute_add(AddProviderRequest {
        provider_id: "demo-provider",
        display_name: Some("Demo Provider"),
        workspace: Some(workspace.path()),
        command: Some("python3"),
        endpoint: None,
        arg: &[script_path.to_string_lossy().into_owned()],
        config_ref: &[],
        secret_handle: &[],
        require_config: &[],
        require_secret: &[],
    });

    assert_eq!(report.exit_status, boundline::cli::CommandExitStatus::Succeeded);
    assert!(report.terminal_output.contains("provider_status: active"));
    let configuration = load_provider_config(workspace.path());
    assert!(configuration.is_some());
    let configuration = configuration.unwrap_or(PersistedCapabilityProviderConfiguration {
        registrations: Vec::new(),
        active_provider_id: None,
        last_validated_at: None,
    });
    assert_eq!(configuration.active_provider_id.as_deref(), Some("demo-provider"));
    assert_eq!(configuration.registrations.len(), 1);
    assert_eq!(configuration.registrations[0].capability_ids, vec!["demo.fetch".to_string()]);
}

#[test]
fn blocked_setup_preserves_previous_active_provider() {
    let workspace = temp_git_workspace("boundline-provider-activation-blocked");
    let script_path = write_provider_script();

    let ready_report = execute_add(AddProviderRequest {
        provider_id: "ready-provider",
        display_name: Some("Ready Provider"),
        workspace: Some(workspace.path()),
        command: Some("python3"),
        endpoint: None,
        arg: &[script_path.to_string_lossy().into_owned()],
        config_ref: &[],
        secret_handle: &[],
        require_config: &[],
        require_secret: &[],
    });
    assert_eq!(ready_report.exit_status, boundline::cli::CommandExitStatus::Succeeded);

    let blocked_report = execute_add(AddProviderRequest {
        provider_id: "blocked-provider",
        display_name: Some("Blocked Provider"),
        workspace: Some(workspace.path()),
        command: Some("python3"),
        endpoint: None,
        arg: &[script_path.to_string_lossy().into_owned()],
        config_ref: &[],
        secret_handle: &[],
        require_config: &[],
        require_secret: &["api_token".to_string()],
    });
    assert_eq!(blocked_report.exit_status, boundline::cli::CommandExitStatus::Succeeded);
    assert!(blocked_report.terminal_output.contains("provider_status: blocked"));

    let configuration = load_provider_config(workspace.path());
    assert!(configuration.is_some());
    let configuration = configuration.unwrap_or(PersistedCapabilityProviderConfiguration {
        registrations: Vec::new(),
        active_provider_id: None,
        last_validated_at: None,
    });
    assert_eq!(configuration.active_provider_id.as_deref(), Some("ready-provider"));

    let show_report = execute_show(ShowProviderRequest { workspace: Some(workspace.path()) });
    assert_eq!(show_report.exit_status, boundline::cli::CommandExitStatus::Succeeded);
    assert!(show_report.terminal_output.contains("provider_id: ready-provider"));
    assert!(show_report.terminal_output.contains("provider_id: blocked-provider"));
}
