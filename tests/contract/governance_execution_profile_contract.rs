use crate::workspace_fixture::{
    temp_optional_governance_workspace, temp_required_governance_workspace,
};
use boundline::fixture::load_workspace_execution_profile;
use boundline::{CanonMode, GovernanceRuntimeKind};

#[test]
fn execution_profile_contract_loads_optional_governance_stage_policy() {
    let workspace = temp_optional_governance_workspace("boundline-governance-profile-contract");

    let profile = load_workspace_execution_profile(&workspace).unwrap();
    let governance = profile.governance.expect("governance profile should be present");
    let policy = governance
        .stage_policy("bug-fix", "investigate")
        .expect("bug-fix investigate policy should exist");

    assert_eq!(governance.default_runtime, GovernanceRuntimeKind::Local);
    assert!(policy.enabled);
    assert!(!policy.required);
    assert_eq!(policy.effective_runtime(governance.default_runtime), GovernanceRuntimeKind::Canon);
    assert_eq!(policy.canon_mode, Some(CanonMode::Discovery));
}

#[test]
fn execution_profile_contract_preserves_required_governance_policy() {
    let workspace = temp_required_governance_workspace("boundline-governance-required-contract");

    let profile = load_workspace_execution_profile(&workspace).unwrap();
    let governance = profile.governance.expect("governance profile should be present");
    let policy = governance
        .stage_policy("bug-fix", "investigate")
        .expect("bug-fix investigate policy should exist");

    assert!(policy.required);
    assert_eq!(policy.risk.as_deref(), Some("medium"));
    assert_eq!(policy.zone.as_deref(), Some("engineering"));
    assert_eq!(policy.owner.as_deref(), Some("platform"));
}
