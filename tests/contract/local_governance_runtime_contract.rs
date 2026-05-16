use boundline::{
    ApprovalState, GovernanceBoundedContext, GovernanceLifecycleState, GovernanceRequestKind,
    GovernanceRuntime, GovernanceRuntimeKind, GovernanceRuntimeRequest, LocalGovernanceRuntime,
    PacketReadiness,
};

fn request_with(read_targets: Vec<&str>) -> GovernanceRuntimeRequest {
    GovernanceRuntimeRequest {
        request_kind: GovernanceRequestKind::Start,
        governance_attempt_id: "contract-attempt".to_string(),
        stage_key: "bug-fix:investigate".to_string(),
        goal: "Investigate a failing change".to_string(),
        workspace_ref: "/tmp/boundline-governance-contract".to_string(),
        autopilot: false,
        mode: None,
        system_context: None,
        risk: None,
        zone: None,
        owner: None,
        run_ref: None,
        packet_ref: None,
        bounded_context: GovernanceBoundedContext {
            read_targets: read_targets.into_iter().map(str::to_string).collect(),
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        input_documents: Vec::new(),
    }
}

#[test]
fn local_governance_runtime_contract_blocks_without_bounded_context() {
    let runtime = LocalGovernanceRuntime;
    let response = runtime.execute(&request_with(Vec::new())).unwrap();

    assert_eq!(runtime.kind(), GovernanceRuntimeKind::Local);
    assert_eq!(response.status, GovernanceLifecycleState::Blocked);
    assert_eq!(response.approval_state, ApprovalState::NotNeeded);
    assert!(response.packet.is_none());
}

#[test]
fn local_governance_runtime_contract_returns_reusable_packet_shape() {
    let runtime = LocalGovernanceRuntime;
    let response = runtime.execute(&request_with(vec!["src/lib.rs"])).unwrap();
    let packet = response.packet.expect("packet should be present");

    assert_eq!(response.status, GovernanceLifecycleState::GovernedReady);
    assert_eq!(packet.runtime, GovernanceRuntimeKind::Local);
    assert_eq!(packet.readiness, PacketReadiness::Reusable);
    assert_eq!(packet.expected_document_refs.len(), 1);
    assert_eq!(packet.document_refs, packet.expected_document_refs);
    assert!(packet.packet_ref.contains("bug-fix-investigate"));
    assert!(packet.authority_governance.is_none());
    assert!(packet.adaptive_governance.is_none());
}
