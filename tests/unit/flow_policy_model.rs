use synod::domain::decision::DecisionType;
use synod::domain::flow_policy::{FlowPolicy, FlowPolicyError, TransitionCondition};

#[test]
fn from_builtin_creates_bug_fix_policy() {
    let policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert_eq!(policy.flow_name, "bug-fix");
    assert_eq!(policy.stage_policies.len(), 3);
    assert_eq!(policy.current_stage_index, 0);
}

#[test]
fn from_builtin_rejects_unknown_flow() {
    let err = FlowPolicy::from_builtin("unknown").unwrap_err();
    assert!(matches!(err, FlowPolicyError::UnknownFlow(_)));
}

#[test]
fn bug_fix_investigate_only_allows_analyze() {
    let policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert!(policy.is_allowed(DecisionType::Analyze));
    assert!(!policy.is_allowed(DecisionType::Code));
    assert!(!policy.is_allowed(DecisionType::Test));
    assert!(!policy.is_allowed(DecisionType::Fix));
    assert!(!policy.is_allowed(DecisionType::Replan));
}

#[test]
fn bug_fix_implement_allows_code_and_fix() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    policy.advance_stage().unwrap();
    assert!(!policy.is_allowed(DecisionType::Analyze));
    assert!(policy.is_allowed(DecisionType::Code));
    assert!(policy.is_allowed(DecisionType::Fix));
    assert!(!policy.is_allowed(DecisionType::Test));
}

#[test]
fn bug_fix_verify_allows_test_and_replan() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    policy.advance_stage().unwrap();
    policy.advance_stage().unwrap();
    assert!(policy.is_allowed(DecisionType::Test));
    assert!(policy.is_allowed(DecisionType::Replan));
    assert!(!policy.is_allowed(DecisionType::Code));
}

#[test]
fn advance_stage_returns_false_at_final_stage() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert!(policy.advance_stage().unwrap()); // investigate → implement
    assert!(policy.advance_stage().unwrap()); // implement → verify
    assert!(!policy.advance_stage().unwrap()); // already at final
}

#[test]
fn is_final_stage_detects_last_stage() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert!(!policy.is_final_stage());
    policy.advance_stage().unwrap();
    assert!(!policy.is_final_stage());
    policy.advance_stage().unwrap();
    assert!(policy.is_final_stage());
}

#[test]
fn current_stage_returns_stage_at_index() {
    let policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    let stage = policy.current_stage().unwrap();
    assert_eq!(stage.stage_id, "investigate");
    assert_eq!(stage.transition_condition, TransitionCondition::AllVerified);
}

#[test]
fn all_builtin_bug_fix_stages_share_verifiable_transition_rules() {
    let policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert!(
        policy
            .stage_policies
            .iter()
            .all(|stage| stage.transition_condition == TransitionCondition::AllVerified)
    );
}

#[test]
fn change_flow_stage_policies_are_correct() {
    let policy = FlowPolicy::from_builtin("change").unwrap();
    assert_eq!(policy.stage_policies.len(), 3);
    assert_eq!(policy.stage_policies[0].stage_id, "understand-change");
    assert!(policy.stage_policies[0].allows(DecisionType::Analyze));
    assert!(policy.stage_policies[1].allows(DecisionType::Code));
    assert!(policy.stage_policies[2].allows(DecisionType::Test));
}

#[test]
fn delivery_flow_has_four_stages() {
    let policy = FlowPolicy::from_builtin("delivery").unwrap();
    assert_eq!(policy.stage_policies.len(), 4);
    assert_eq!(policy.stage_policies[0].stage_id, "requirements");
    assert_eq!(policy.stage_policies[1].stage_id, "architecture");
    assert_eq!(policy.stage_policies[2].stage_id, "backlog");
    assert_eq!(policy.stage_policies[3].stage_id, "implementation");
}

#[test]
fn delivery_implementation_allows_code_test_fix_replan() {
    let mut policy = FlowPolicy::from_builtin("delivery").unwrap();
    policy.advance_stage().unwrap(); // requirements → architecture
    policy.advance_stage().unwrap(); // architecture → backlog
    policy.advance_stage().unwrap(); // backlog → implementation
    assert!(policy.is_allowed(DecisionType::Code));
    assert!(policy.is_allowed(DecisionType::Test));
    assert!(policy.is_allowed(DecisionType::Fix));
    assert!(policy.is_allowed(DecisionType::Replan));
    assert!(!policy.is_allowed(DecisionType::Analyze));
}

#[test]
fn validate_rejects_empty_flow_name() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    policy.flow_name = String::new();
    assert!(matches!(policy.validate(), Err(FlowPolicyError::EmptyFlowName)));
}

#[test]
fn validate_rejects_empty_stages() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    policy.stage_policies.clear();
    assert!(matches!(policy.validate(), Err(FlowPolicyError::NoStages)));
}

#[test]
fn validate_rejects_out_of_range_stage_index() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    policy.current_stage_index = 99;
    assert!(matches!(policy.validate(), Err(FlowPolicyError::InvalidStageIndex { .. })));
}

#[test]
fn validate_rejects_stage_with_no_allowed_decisions() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    policy.stage_policies[0].allowed_decisions.clear();
    assert!(matches!(policy.validate(), Err(FlowPolicyError::NoAllowedDecisions { .. })));
}

#[test]
fn flow_policy_round_trips_through_json() {
    let policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    let json = serde_json::to_string(&policy).unwrap();
    let parsed: FlowPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(policy, parsed);
}
