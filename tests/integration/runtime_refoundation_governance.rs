use crate::runtime_refoundation::temp_runtime_refoundation_governed_workspace;
use crate::workspace_fixture::{run_boundline_in, terminal_text};

#[test]
fn native_inspect_falls_back_to_runtime_evidence_when_canon_input_is_missing() {
    let workspace =
        temp_runtime_refoundation_governed_workspace("runtime-refoundation-governance-evidence");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--no-flow"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);

    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("routing: native (goal_plan)"), "{inspect_text}");
    assert!(
        inspect_text.contains(
            "evidence_summary: runtime(2): context, trace_evidence; canon(0): none; missing(1): canon_input"
        ),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(
            "fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only"
        ),
        "{inspect_text}"
    );
}
