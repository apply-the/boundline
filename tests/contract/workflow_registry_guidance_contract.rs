use std::fs;

#[test]
fn workflow_registry_guidance_contract_describes_supported_authored_shape_and_boundaries() {
    let configuration = fs::read_to_string("docs/configuration.md").unwrap();

    assert!(configuration.contains("[workflow.governed-delivery]"), "{configuration}");
    assert!(
        configuration.contains(
            "phases = [\"capture\", \"plan\", \"run\", \"review\", \"govern\", \"inspect\"]"
        ),
        "{configuration}"
    );
    assert!(
        configuration.contains(
            "summary = \"bounded delivery path with review and governance before completion\""
        ),
        "{configuration}"
    );
    assert!(
        configuration.contains(
            "recommended_when = \"the task needs explicit review and governance evidence\""
        ),
        "{configuration}"
    );
    assert!(configuration.contains("review = \"review_triggered\""), "{configuration}");
    assert!(configuration.contains("governance = \"governance_required\""), "{configuration}");
    assert!(configuration.contains("no branching"), "{configuration}");
    assert!(configuration.contains("loops"), "{configuration}");
    assert!(configuration.contains("fan-out"), "{configuration}");
    assert!(configuration.contains("fan-in"), "{configuration}");
    assert!(configuration.contains("hidden background progression"), "{configuration}");
    assert!(configuration.contains("Canon-owned workflow control"), "{configuration}");
}

#[test]
fn workflow_registry_guidance_contract_keeps_route_relationships_explicit() {
    let readme = fs::read_to_string("README.md").unwrap();
    let assistant = fs::read_to_string("assistant/README.md").unwrap();

    assert!(readme.contains("session-native: start a session"), "{readme}");
    assert!(readme.contains("available as an explicit compatibility path"), "{readme}");
    assert!(
        assistant
            .contains("bounded named-workflow CLI surface directly: `workflow list -> workflow run -> workflow"),
        "{assistant}"
    );
    assert!(
        assistant.contains("Do not expose dedicated `/boundline-workflow-*` prompt surfaces"),
        "{assistant}"
    );
    assert!(!assistant.contains("/boundline-workflow-list"), "{assistant}");
    assert!(assistant.contains("compatibility remains explicit and subordinate"), "{assistant}");
}
