use crate::runtime_refoundation::temp_runtime_refoundation_governed_workspace;
use crate::workspace_fixture::{run_boundline_in, terminal_text};

#[test]
fn canon_artifacts_surface_as_bounded_evidence_in_native_inspect_output() {
    let workspace =
        temp_runtime_refoundation_governed_workspace("runtime-refoundation-governance-evidence");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "fix the failing add test"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--no-flow"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);

    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("routing: native (goal_plan)"), "{inspect_text}");
    assert!(inspect_text.contains("evidence_inputs: canon:"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_selected:"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_completed:"), "{inspect_text}");
}
