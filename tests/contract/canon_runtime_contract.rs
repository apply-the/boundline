use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use boundline::{
    CanonCliRuntime, GovernanceRuntime, GovernanceRuntimeKind, GovernanceRuntimeRequest,
};
use boundline::{GovernanceBoundedContext, GovernanceRequestKind, SystemContextBinding};
use uuid::Uuid;

fn request() -> GovernanceRuntimeRequest {
    GovernanceRuntimeRequest {
        request_kind: GovernanceRequestKind::Start,
        governance_attempt_id: "canon-contract-attempt".to_string(),
        stage_key: "bug-fix:investigate".to_string(),
        goal: "Investigate a failing change".to_string(),
        workspace_ref: temp_workspace("boundline-governance-contract")
            .to_string_lossy()
            .into_owned(),
        autopilot: false,
        mode: Some(boundline::CanonMode::Discovery),
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
        run_ref: None,
        packet_ref: None,
        bounded_context: GovernanceBoundedContext {
            read_targets: vec!["src/lib.rs".to_string()],
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        input_documents: Vec::new(),
    }
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

#[test]
fn canon_runtime_contract_exposes_configuration_and_parses_start_response() {
    let workspace = temp_workspace("boundline-canon-contract");
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-100/discovery.md",
        "# Discovery\n\nCaptured governed evidence for the bug-fix stage.\n",
    );
    let script = write_canon_stub(
        "boundline-canon-contract-command",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-100\",\"packet_ref\":\".canon/runs/canon-run-100\",\"expected_document_refs\":[\".canon/runs/canon-run-100/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-100/discovery.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"discovery packet ready\",\"message\":\"Canon completed the governed discovery run\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let request = GovernanceRuntimeRequest {
        workspace_ref: workspace.to_string_lossy().into_owned(),
        ..request()
    };

    assert_eq!(runtime.kind(), GovernanceRuntimeKind::Canon);
    assert_eq!(runtime.command(), script.to_string_lossy().as_ref());
    assert_eq!(runtime.working_directory(), Some(workspace.as_path()));

    let response = runtime.execute(&request).unwrap();
    let packet = response.packet.expect("packet should be present");
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-100"));
    assert_eq!(response.status, boundline::GovernanceLifecycleState::GovernedReady);
    assert_eq!(packet.packet_ref, ".canon/runs/canon-run-100");
    assert_eq!(packet.readiness, boundline::PacketReadiness::Reusable);
}

#[test]
fn canon_runtime_contract_sends_refresh_requests_with_lineage_fields() {
    let workspace = temp_workspace("boundline-canon-refresh-contract");
    let capture_path = workspace.join("canon-request.json");
    let script_path = workspace.join("canon-refresh-stub.sh");
    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\nrequest=$(cat)\nprintf '%s' \"$request\" > '{}'\nprintf '%s' '{{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-200\",\"packet_ref\":\".canon/runs/canon-run-200\",\"expected_document_refs\":[\".canon/runs/canon-run-200/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-200/discovery.md\"],\"approval_state\":\"granted\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"refresh packet ready\",\"message\":\"Canon refreshed the governed packet\"}}'\n",
            capture_path.display()
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-200/discovery.md",
        "# Discovery\n\nRefreshed governed packet.\n",
    );

    let runtime = CanonCliRuntime::new(script_path.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = runtime
        .execute(&GovernanceRuntimeRequest {
            request_kind: GovernanceRequestKind::Refresh,
            governance_attempt_id: "canon-contract-refresh".to_string(),
            stage_key: "bug-fix:investigate".to_string(),
            goal: "Investigate a failing change".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            autopilot: true,
            mode: Some(boundline::CanonMode::Discovery),
            system_context: Some(SystemContextBinding::Existing),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            run_ref: Some("canon-run-200".to_string()),
            packet_ref: Some(".canon/runs/canon-run-200".to_string()),
            bounded_context: GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            input_documents: Vec::new(),
        })
        .unwrap();

    let request_json = fs::read_to_string(capture_path).unwrap();
    assert!(request_json.contains("\"request_kind\":\"refresh\""), "{request_json}");
    assert!(request_json.contains("\"run_ref\":\"canon-run-200\""), "{request_json}");
    assert!(
        request_json.contains("\"packet_ref\":\".canon/runs/canon-run-200\""),
        "{request_json}"
    );
    assert_eq!(response.approval_state, boundline::ApprovalState::Granted);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-200"));
}

#[test]
fn canon_runtime_contract_marks_scaffold_packets_as_rejected() {
    let workspace = temp_workspace("boundline-canon-rejected-contract");
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-300/discovery.md",
        "# Discovery\n\nTODO\n",
    );
    let script = write_canon_stub(
        "boundline-canon-rejected-command",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-300\",\"packet_ref\":\".canon/runs/canon-run-300\",\"expected_document_refs\":[\".canon/runs/canon-run-300/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-300/discovery.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"discovery packet ready\",\"message\":\"Canon completed the governed discovery run\"}",
    );
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let request = GovernanceRuntimeRequest {
        workspace_ref: workspace.to_string_lossy().into_owned(),
        ..request()
    };

    let response = runtime.execute(&request).unwrap();
    let packet = response.packet.expect("packet should be present");
    assert_eq!(packet.readiness, boundline::PacketReadiness::Rejected);
    assert_eq!(packet.missing_sections, vec!["substantive_body".to_string()]);
}

#[test]
fn canon_runtime_contract_serializes_security_assessment_mode_in_start_requests() {
    let workspace = temp_workspace("boundline-canon-security-start-contract");
    let capture_path = workspace.join("canon-security-request.json");
    let script_path = workspace.join("canon-security-stub.sh");
    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\nrequest=$(cat)\nprintf '%s' \"$request\" > '{}'\nprintf '%s' '{{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-security\",\"packet_ref\":\".canon/runs/canon-run-security\",\"expected_document_refs\":[\".canon/runs/canon-run-security/security-assessment.md\"],\"document_refs\":[\".canon/runs/canon-run-security/security-assessment.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"security assessment packet ready\",\"message\":\"Canon completed the governed security assessment\"}}'\n",
            capture_path.display()
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-security/security-assessment.md",
        "# Security Assessment\n\nValidated the bounded security review.\n",
    );

    let runtime = CanonCliRuntime::new(script_path.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = runtime
        .execute(&GovernanceRuntimeRequest {
            stage_key: "bug-fix:verify".to_string(),
            mode: Some(boundline::CanonMode::SecurityAssessment),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            ..request()
        })
        .unwrap();

    let request_json = fs::read_to_string(capture_path).unwrap();
    assert!(request_json.contains("\"stage_key\":\"bug-fix:verify\""), "{request_json}");
    assert!(request_json.contains("\"mode\":\"security-assessment\""), "{request_json}");
    let packet = response.packet.expect("packet should be present");
    assert_eq!(packet.packet_ref, ".canon/runs/canon-run-security");
    assert_eq!(packet.readiness, boundline::PacketReadiness::Reusable);
}

#[test]
fn canon_runtime_contract_preserves_security_assessment_refresh_lineage() {
    let workspace = temp_workspace("boundline-canon-security-refresh-contract");
    let capture_path = workspace.join("canon-security-refresh-request.json");
    let script_path = workspace.join("canon-security-refresh-stub.sh");
    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\nrequest=$(cat)\nprintf '%s' \"$request\" > '{}'\nprintf '%s' '{{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-security-refresh\",\"packet_ref\":\".canon/runs/canon-run-security-refresh\",\"expected_document_refs\":[\".canon/runs/canon-run-security-refresh/security-assessment.md\"],\"document_refs\":[\".canon/runs/canon-run-security-refresh/security-assessment.md\"],\"approval_state\":\"granted\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"security refresh packet ready\",\"message\":\"Canon refreshed the governed security packet\"}}'\n",
            capture_path.display()
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-security-refresh/security-assessment.md",
        "# Security Assessment\n\nRefreshed bounded security review.\n",
    );

    let runtime = CanonCliRuntime::new(script_path.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = runtime
        .execute(&GovernanceRuntimeRequest {
            request_kind: GovernanceRequestKind::Refresh,
            governance_attempt_id: "canon-contract-security-refresh".to_string(),
            stage_key: "bug-fix:verify".to_string(),
            goal: "Verify the bounded security fix".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            autopilot: true,
            mode: Some(boundline::CanonMode::SecurityAssessment),
            system_context: Some(SystemContextBinding::Existing),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            run_ref: Some("canon-run-security-refresh".to_string()),
            packet_ref: Some(".canon/runs/canon-run-security-refresh".to_string()),
            bounded_context: GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            input_documents: Vec::new(),
        })
        .unwrap();

    let request_json = fs::read_to_string(capture_path).unwrap();
    assert!(request_json.contains("\"request_kind\":\"refresh\""), "{request_json}");
    assert!(request_json.contains("\"mode\":\"security-assessment\""), "{request_json}");
    assert!(request_json.contains("\"run_ref\":\"canon-run-security-refresh\""), "{request_json}");
    assert_eq!(response.approval_state, boundline::ApprovalState::Granted);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-security-refresh"));
}
