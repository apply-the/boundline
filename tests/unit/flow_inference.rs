use synod::orchestrator::flow_inference::infer_flow;

#[test]
fn bug_fix_keywords_infer_bug_fix_flow() {
    for keyword in &["fix", "bug", "broken", "failing", "regression", "crash", "error"] {
        let goal = format!("Need to {keyword} the login issue");
        let result = infer_flow(&goal);
        assert!(result.is_some(), "keyword '{keyword}' should infer a flow");
        let flow = result.unwrap();
        assert_eq!(flow.flow_name, "bug-fix", "keyword '{keyword}' should map to bug-fix");
        assert!(flow.confidence_reason.contains(keyword));
        assert!(!flow.confirmed);
    }
}

#[test]
fn change_keywords_infer_change_flow() {
    for keyword in &["add", "implement", "feature", "new", "create", "extend", "refactor"] {
        let goal = format!("I want to {keyword} a dashboard");
        let result = infer_flow(&goal);
        assert!(result.is_some(), "keyword '{keyword}' should infer a flow");
        let flow = result.unwrap();
        assert_eq!(flow.flow_name, "change", "keyword '{keyword}' should map to change");
    }
}

#[test]
fn delivery_keywords_infer_delivery_flow() {
    for keyword in &["deliver", "release", "ship", "deploy", "complete", "launch"] {
        let goal = format!("We need to {keyword} the product");
        let result = infer_flow(&goal);
        assert!(result.is_some(), "keyword '{keyword}' should infer a flow");
        let flow = result.unwrap();
        assert_eq!(flow.flow_name, "delivery", "keyword '{keyword}' should map to delivery");
    }
}

#[test]
fn no_keywords_returns_none() {
    assert!(infer_flow("update the README documentation").is_none());
    assert!(infer_flow("review the pull request").is_none());
    assert!(infer_flow("check formatting").is_none());
}

#[test]
fn bug_fix_takes_priority_over_change() {
    // "fix" (bug-fix) should win over "add" (change)
    let result = infer_flow("fix and add a new handler").unwrap();
    assert_eq!(result.flow_name, "bug-fix");
}

#[test]
fn delivery_takes_priority_over_change() {
    // "ship" (delivery) should win over "implement" (change)
    let result = infer_flow("ship the new implementation").unwrap();
    assert_eq!(result.flow_name, "delivery");
}

#[test]
fn case_insensitive_matching() {
    let result = infer_flow("FIX the BUG").unwrap();
    assert_eq!(result.flow_name, "bug-fix");
}
