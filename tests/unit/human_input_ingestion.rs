use std::fs;
use std::path::PathBuf;

use boundline::domain::brief::{
    AuthoredBriefBundle, AuthoredBriefResolutionState, BriefIngestionError, InputSourceKind,
    MAX_BRIEF_SOURCES, normalize_inputs,
};
use boundline::domain::task::ClarificationReasonKind;
use uuid::Uuid;

fn temp_workspace(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn rejects_when_neither_goal_nor_brief_provided() {
    let workspace = temp_workspace("boundline-h-input-empty");
    let error = normalize_inputs(&workspace, None, &[]).unwrap_err();
    assert!(matches!(error, BriefIngestionError::NoInputProvided));
}

#[test]
fn ingests_direct_text_only_into_bundle_with_no_markdown_sources() {
    let workspace = temp_workspace("boundline-h-input-direct");
    let bundle: AuthoredBriefBundle =
        normalize_inputs(&workspace, Some("Fix the failing add test"), &[]).unwrap();
    assert_eq!(bundle.markdown_source_count(), 0);
    assert_eq!(bundle.primary_goal_text.as_deref(), Some("Fix the failing add test"));
    assert_eq!(bundle.render_goal_text(), "Fix the failing add test");
    assert_eq!(bundle.sources.len(), 1);
    assert_eq!(bundle.sources[0].kind, InputSourceKind::DirectText);
}

#[test]
fn ingests_markdown_brief_and_renders_provenance_header() {
    let workspace = temp_workspace("boundline-h-input-md");
    let brief = workspace.join("brief.md");
    fs::write(&brief, "# Goal\n\nReplace subtraction with addition\n").unwrap();

    let bundle = normalize_inputs(&workspace, None, std::slice::from_ref(&brief)).unwrap();
    assert_eq!(bundle.markdown_source_count(), 1);
    let goal_text = bundle.render_goal_text();
    assert!(goal_text.contains("## brief.md"), "{goal_text}");
    assert!(goal_text.contains("Replace subtraction with addition"), "{goal_text}");

    let source = &bundle.sources[0];
    assert_eq!(source.kind, InputSourceKind::AttachedMarkdown);
    assert_eq!(source.workspace_path.as_deref(), Some("brief.md"));
}

#[test]
fn rejects_brief_outside_workspace_with_dedicated_error() {
    let workspace = temp_workspace("boundline-h-input-out-ws");
    let foreign = temp_workspace("boundline-h-input-out-foreign");
    let brief = foreign.join("brief.md");
    fs::write(&brief, "outside\n").unwrap();
    let error = normalize_inputs(&workspace, None, &[brief]).unwrap_err();
    assert!(matches!(error, BriefIngestionError::OutsideWorkspace { .. }), "{error}");
}

#[test]
fn rejects_brief_with_unsupported_extension() {
    let workspace = temp_workspace("boundline-h-input-ext");
    let brief = workspace.join("notes.txt");
    fs::write(&brief, "nope\n").unwrap();
    let error = normalize_inputs(&workspace, None, &[brief]).unwrap_err();
    assert!(matches!(error, BriefIngestionError::UnsupportedExtension { .. }));
}

#[test]
fn rejects_more_than_max_brief_sources() {
    let workspace = temp_workspace("boundline-h-input-too-many");
    let mut paths = Vec::new();
    for i in 0..(MAX_BRIEF_SOURCES + 1) {
        let path = workspace.join(format!("brief-{i}.md"));
        fs::write(&path, format!("brief {i}\n")).unwrap();
        paths.push(path);
    }
    let error = normalize_inputs(&workspace, None, &paths).unwrap_err();
    assert!(
        matches!(error, BriefIngestionError::TooManySources(n) if n == MAX_BRIEF_SOURCES + 1),
        "{error}"
    );
}

#[test]
fn combines_direct_text_and_markdown_brief_in_render_order() {
    let workspace = temp_workspace("boundline-h-input-combo");
    let brief = workspace.join("plan.md");
    fs::write(&brief, "Step 1: investigate\nStep 2: fix\n").unwrap();
    let bundle = normalize_inputs(&workspace, Some("Goal: deliver fix"), &[brief]).unwrap();
    let rendered = bundle.render_goal_text();
    assert!(rendered.starts_with("Goal: deliver fix"));
    assert!(rendered.contains("## plan.md"));
}

#[test]
fn extracts_markdown_paths_mentioned_in_goal_text() {
    let workspace = temp_workspace("boundline-h-input-referenced");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    let architecture = workspace.join("docs/architecture.md");
    let regression = workspace.join("docs/regression.markdown");
    fs::write(&architecture, "Architecture context\n").unwrap();
    fs::write(&regression, "Regression notes\n").unwrap();

    let bundle = normalize_inputs(
        &workspace,
        Some(
            "Implement caching using docs/architecture.md and docs/regression.markdown before release.",
        ),
        &[],
    )
    .unwrap();

    let accepted_paths = bundle
        .sources
        .iter()
        .filter_map(|source| source.workspace_path.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(accepted_paths, vec!["docs/architecture.md", "docs/regression.markdown"]);
    assert_eq!(bundle.markdown_source_count(), 2);
}

#[test]
fn deduplicates_repeated_explicit_brief_paths_by_canonical_workspace_path() {
    let workspace = temp_workspace("boundline-h-input-dedup");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    let brief = workspace.join("docs/brief.md");
    fs::write(&brief, "Shared context\n").unwrap();

    let bundle = normalize_inputs(
        &workspace,
        Some("Use the shared brief"),
        &[brief.clone(), workspace.join("./docs/brief.md")],
    )
    .unwrap();

    let accepted_paths = bundle
        .sources
        .iter()
        .filter_map(|source| source.workspace_path.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(accepted_paths, vec!["docs/brief.md"]);
    assert_eq!(bundle.markdown_source_count(), 1);
    assert_eq!(bundle.sources.len(), 2);
    assert_eq!(bundle.deduplicated_sources, vec!["docs/brief.md"]);
}

#[test]
fn flags_unbounded_requests_for_clarification_before_planning() {
    let workspace = temp_workspace("boundline-h-input-clarification");

    let bundle = normalize_inputs(
        &workspace,
        Some("Improve the platform docs and fix whatever tests are broken"),
        &[],
    )
    .unwrap();

    assert_eq!(bundle.resolution_state, AuthoredBriefResolutionState::ClarificationRequired);
    let clarification = bundle.clarification.as_ref().expect("clarification should exist");
    assert_eq!(clarification.reason_kind, ClarificationReasonKind::UnboundedRequest);
    assert_eq!(clarification.missing_fields, vec!["bounded_scope"]);
    assert_eq!(
        clarification.headline(),
        "clarification required: narrow the request to one bounded outcome"
    );

    let draft = bundle.derived_task_draft.as_ref().expect("derived task draft should exist");
    assert!(!draft.planning_ready);
    assert_eq!(
        draft.blocking_clarification_ref.as_deref(),
        Some(clarification.clarification_id.as_str())
    );
}

#[test]
fn delivery_goal_without_validation_or_success_criteria_requires_quality_clarification() {
    let workspace = temp_workspace("boundline-h-input-goal-quality");

    let bundle = normalize_inputs(
        &workspace,
        Some(
            "Implement the first slice of a user management API with users, create, list, and \
             update operations, PostgreSQL persistence, and no new authentication boundary.",
        ),
        &[],
    )
    .unwrap();

    assert_eq!(bundle.resolution_state, AuthoredBriefResolutionState::ClarificationRequired);
    assert_eq!(bundle.goal_quality_state().as_deref(), Some("clarification_required"));
    let findings = bundle.goal_quality_findings().expect("quality findings");
    assert!(findings.contains(&"success_criteria".to_string()), "{findings:?}");
    assert!(findings.contains(&"validation_target".to_string()), "{findings:?}");

    let clarification = bundle.clarification.as_ref().expect("quality clarification");
    assert_eq!(clarification.reason_kind, ClarificationReasonKind::MissingContext);
    assert!(clarification.questions.len() <= 3, "{:?}", clarification.questions);
    assert!(
        clarification
            .questions
            .iter()
            .any(|question| question.contains("measurable success criteria")),
        "{:?}",
        clarification.questions
    );
    assert!(
        clarification
            .questions
            .iter()
            .any(|question| question.contains("validation command or acceptance evidence")),
        "{:?}",
        clarification.questions
    );
}

#[test]
fn quality_clarification_limits_questions_to_three_by_priority() {
    let workspace = temp_workspace("boundline-h-input-goal-quality-priority");

    let bundle =
        normalize_inputs(&workspace, Some("Build a user management API with OAuth2 roles"), &[])
            .unwrap();

    let clarification = bundle.clarification.as_ref().expect("quality clarification");
    assert_eq!(clarification.reason_kind, ClarificationReasonKind::MissingContext);
    assert!(clarification.questions.len() <= 3, "{:?}", clarification.questions);
    assert_eq!(
        clarification.questions.first().map(String::as_str),
        Some("Which bounded outcome and scope boundary should this first slice cover?")
    );
    assert!(
        clarification.missing_fields.iter().position(|field| field == "scope_boundary")
            < clarification.missing_fields.iter().position(|field| field == "auth_boundary"),
        "{:?}",
        clarification.missing_fields
    );
}

#[test]
fn low_impact_omissions_are_recorded_as_goal_quality_assumptions() {
    let workspace = temp_workspace("boundline-h-input-goal-quality-assumptions");

    let bundle = normalize_inputs(
        &workspace,
        Some(
            "Fix payment timeout bug in checkout retry handling. Validation target: checkout \
             timeout regression test passes. Success criteria: customers complete retryable \
             checkout flows without timeout failures.",
        ),
        &[],
    )
    .unwrap();

    assert_eq!(bundle.resolution_state, AuthoredBriefResolutionState::Ready);
    assert_eq!(bundle.goal_quality_state().as_deref(), Some("ready"));
    assert!(bundle.clarification.is_none(), "{:?}", bundle.clarification);
    let assumptions = bundle.goal_quality_assumptions().expect("quality assumptions");
    assert!(
        assumptions.iter().any(|assumption| assumption.contains("No new authentication")),
        "{assumptions:?}"
    );
}
