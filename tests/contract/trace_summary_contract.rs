use std::path::PathBuf;

use crate::workspace_fixture::{
    extract_trace_path, run_boundline, temp_adaptive_ordering_boundary_workspace,
    temp_broken_fixture_workspace, temp_fixture_workspace, terminal_text, write_markdown_brief,
};
use boundline::FileConfigStore;
use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::adapters::trace_store::{FileTraceStore, TraceStore};
use boundline::cli::inspect::summarize_trace;
use boundline::cli::output::{render_trace_summary, trace_execution_condition_text};
use boundline::cli::session::{execute_capture, execute_plan, execute_run, execute_start};
use boundline::domain::configuration::{ConfigFile, ModelRoute, RoutingConfig, RuntimeKind};

#[test]
fn trace_summary_preserves_step_order_and_terminal_reason() {
    let workspace = temp_fixture_workspace("boundline-trace-summary");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert_eq!(summary.trace_ref, trace_path.to_string_lossy());
    assert_eq!(summary.executed_steps[0].step_id, "analyze");
    assert_eq!(summary.executed_steps[1].step_id, "code-fix-add");
    assert_eq!(summary.executed_steps[2].step_id, "verify-fix-add");
    assert!(summary.executed_steps[1].headline.contains("src/lib.rs"));
    assert!(summary.executed_steps[2].headline.contains("validation passed"));
    assert_eq!(summary.terminal_status, trace.terminal_status.unwrap());
    assert_eq!(summary.terminal_reason, trace.terminal_reason.unwrap());
}

#[test]
fn trace_summary_handles_fixture_terminal_failures_without_fake_recovery_events() {
    let workspace = temp_broken_fixture_workspace("boundline-trace-summary-broken");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Attempt the fixture patch on a broken workspace",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert!(summary.recovery_events.is_empty());
    assert!(summary.duration.is_some());
    assert_eq!(summary.terminal_status, trace.terminal_status.unwrap());
}

#[test]
fn trace_summary_carries_authored_input_summary_and_source_order() {
    let workspace = temp_fixture_workspace("boundline-trace-summary-human-input");
    write_markdown_brief(&workspace, "docs/explicit.md", "Explicit context\n");
    write_markdown_brief(&workspace, "docs/referenced.md", "Referenced context\n");

    let output = run_boundline(&[
        "run",
        "--goal",
        "Use docs/referenced.md with the explicit brief",
        "--brief",
        "docs/explicit.md",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert_eq!(
        summary.authored_input_summary.as_deref(),
        Some("direct_text + 2 markdown source(s)")
    );
    assert_eq!(
        summary.authored_input_sources,
        vec![
            "direct_text: developer goal".to_string(),
            "attached_markdown: docs/explicit.md".to_string(),
            "referenced_markdown: docs/referenced.md".to_string(),
        ]
    );
}

#[test]
fn trace_summary_uses_shared_route_and_execution_condition_vocabulary_for_compatibility_traces() {
    let workspace = temp_fixture_workspace("boundline-trace-summary-shared-vocabulary");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert_eq!(
        summary.routing_summary.as_deref(),
        Some(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
        )
    );
    assert_eq!(
        trace_execution_condition_text(&summary),
        format!("terminal - {}", summary.terminal_reason.message)
    );
}

#[test]
fn trace_summary_surfaces_broader_adaptive_family_evidence() {
    let workspace = temp_adaptive_ordering_boundary_workspace("boundline-trace-summary-ordering");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the inclusive threshold boundary",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert!(
        summary
            .adaptive_evidence
            .iter()
            .any(|line| line == "candidate_family: ordering_boundary_flip"),
        "{:?}",
        summary.adaptive_evidence
    );
    assert!(
        summary.adaptive_evidence.iter().any(|line| {
            line.contains("selection_reason: selected src/lib.rs via ordering_boundary_flip")
        }),
        "{:?}",
        summary.adaptive_evidence
    );
}

#[test]
fn trace_summary_projects_route_owner_and_effective_routing_snapshot() {
    let workspace = temp_fixture_workspace("boundline-trace-summary-route-config");
    let config = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-1".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();
    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(rendered.contains("route_owner: compatibility"), "{rendered}");
    assert!(
        rendered.contains(
            "route_config_projection: effective_routing: planning=codex/gpt-5-codex [built-in], implementation=codex/gpt-5-codex [built-in], verification=copilot/gpt-5.5 [built-in], review=claude/reviewer-1 [workspace], adjudication=codex/gpt-5-codex [built-in]"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "assistant_bindings: planning=codex, implementation=codex, verification=copilot, review=claude, adjudication=codex"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "follow_through_guidance: authoritative trace state currently points to `/boundline-next` as the next bounded action"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("follow_through_evidence_source: trace:lifecycle"), "{rendered}");
    assert!(rendered.contains("follow_through_next_action: /boundline-next"), "{rendered}");
}

#[test]
fn trace_summary_surfaces_source_attribution_fallback_and_next_best_action() {
    let workspace = temp_fixture_workspace("boundline-trace-summary-attribution");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Explain the current risk and next safe action",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();
    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(rendered.contains("source_attribution: runtime="), "{rendered}");
    assert!(
        rendered.contains(
            "fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("next_best_action:"), "{rendered}");
}

#[test]
fn trace_summary_surfaces_context_pack_for_native_runs() {
    let workspace = temp_fixture_workspace("boundline-trace-summary-context-pack");
    std::fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"trace_summary_context_pack\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::create_dir_all(workspace.join("tests")).unwrap();
    std::fs::write(
        workspace.join("src/context_router.rs"),
        "pub fn build_context_router() -> &'static str { \"ok\" }\n",
    )
    .unwrap();
    std::fs::write(
        workspace.join("src/lib.rs"),
        "pub mod context_router;\npub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    std::fs::write(
        workspace.join("tests/basic.rs"),
        "#[test]\nfn it_works() { assert_eq!(trace_summary_context_pack::add(2, 2), 4); }\n",
    )
    .unwrap();

    execute_start(Some(&workspace)).unwrap();
    execute_capture(Some(&workspace), Some("build a context router"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), None, false, false).unwrap();
    execute_plan(Some(&workspace), None, false, true).unwrap();
    execute_run(Some(&workspace)).unwrap();

    let record = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let trace_path = PathBuf::from(record.latest_trace_ref.unwrap());
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();
    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    assert_eq!(summary.context_credibility.as_deref(), Some("credible"));
    assert!(!summary.context_primary_inputs.is_empty());
    assert!(!summary.context_provenance.is_empty());
    assert!(rendered.contains("context_summary:"), "{rendered}");
    assert!(rendered.contains("context_credibility: credible"), "{rendered}");
    assert!(rendered.contains("context_primary_inputs:"), "{rendered}");
}
