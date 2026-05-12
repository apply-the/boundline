use boundline::{
    VotingBoundaryInput, VotingBoundaryTrigger, VotingStageRisk, voting_boundary_decision,
};

#[test]
fn high_impact_architecture_requires_voting() {
    let decision = voting_boundary_decision(VotingBoundaryInput {
        stage: VotingBoundaryTrigger::Architecture,
        risk: VotingStageRisk::High,
        structural_impact: true,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready: false,
        material_security_finding: false,
        critical_supply_chain_finding: false,
        migration_cutover: false,
        incident_high_blast_radius: false,
        preserved_behavior_evidence: false,
        explicitly_requested: false,
    });

    assert!(decision.required);
    assert_eq!(decision.trigger, Some("high_impact_architecture".to_string()));
    assert!(decision.blocks_continuation_until_resolved);
}

#[test]
fn low_risk_refactor_with_preserved_behavior_skips_default_voting() {
    let decision = voting_boundary_decision(VotingBoundaryInput {
        stage: VotingBoundaryTrigger::Refactor,
        risk: VotingStageRisk::Low,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready: false,
        material_security_finding: false,
        critical_supply_chain_finding: false,
        migration_cutover: false,
        incident_high_blast_radius: false,
        preserved_behavior_evidence: true,
        explicitly_requested: false,
    });

    assert!(!decision.required);
    assert_eq!(decision.skip_reason.as_deref(), Some("low_risk_preserved_behavior"));
}

#[test]
fn validation_exhaustion_and_pr_ready_trigger_voting() {
    let validation = voting_boundary_decision(VotingBoundaryInput {
        stage: VotingBoundaryTrigger::Implementation,
        risk: VotingStageRisk::Medium,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: true,
        pr_ready: false,
        material_security_finding: false,
        critical_supply_chain_finding: false,
        migration_cutover: false,
        incident_high_blast_radius: false,
        preserved_behavior_evidence: false,
        explicitly_requested: false,
    });
    let pr = voting_boundary_decision(VotingBoundaryInput {
        validation_exhausted: false,
        pr_ready: true,
        ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::PrReview)
    });

    assert_eq!(validation.trigger.as_deref(), Some("validation_exhausted"));
    assert_eq!(pr.trigger.as_deref(), Some("pr_ready"));
}
