use std::fs;
use std::os::unix::fs::PermissionsExt;

use boundline::cli::provider::{AddProviderRequest, execute_add};
use boundline::domain::capability_provider::{
    ProviderExecutionRequest, ProviderPermissionEnvelope,
};
use boundline::orchestrator::capability_provider_runtime::execute_provider;

use crate::workspace_fixture::temp_git_workspace;

const EXECUTION_PROVIDER_SCRIPT: &str = concat!(
    "#!/usr/bin/env python3\n",
    "import json, sys\n",
    "op = sys.argv[1]\n",
    "payload = json.load(sys.stdin)\n",
    "if op == 'capabilities':\n",
    "  print(json.dumps({'declarations':[{'provider_id':'demo-provider','protocol_line':'capability-provider-v1','protocol_version':'1.0.0','capability_id':'demo.fetch','supported_lifecycle_phases':['run'],'supported_inputs':['context_pack'],'supported_outputs':['artifact'],'mutation_support':'proposal_only','required_permissions':['read_files'],'evidence_formats':['ref']}]}))\n",
    "elif op == 'health':\n",
    "  print(json.dumps({'provider_id':'demo-provider','readiness_state':'ready','missing_dependencies':[],'warnings':[],'runtime_environment':['local'],'checked_at':1}))\n",
    "elif op == 'prepare':\n",
    "  request = payload['request']\n",
    "  missing = []\n",
    "  if request['request_id'] == 'missing-evidence':\n",
    "    missing = ['canon://missing']\n",
    "  print(json.dumps({'request_id':request['request_id'],'required_context_refs':[],'optional_context_refs':[],'missing_evidence_refs':missing,'expected_artifacts':['artifact.md'],'risk_observations':[],'estimated_cost_or_runtime':None}))\n",
    "elif op == 'execute':\n",
    "  request = payload['request']\n",
    "  proposals = []\n",
    "  evidence = ['provider://evidence/demo']\n",
    "  if request['request_id'] == 'patch-proposal':\n",
    "    proposals = ['replace src/lib.rs block']\n",
    "  if request['request_id'] == 'no-evidence':\n",
    "    evidence = []\n",
    "  print(json.dumps({'request_id':request['request_id'],'status':'succeeded','observations':['completed'],'findings':['found item'],'artifact_refs':['artifact.md'],'evidence_refs':evidence,'state_patch_proposals':proposals,'limitations':['bounded'],'next_actions':['review']}))\n",
    "elif op == 'collect_evidence':\n",
    "  result = payload['execution_result']\n",
    "  print(json.dumps({'request_id':payload['request_id'],'claims':['claim'],'evidence_refs':result['evidence_refs'],'artifact_refs':result['artifact_refs'],'findings':result['findings'],'limitations':result['limitations'],'reproducibility_metadata':['replayable']}))\n",
    "else:\n",
    "  raise SystemExit(1)\n",
);

fn write_execution_provider_script() -> std::path::PathBuf {
    let root =
        std::env::temp_dir().join(format!("boundline-provider-execution-{}", uuid::Uuid::new_v4()));
    let create_dir = fs::create_dir_all(&root);
    assert!(create_dir.is_ok());
    let script_path = root.join("provider.py");
    let write_result = fs::write(&script_path, EXECUTION_PROVIDER_SCRIPT);
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

fn register_demo_provider(workspace: &std::path::Path, script_path: &std::path::Path) {
    let report = execute_add(AddProviderRequest {
        provider_id: "demo-provider",
        display_name: Some("Execution Provider"),
        workspace: Some(workspace),
        command: Some("python3"),
        endpoint: None,
        arg: &[script_path.to_string_lossy().into_owned()],
        config_ref: &[],
        secret_handle: &[],
        require_config: &[],
        require_secret: &[],
    });
    assert_eq!(report.exit_status, boundline::cli::CommandExitStatus::Succeeded);
}

fn sample_request(request_id: &str) -> ProviderExecutionRequest {
    ProviderExecutionRequest {
        request_id: request_id.to_string(),
        session_ref: "session-provider".to_string(),
        step_or_stage_ref: "run".to_string(),
        capability_id: "demo.fetch".to_string(),
        goal_summary: "Collect bounded provider evidence".to_string(),
        lifecycle_phase: "run".to_string(),
        authority_zone: "workspace".to_string(),
        context_pack_refs: vec!["context://workspace".to_string()],
        permission_envelope: ProviderPermissionEnvelope {
            read_files: true,
            write_files: false,
            run_commands: false,
            network: false,
            read_secrets: false,
            write_artifacts: true,
            allowed_paths: vec!["src/".to_string()],
            max_runtime_ms: 1000,
            max_output_bytes: 4096,
        },
        expected_outputs: vec!["artifact".to_string()],
    }
}

#[test]
fn provider_execution_accepts_evidence_when_no_patch_proposals_exist() {
    let workspace = temp_git_workspace("boundline-provider-execution-accepted");
    let script_path = write_execution_provider_script();
    register_demo_provider(workspace.path(), &script_path);

    let outcome = execute_provider(workspace.path(), &sample_request("accepted"));
    assert!(outcome.is_ok());
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => panic!("{error}"),
    };
    assert_eq!(
        outcome.trace_record.validation.disposition,
        boundline::domain::capability_provider::ProviderValidationOutcome::Accepted
    );
    assert_eq!(
        outcome.session_record.projection.accepted_evidence_refs,
        vec!["provider://evidence/demo".to_string()]
    );
}

#[test]
fn provider_execution_rejects_patch_proposals_until_host_validation() {
    let workspace = temp_git_workspace("boundline-provider-execution-patch");
    let script_path = write_execution_provider_script();
    register_demo_provider(workspace.path(), &script_path);

    let outcome = execute_provider(workspace.path(), &sample_request("patch-proposal"));
    assert!(outcome.is_ok());
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => panic!("{error}"),
    };
    assert_eq!(
        outcome.trace_record.validation.disposition,
        boundline::domain::capability_provider::ProviderValidationOutcome::Rejected
    );
    assert_eq!(
        outcome.trace_record.validation.failure_class,
        Some(boundline::domain::capability_provider::ProviderFailureClass::PostExecutionValidation)
    );
}

#[test]
fn provider_prepare_missing_evidence_blocks_before_execute() {
    let workspace = temp_git_workspace("boundline-provider-execution-missing-evidence");
    let script_path = write_execution_provider_script();
    register_demo_provider(workspace.path(), &script_path);

    let outcome = execute_provider(workspace.path(), &sample_request("missing-evidence"));
    assert!(outcome.is_ok());
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => panic!("{error}"),
    };
    assert_eq!(
        outcome.trace_record.validation.disposition,
        boundline::domain::capability_provider::ProviderValidationOutcome::Blocked
    );
    assert_eq!(
        outcome.trace_record.validation.failure_class,
        Some(boundline::domain::capability_provider::ProviderFailureClass::Readiness)
    );
}
