use std::fs;

#[test]
fn delivery_model_docs_cover_pilot_loop_stop_rules_and_project_scale_example() {
    let docs = fs::read_to_string("docs/delivery-model.md").unwrap();

    assert!(docs.contains("# Delivery Pilot Model"), "{docs}");
    assert!(
        docs.contains("Large work is supported by decomposition, not by unbounded autonomy."),
        "{docs}"
    );
    for term in ["observe", "decide", "act", "verify", "update context"] {
        assert!(docs.contains(term), "missing {term}\n{docs}");
    }
    for stop_rule in [
        "context is insufficient",
        "governance is blocked",
        "validation is exhausted",
        "risk exceeds policy",
        "exceed the current boundary",
    ] {
        assert!(docs.contains(stop_rule), "missing {stop_rule}\n{docs}");
    }
    assert!(docs.contains("Build a customer onboarding capability with audit logging."), "{docs}");
    assert!(docs.contains("implementation slice 1"), "{docs}");
    assert!(docs.contains("implementation slice 2"), "{docs}");
}
