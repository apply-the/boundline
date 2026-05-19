use std::error::Error;
use std::fs;
use std::io;
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
        workspace_ref: "/tmp/boundline-governance-contract".to_string(),
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

fn temp_workspace(prefix: &str) -> io::Result<PathBuf> {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)?;
    Ok(workspace)
}

fn write_workspace_file(workspace: &Path, relative_path: &str, contents: &str) -> io::Result<()> {
    let path = workspace.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

fn make_executable(path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

fn write_canon_stub(prefix: &str, stdout_json: &str) -> io::Result<PathBuf> {
    let dir = temp_workspace(prefix)?;
    let script_path = dir.join("canon-stub.sh");
    fs::write(&script_path, format!("#!/bin/sh\ncat >/dev/null\nprintf '%s' '{stdout_json}'\n"))?;
    make_executable(&script_path)?;
    Ok(script_path)
}

fn write_canon_capture_stub(
    workspace: &Path,
    script_name: &str,
    capture_file_name: &str,
    stdout_json: &str,
) -> io::Result<PathBuf> {
    let script_path = workspace.join(script_name);
    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\nrequest=$(cat)\nprintf '%s' \"$request\" > './{capture_file_name}'\nprintf '%s' '{stdout_json}'\n"
        ),
    )?;
    make_executable(&script_path)?;
    Ok(script_path)
}

#[test]
fn canon_runtime_contract_exposes_configuration_and_parses_start_response()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-canon-contract")?;
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-100/discovery.md",
        "# Discovery\n\nCaptured governed evidence for the bug-fix stage.\n",
    )?;
    let script = write_canon_stub(
        "boundline-canon-contract-command",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-100\",\"packet_ref\":\".canon/runs/canon-run-100\",\"expected_document_refs\":[\".canon/runs/canon-run-100/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-100/discovery.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"discovery packet ready\",\"message\":\"Canon completed the governed discovery run\"}",
    )?;
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let request = GovernanceRuntimeRequest {
        workspace_ref: workspace.to_string_lossy().into_owned(),
        ..request()
    };

    assert_eq!(runtime.kind(), GovernanceRuntimeKind::Canon);
    assert_eq!(runtime.command(), script.to_string_lossy().as_ref());
    assert_eq!(runtime.working_directory(), Some(workspace.as_path()));

    let response = runtime.execute(&request)?;
    let packet = response.packet.ok_or_else(|| io::Error::other("packet should be present"))?;
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-100"));
    assert_eq!(response.status, boundline::GovernanceLifecycleState::GovernedReady);
    assert_eq!(packet.packet_ref, ".canon/runs/canon-run-100");
    assert_eq!(packet.readiness, boundline::PacketReadiness::Reusable);
    Ok(())
}

#[test]
fn canon_runtime_contract_sends_refresh_requests_with_lineage_fields() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_workspace("boundline-canon-refresh-contract")?;
    let capture_path = workspace.join("canon-request.json");
    let script_path = write_canon_capture_stub(
        &workspace,
        "canon-refresh-stub.sh",
        "canon-request.json",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-200\",\"packet_ref\":\".canon/runs/canon-run-200\",\"expected_document_refs\":[\".canon/runs/canon-run-200/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-200/discovery.md\"],\"approval_state\":\"granted\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"refresh packet ready\",\"message\":\"Canon refreshed the governed packet\"}",
    )?;
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-200/discovery.md",
        "# Discovery\n\nRefreshed governed packet.\n",
    )?;

    let runtime = CanonCliRuntime::new(script_path.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = runtime.execute(&GovernanceRuntimeRequest {
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
    })?;

    let request_json = fs::read_to_string(capture_path)?;
    assert!(request_json.contains("\"request_kind\":\"refresh\""), "{request_json}");
    assert!(request_json.contains("\"run_ref\":\"canon-run-200\""), "{request_json}");
    assert!(
        request_json.contains("\"packet_ref\":\".canon/runs/canon-run-200\""),
        "{request_json}"
    );
    assert_eq!(response.approval_state, boundline::ApprovalState::Granted);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-200"));
    Ok(())
}

#[test]
fn canon_runtime_contract_marks_scaffold_packets_as_rejected() -> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-canon-rejected-contract")?;
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-300/discovery.md",
        "# Discovery\n\nTODO\n",
    )?;
    let script = write_canon_stub(
        "boundline-canon-rejected-command",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-300\",\"packet_ref\":\".canon/runs/canon-run-300\",\"expected_document_refs\":[\".canon/runs/canon-run-300/discovery.md\"],\"document_refs\":[\".canon/runs/canon-run-300/discovery.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"discovery packet ready\",\"message\":\"Canon completed the governed discovery run\"}",
    )?;
    let runtime = CanonCliRuntime::new(script.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let request = GovernanceRuntimeRequest {
        workspace_ref: workspace.to_string_lossy().into_owned(),
        ..request()
    };

    let response = runtime.execute(&request)?;
    let packet = response.packet.ok_or_else(|| io::Error::other("packet should be present"))?;
    assert_eq!(packet.readiness, boundline::PacketReadiness::Rejected);
    assert_eq!(packet.missing_sections, vec!["substantive_body".to_string()]);
    Ok(())
}

#[test]
fn canon_runtime_contract_serializes_security_assessment_mode_in_start_requests()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-canon-security-start-contract")?;
    let capture_path = workspace.join("canon-security-request.json");
    let script_path = write_canon_capture_stub(
        &workspace,
        "canon-security-stub.sh",
        "canon-security-request.json",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-security\",\"packet_ref\":\".canon/runs/canon-run-security\",\"expected_document_refs\":[\".canon/runs/canon-run-security/security-assessment.md\"],\"document_refs\":[\".canon/runs/canon-run-security/security-assessment.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"security assessment packet ready\",\"message\":\"Canon completed the governed security assessment\"}",
    )?;
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-security/security-assessment.md",
        "# Security Assessment\n\nValidated the bounded security review.\n",
    )?;

    let runtime = CanonCliRuntime::new(script_path.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = runtime.execute(&GovernanceRuntimeRequest {
        stage_key: "bug-fix:verify".to_string(),
        mode: Some(boundline::CanonMode::SecurityAssessment),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        ..request()
    })?;

    let request_json = fs::read_to_string(capture_path)?;
    assert!(request_json.contains("\"stage_key\":\"bug-fix:verify\""), "{request_json}");
    assert!(request_json.contains("\"mode\":\"security-assessment\""), "{request_json}");
    let packet = response.packet.ok_or_else(|| io::Error::other("packet should be present"))?;
    assert_eq!(packet.packet_ref, ".canon/runs/canon-run-security");
    assert_eq!(packet.readiness, boundline::PacketReadiness::Reusable);
    Ok(())
}

#[test]
fn canon_runtime_contract_preserves_security_assessment_refresh_lineage()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-canon-security-refresh-contract")?;
    let capture_path = workspace.join("canon-security-refresh-request.json");
    let script_path = write_canon_capture_stub(
        &workspace,
        "canon-security-refresh-stub.sh",
        "canon-security-refresh-request.json",
        "{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-security-refresh\",\"packet_ref\":\".canon/runs/canon-run-security-refresh\",\"expected_document_refs\":[\".canon/runs/canon-run-security-refresh/security-assessment.md\"],\"document_refs\":[\".canon/runs/canon-run-security-refresh/security-assessment.md\"],\"approval_state\":\"granted\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"security refresh packet ready\",\"message\":\"Canon refreshed the governed security packet\"}",
    )?;
    write_workspace_file(
        &workspace,
        ".canon/runs/canon-run-security-refresh/security-assessment.md",
        "# Security Assessment\n\nRefreshed bounded security review.\n",
    )?;

    let runtime = CanonCliRuntime::new(script_path.to_string_lossy().into_owned())
        .with_working_directory(&workspace);
    let response = runtime.execute(&GovernanceRuntimeRequest {
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
    })?;

    let request_json = fs::read_to_string(capture_path)?;
    assert!(request_json.contains("\"request_kind\":\"refresh\""), "{request_json}");
    assert!(request_json.contains("\"mode\":\"security-assessment\""), "{request_json}");
    assert!(request_json.contains("\"run_ref\":\"canon-run-security-refresh\""), "{request_json}");
    assert_eq!(response.approval_state, boundline::ApprovalState::Granted);
    assert_eq!(response.run_ref.as_deref(), Some("canon-run-security-refresh"));
    Ok(())
}

#[test]
fn assistant_delight_contract_alignment_matches_canon_provider_contract_when_available()
-> Result<(), Box<dyn Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let canon_contract_path = repo_root
        .parent()
        .map(|path| {
            path.join("canon/specs/057-s7-delight-provider/contracts/delight-provider-contract.md")
        })
        .unwrap_or_default();
    if !canon_contract_path.is_file() {
        return Ok(());
    }

    let canon_contract = fs::read_to_string(&canon_contract_path)?;
    let boundline_contract_path =
        repo_root.join("specs/060-assistant-delight-layer/contracts/assistant-delight-contract.md");
    let boundline_contract = fs::read_to_string(&boundline_contract_path)?;

    let artifact_class_pairs = [
        ("### `packets`", "Packets"),
        ("### `approval-states`", "Approval States"),
        ("### `readiness-signals`", "Readiness Signals"),
        ("### `security-findings`", "Security Findings"),
        ("### `audit-findings`", "Audit/Review Findings"),
        ("### `promotion-references`", "Promotion References"),
    ];
    for (canon_anchor, boundline_anchor) in artifact_class_pairs {
        assert!(
            canon_contract.contains(canon_anchor),
            "{canon_anchor} missing from Canon contract"
        );
        assert!(
            boundline_contract.contains(boundline_anchor),
            "{boundline_anchor} missing from Boundline delight contract"
        );
    }

    let degradation_pairs = [
        ("`stale`", "Stale Inputs"),
        ("`incompatible`", "Incompatible Inputs"),
        ("`absent`", "Missing Inputs"),
        ("`contradicted`", "Contradictory Inputs"),
    ];
    for (canon_signal, boundline_signal) in degradation_pairs {
        assert!(
            canon_contract.contains(canon_signal),
            "{canon_signal} missing from Canon contract"
        );
        assert!(
            boundline_contract.contains(boundline_signal),
            "{boundline_signal} missing from Boundline delight contract"
        );
    }

    assert!(canon_contract.contains("delight-provider-v1"), "Canon contract line missing");
    assert!(
        boundline_contract.contains("057-s7-delight-provider")
            || boundline_contract.contains("Canon 057"),
        "Boundline contract must reference Canon 057"
    );
    Ok(())
}
