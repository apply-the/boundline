use crate::dashboard_fixture::{DashboardTestResult, require_contains};
use boundline::domain::dashboard::{
    ContextPackPanelItem, DashboardAuthority, DashboardBrandMark, DashboardColorProfile,
    DashboardDiagnosticItem, DashboardPanels, DashboardSessionView, DashboardSnapshot,
    DegradedDashboardState, DegradedReason, DegradedSeverity, ExecutionCondition, GoalPlanPanel,
    GovernedReferencePanelItem, RuntimeEventProjection,
};
use boundline_dashboard::render::{RenderMode, RenderOptions, render_snapshot};

fn degraded_snapshot(reason: DegradedReason) -> DashboardSnapshot {
    DashboardSnapshot {
        snapshot_id: "snapshot-degraded".to_string(),
        workspace_ref: "/workspace".to_string(),
        captured_at: "2026-05-20T00:00:00Z".to_string(),
        authority: DashboardAuthority::Degraded,
        session_revision: None,
        session: None,
        timeline: Vec::new(),
        panels: DashboardPanels::empty(),
        actions: Vec::new(),
        degraded_state: Some(DegradedDashboardState {
            reason,
            severity: DegradedSeverity::Info,
            available_commands: vec!["boundline start --workspace /workspace".to_string()],
            unavailable_panels: vec!["goal_plan".to_string()],
            recovery_hint: Some("Start or capture a session first.".to_string()),
        }),
        branding: DashboardBrandMark {
            wordmark_lines: vec!["boundline".to_string()],
            color_profile: DashboardColorProfile::Monochrome,
            min_width: 20,
            fallback_label: "boundline".to_string(),
        },
    }
}

fn active_snapshot() -> DashboardSnapshot {
    DashboardSnapshot {
        snapshot_id: "snapshot-active".to_string(),
        workspace_ref: "/workspace".to_string(),
        captured_at: "2026-05-20T00:00:00Z".to_string(),
        authority: DashboardAuthority::SessionNative,
        session_revision: Some(7),
        session: Some(DashboardSessionView {
            session_id: "session-7".to_string(),
            goal: "Fix the failing checkout flow".to_string(),
            route_kind: "native_goal_plan".to_string(),
            route_owner: "runtime".to_string(),
            active_flow: Some("bug-fix".to_string()),
            flow_state: Some("confirmed".to_string()),
            goal_plan_state: Some("confirmed".to_string()),
            goal_plan_revision: Some(2),
            current_stage: Some("verify".to_string()),
            current_step_id: Some("run-dashboard-tests".to_string()),
            current_step_index: Some(1),
            execution_condition: ExecutionCondition::Ready,
            latest_status: "running".to_string(),
            next_action_label: "Continue bounded execution".to_string(),
            next_command: "boundline run --workspace /workspace".to_string(),
            blocking_reason: None,
            compatibility_context: None,
        }),
        timeline: vec![
            RuntimeEventProjection {
                event_id: "event-1".to_string(),
                event_kind: "action".to_string(),
                occurred_at: "unix-ms:1".to_string(),
                stage: Some("verify".to_string()),
                step_id: Some("run-dashboard-tests".to_string()),
                status: "recorded".to_string(),
                headline: "Decision verified".to_string(),
                evidence_refs: vec!["trace:task-1".to_string()],
                trace_ref: Some("/workspace/.boundline/traces/task-1.json".to_string()),
                details: Vec::new(),
            },
            RuntimeEventProjection {
                event_id: "event-2".to_string(),
                event_kind: "checkpoint".to_string(),
                occurred_at: "unix-ms:2".to_string(),
                stage: Some("verify".to_string()),
                step_id: Some("save".to_string()),
                status: "recorded".to_string(),
                headline: "Checkpoint created".to_string(),
                evidence_refs: vec!["trace:task-1".to_string()],
                trace_ref: Some("/workspace/.boundline/traces/task-1.json".to_string()),
                details: Vec::new(),
            },
        ],
        panels: DashboardPanels {
            goal_plan: Some(GoalPlanPanel {
                revision: 2,
                state: "confirmed".to_string(),
                verification_strategy: Some("Run the focused dashboard suite".to_string()),
                targets: vec!["tests/contract/dashboard_render_contract.rs".to_string()],
            }),
            context_pack: vec![ContextPackPanelItem {
                reason: "recent status output".to_string(),
                source: "workspace".to_string(),
                budget: Some("bounded".to_string()),
                authority: "session_native".to_string(),
                evidence_ref: "status:latest".to_string(),
            }],
            evidence: Vec::new(),
            context_degradation: Vec::new(),
            stop_rules: Vec::new(),
            findings: Vec::new(),
            checkpoints: Vec::new(),
            governed_references: vec![GovernedReferencePanelItem {
                reference: "/workspace/.canon/packet-1.json".to_string(),
                readiness: "available".to_string(),
                provenance: "task_context".to_string(),
                approval_cue: Some("approved".to_string()),
                read_only: true,
            }],
            diagnostics: vec![DashboardDiagnosticItem {
                category: "workspace".to_string(),
                status: "readable".to_string(),
                details: "dashboard snapshot assembled from Boundline-owned state".to_string(),
            }],
        },
        actions: Vec::new(),
        degraded_state: None,
        branding: DashboardBrandMark {
            wordmark_lines: vec!["boundline".to_string()],
            color_profile: DashboardColorProfile::Color,
            min_width: 20,
            fallback_label: "boundline".to_string(),
        },
    }
}

#[test]
fn degraded_render_preserves_reason_and_fallback_command() -> DashboardTestResult {
    let output = render_snapshot(
        &degraded_snapshot(DegradedReason::MissingActiveSession),
        RenderOptions { mode: RenderMode::Degraded, width: 80, height: 20, color: false },
    );
    require_contains(&output, "boundline", "wordmark")?;
    require_contains(&output, "missing_active_session", "degraded reason")?;
    require_contains(&output, "boundline start", "fallback command")
}

#[test]
fn compact_render_keeps_summary_visible_on_narrow_terminals() -> DashboardTestResult {
    let output = render_snapshot(
        &degraded_snapshot(DegradedReason::MissingActiveSession),
        RenderOptions { mode: RenderMode::Compact, width: 42, height: 8, color: true },
    );
    require_contains(&output, "mode: compact", "compact marker")?;
    require_contains(&output, "workspace: /workspace", "workspace summary")
}

#[test]
fn interactive_render_includes_session_timeline_and_panels() -> DashboardTestResult {
    let output = render_snapshot(
        &active_snapshot(),
        RenderOptions { mode: RenderMode::Interactive, width: 100, height: 30, color: true },
    );

    require_contains(&output, "mode: interactive", "interactive mode")?;
    require_contains(&output, "goal: Fix the failing checkout flow", "goal")?;
    require_contains(&output, "stage: verify", "stage")?;
    require_contains(&output, "step: run-dashboard-tests", "step")?;
    require_contains(&output, "timeline:", "timeline heading")?;
    require_contains(&output, "Decision verified", "timeline event")?;
    require_contains(&output, "goal_plan: confirmed rev 2", "goal plan")?;
    require_contains(&output, "verification: Run the focused dashboard suite", "verification")?;
    require_contains(&output, "context_pack:", "context pack heading")?;
    require_contains(&output, "governed_references:", "governed references heading")?;
    require_contains(&output, "diagnostics:", "diagnostics heading")
}

#[test]
fn monochrome_render_marks_mode_for_active_snapshot() -> DashboardTestResult {
    let output = render_snapshot(
        &active_snapshot(),
        RenderOptions { mode: RenderMode::Monochrome, width: 80, height: 20, color: false },
    );

    require_contains(&output, "mode: monochrome", "monochrome mode")?;
    require_contains(&output, "next: boundline run --workspace /workspace", "next command")
}

#[test]
fn degraded_render_labels_each_supported_reason() -> DashboardTestResult {
    for (reason, label) in [
        (DegradedReason::InvalidWorkspace, "invalid_workspace"),
        (DegradedReason::MissingActiveSession, "missing_active_session"),
        (DegradedReason::InvalidSessionJson, "invalid_session_json"),
        (DegradedReason::StaleTraceReference, "stale_trace_reference"),
        (DegradedReason::TerminalUnsupported, "terminal_unsupported"),
        (DegradedReason::DashboardUnavailable, "dashboard_unavailable"),
        (DegradedReason::RuntimeCommandUnavailable, "runtime_command_unavailable"),
        (DegradedReason::StateReadFailed, "state_read_failed"),
    ] {
        let output = render_snapshot(
            &degraded_snapshot(reason),
            RenderOptions { mode: RenderMode::Degraded, width: 80, height: 20, color: false },
        );
        require_contains(&output, label, label)?;
    }
    Ok(())
}
