use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use synod::{
    ApprovalState, CanonCliRuntime, GovernanceBoundedContext, GovernanceInputDocument,
    GovernanceLifecycleState, GovernanceRequestKind, GovernanceRuntime, GovernanceRuntimeKind,
    GovernanceRuntimeRequest, LocalGovernanceRuntime, PacketReadiness, SystemContextBinding,
    classify_packet_readiness,
};
use uuid::Uuid;

fn sample_request() -> GovernanceRuntimeRequest {
    GovernanceRuntimeRequest {
        request_kind: GovernanceRequestKind::Start,
        governance_attempt_id: "attempt-1".to_string(),
        stage_key: "bug-fix:investigate".to_string(),
        goal: "Investigate a failing change".to_string(),
        workspace_ref: "/tmp/synod-workspace".to_string(),
        autopilot: false,
        mode: None,
        system_context: None,
        risk: None,
        zone: None,
        owner: None,
        run_ref: None,
        packet_ref: None,
        bounded_context: GovernanceBoundedContext {
            read_targets: Vec::new(),
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        input_documents: Vec::new(),
    }
}

fn canon_request(workspace: &Path) -> GovernanceRuntimeRequest {
    let mut request = sample_request();
    request.workspace_ref = workspace.to_string_lossy().into_owned();
    request.mode = Some(synod::CanonMode::Discovery);
    request.system_context = Some(SystemContextBinding::Existing);
    request.risk = Some("medium".to_string());
    request.zone = Some("engineering".to_string());
    request.owner = Some("platform".to_string());
    request
}

fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

fn write_workspace_file(workspace: &Path, relative_path: &str, contents: &str) {
    let path = workspace.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn write_canon_stub(prefix: &str, stdout_json: &str) -> PathBuf {
    let dir = temp_workspace(prefix);
    let script_path = dir.join("canon-stub.sh");
    fs::write(&script_path, format!("#!/bin/sh\ncat >/dev/null\nprintf '%s' '{stdout_json}'\n"))
        .unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    script_path
}

fn write_shell_script(prefix: &str, body: &str) -> PathBuf {
    let dir = temp_workspace(prefix);
    let script_path = dir.join("canon-script.sh");
    fs::write(&script_path, body).unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    script_path
}

#[test]
fn local_governance_runtime_rejects_missing_required_request_fields() {
    let runtime = LocalGovernanceRuntime;

    let mut missing_stage = sample_request();
    missing_stage.stage_key.clear();
    let error = runtime.execute(&missing_stage).unwrap_err();
    assert!(error.to_string().contains("stage_key must not be empty"));

    let mut missing_attempt = sample_request();
    missing_attempt.governance_attempt_id.clear();
    let error = runtime.execute(&missing_attempt).unwrap_err();
    assert!(error.to_string().contains("governance_attempt_id must not be empty"));
}

#[test]
fn local_governance_runtime_blocks_when_no_bounded_context_is_available() {
    let runtime = LocalGovernanceRuntime;

    let response = runtime.execute(&sample_request()).unwrap();

    assert_eq!(runtime.kind(), GovernanceRuntimeKind::Local);
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert_eq!(response.approval_state, ApprovalState::NotNeeded);
    assert!(response.packet.is_none());
    assert!(response.message.contains("no bounded stage context"));
}

#[test]
fn local_governance_runtime_creates_reusable_packet_from_read_targets() {
    let runtime = LocalGovernanceRuntime;
    let mut request = sample_request();
    request.bounded_context.read_targets = vec!["README.md".to_string()];

    let response = runtime.execute(&request).unwrap();
    let packet = response.packet.unwrap();

    assert_eq!(response.status, GovernanceLifecycleState::GovernedReady);
    assert_eq!(packet.runtime, GovernanceRuntimeKind::Local);
    assert_eq!(packet.readiness, PacketReadiness::Reusable);
    assert_eq!(packet.packet_ref, ".synod/governance/bug-fix-investigate/attempt-1");
    assert_eq!(packet.expected_document_refs, vec![format!("{}/brief.md", packet.packet_ref)]);
    assert_eq!(packet.document_refs, vec![format!("{}/brief.md", packet.packet_ref)]);
    assert!(packet.headline.contains("bug-fix:investigate"));
}

#[test]
fn local_governance_runtime_uses_stage_brief_documents_and_explicit_packet_ref() {
    let runtime = LocalGovernanceRuntime;
    let mut request = sample_request();
    request.packet_ref = Some(".canon/packets/investigate-01".to_string());
    request.input_documents.push(GovernanceInputDocument {
        kind: "stage-brief".to_string(),
        path: "docs/stage-brief.md".to_string(),
    });

    let response = runtime.execute(&request).unwrap();
    let packet = response.packet.unwrap();

    assert_eq!(response.status, GovernanceLifecycleState::GovernedReady);
    assert_eq!(packet.packet_ref, ".canon/packets/investigate-01");
    assert_eq!(packet.expected_document_refs, vec!["docs/stage-brief.md".to_string()]);
    assert_eq!(packet.document_refs, vec!["docs/stage-brief.md".to_string()]);
}

#[test]
fn canon_cli_runtime_exposes_configuration_and_reports_unimplemented_execution() {
    let workspace = temp_workspace("synod-canon-config");
    let script = write_canon_stub(
        "synod-canon-config-command",
        "{\"status\":\"failed\",\"approval_state\":\"not_needed\",\"message\":\"unused\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);

    assert_eq!(runtime.kind(), GovernanceRuntimeKind::Canon);
    assert_eq!(runtime.command(), script.to_string_lossy().as_ref());
    assert_eq!(runtime.working_directory(), Some(workspace.as_path()));

    let response = runtime.execute(&canon_request(&workspace)).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Failed);
}

#[test]
fn canon_cli_runtime_parses_start_response_into_a_reusable_packet() {
    let workspace = temp_workspace("synod-canon-start");
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-123/discovery.md",
        "# Discovery\n\nObserved checkout failure in the parser boundary.\n",
    );
    let script = write_canon_stub(
        "synod-canon-start-command",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-123\",\"packet_ref\":\".canon/runs/canon-run-123\",\"expected_document_refs\":[\".canon/runs/canon-run-123/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-123/discovery.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"discovery packet ready\",\"message\":\"Canon completed the governed stage\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);

    let response = runtime.execute(&canon_request(&workspace)).unwrap();
    let packet = response.packet.expect("packet should be present");

    assert_eq!(response.status, GovernanceLifecycleState::GovernedReady);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-123"));
    assert_eq!(packet.runtime, GovernanceRuntimeKind::Canon);
    assert_eq!(packet.readiness, PacketReadiness::Reusable);
    assert_eq!(packet.canon_mode, Some(synod::CanonMode::Discovery));
}

#[test]
fn canon_cli_runtime_tolerates_additive_v1_fields_from_canon_036() {
    let workspace = temp_workspace("synod-canon-additive-v1");
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-036/discovery.md",
        "# Discovery\n\nCaptured governed evidence for the compatibility check.\n",
    );
    let script = write_canon_stub(
        "synod-canon-additive-v1-command",
        "{\"adapter_schema_version\":\"v1\",\"status\":\"governed_ready\",\"run_ref\":\"canon-run-036\",\"packet_ref\":\".canon/runs/canon-run-036\",\"expected_document_refs\":[\".canon/runs/canon-run-036/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-036/discovery.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"discovery packet ready\",\"message\":\"Canon completed the governed stage\",\"reason_code\":\"governed_ready\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);

    let response = runtime.execute(&canon_request(&workspace)).unwrap();
    let packet = response.packet.expect("packet should be present");

    assert_eq!(response.status, GovernanceLifecycleState::GovernedReady);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-036"));
    assert_eq!(response.approval_state, ApprovalState::NotNeeded);
    assert_eq!(packet.packet_ref, ".canon/runs/canon-run-036");
    assert_eq!(packet.readiness, PacketReadiness::Reusable);
}

#[test]
fn canon_cli_runtime_preserves_refresh_lineage_and_pending_state() {
    let workspace = temp_workspace("synod-canon-refresh");
    let script = write_canon_stub(
        "synod-canon-refresh-command",
        "{\"status\":\"awaiting_approval\",\"run_ref\":\"canon-run-456\",\"packet_ref\":\".canon/runs/canon-run-456\",\"expected_document_refs\":[\".canon/runs/canon-run-456/discovery.md\"],\"document_refs\":[],\"approval_state\":\"requested\",\"packet_readiness\":\"pending\",\"missing_sections\":[],\"headline\":\"awaiting approval\",\"message\":\"Canon is waiting for approval\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let mut request = canon_request(&workspace);
    request.request_kind = GovernanceRequestKind::Refresh;
    request.run_ref = Some("canon-run-456".to_string());
    request.packet_ref = Some(".canon/runs/canon-run-456".to_string());

    let response = runtime.execute(&request).unwrap();
    let packet = response.packet.expect("packet should be present");

    assert_eq!(response.status, GovernanceLifecycleState::AwaitingApproval);
    assert_eq!(response.approval_state, ApprovalState::Requested);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-456"));
    assert_eq!(packet.packet_ref, ".canon/runs/canon-run-456");
    assert_eq!(packet.readiness, PacketReadiness::Pending);
}

#[test]
fn canon_cli_runtime_blocks_when_required_request_fields_are_missing() {
    let workspace = temp_workspace("synod-canon-missing-field");
    let script = write_canon_stub(
        "synod-canon-missing-field-command",
        "{\"status\":\"failed\",\"approval_state\":\"not_needed\",\"message\":\"unused\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);

    let response = runtime.execute(&sample_request()).unwrap();

    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert!(response.message.contains("required field 'mode'"));
}

#[test]
fn canon_cli_runtime_validates_each_required_field() {
    let workspace = temp_workspace("synod-canon-required-fields");
    let script = write_canon_stub(
        "synod-canon-required-fields-command",
        "{\"status\":\"failed\",\"approval_state\":\"not_needed\",\"message\":\"unused\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);

    let mut missing_system_context = canon_request(&workspace);
    missing_system_context.system_context = None;
    let response = runtime.execute(&missing_system_context).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert!(response.message.contains("required field 'system_context'"));

    let mut missing_risk = canon_request(&workspace);
    missing_risk.risk = Some(" ".to_string());
    let response = runtime.execute(&missing_risk).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert!(response.message.contains("required field 'risk'"));

    let mut missing_zone = canon_request(&workspace);
    missing_zone.zone = Some(" ".to_string());
    let response = runtime.execute(&missing_zone).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert!(response.message.contains("required field 'zone'"));

    let mut missing_owner = canon_request(&workspace);
    missing_owner.owner = Some(" ".to_string());
    let response = runtime.execute(&missing_owner).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert!(response.message.contains("required field 'owner'"));

    let mut refresh_request = canon_request(&workspace);
    refresh_request.request_kind = GovernanceRequestKind::Refresh;
    refresh_request.run_ref = Some(" ".to_string());
    let response = runtime.execute(&refresh_request).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert!(response.message.contains("required field 'run_ref'"));
}

#[test]
fn canon_cli_runtime_handles_startup_and_output_failures() {
    let workspace = temp_workspace("synod-canon-runtime-failures");

    let missing_runtime =
        CanonCliRuntime::new("/definitely/missing/canon").with_working_directory(&workspace);
    let response = missing_runtime.execute(&canon_request(&workspace)).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Failed);
    assert!(response.message.contains("failed to start Canon"));

    let malformed_script = write_shell_script(
        "synod-canon-malformed-command",
        "#!/bin/sh\ncat >/dev/null\nprintf '%s' '{not-json}'\n",
    );
    let malformed_runtime = CanonCliRuntime::new(malformed_script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = malformed_runtime.execute(&canon_request(&workspace)).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Failed);
    assert!(response.message.contains("malformed output"));

    let failing_script = write_shell_script(
        "synod-canon-failing-command",
        "#!/bin/sh\ncat >/dev/null\necho boom >&2\nexit 2\n",
    );
    let failing_runtime = CanonCliRuntime::new(failing_script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = failing_runtime.execute(&canon_request(&workspace)).unwrap();
    assert_eq!(response.status, GovernanceLifecycleState::Failed);
    assert!(response.message.contains("Canon command failed"));
    assert!(response.message.contains("boom"));
}

#[test]
fn canon_cli_runtime_applies_wire_defaults_for_missing_fields() {
    let workspace = temp_workspace("synod-canon-wire-defaults");
    let script = write_canon_stub(
        "synod-canon-wire-defaults-command",
        "{\"status\":\"awaiting_approval\",\"packet_ref\":\".canon/runs/canon-run-default\",\"message\":\"waiting for approval\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);

    let response = runtime.execute(&canon_request(&workspace)).unwrap();
    let packet = response.packet.expect("packet should be present");

    assert_eq!(response.status, GovernanceLifecycleState::AwaitingApproval);
    assert_eq!(response.approval_state, ApprovalState::NotNeeded);
    assert_eq!(packet.readiness, PacketReadiness::Pending);
    assert_eq!(
        packet.expected_document_refs,
        vec![".canon/runs/canon-run-default/discovery.md".to_string()]
    );
    assert!(packet.document_refs.is_empty());
}

#[test]
fn packet_readiness_classifier_rejects_documents_without_authored_body() {
    let workspace = temp_workspace("synod-packet-readiness-rejected");
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-789/discovery.md",
        "# Discovery\n\nTODO\n",
    );

    let readiness = classify_packet_readiness(
        &workspace,
        &[".canon/runs/canon-run-789/discovery.md".to_string()],
        &[".canon/runs/canon-run-789/discovery.md".to_string()],
        &[],
        PacketReadiness::Reusable,
    );

    assert_eq!(readiness, PacketReadiness::Rejected);
}

#[test]
fn packet_readiness_classifier_marks_missing_documents_as_incomplete() {
    let workspace = temp_workspace("synod-packet-readiness-incomplete");
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-790/discovery.md",
        "# Discovery\n\nObserved concrete failure evidence.\n",
    );

    let readiness = classify_packet_readiness(
        &workspace,
        &[
            ".canon/runs/canon-run-790/discovery.md".to_string(),
            ".canon/runs/canon-run-790/change.md".to_string(),
        ],
        &[".canon/runs/canon-run-790/discovery.md".to_string()],
        &[],
        PacketReadiness::Reusable,
    );

    assert_eq!(readiness, PacketReadiness::Incomplete);
}
