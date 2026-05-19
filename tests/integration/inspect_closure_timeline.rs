use std::error::Error;
use std::io;

use boundline::cli::inspect::summarize_trace;
use boundline::cli::output::render_trace_summary;
use boundline::domain::limits::TerminalCondition;
use boundline::domain::task::{TaskStatus, TerminalReason};
use boundline::domain::trace::{TraceEvent, TraceEventType};
use serde_json::json;

use crate::assistant_delight_support::load_delight_trace_fixture;

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(io::Error::other(message.into()))
}

fn ensure_contains(haystack: &str, needle: &str, context: &str) -> Result<(), Box<dyn Error>> {
    if haystack.contains(needle) {
        return Ok(());
    }

    Err(fail(format!("{context} missing expected text `{needle}` in output:\n{haystack}")))
}

#[test]
fn inspect_timeline_preserves_recovery_order_and_failed_terminal_state()
-> Result<(), Box<dyn Error>> {
    let mut trace = load_delight_trace_fixture()?;
    trace.terminal_status = Some(TaskStatus::Failed);
    trace.terminal_reason = Some(TerminalReason::new(
        TerminalCondition::UnrecoverableError,
        "timeline stayed blocked after verification failed",
        None,
    ));
    trace.events.push(TraceEvent {
        event_id: "event-stage-failed".to_string(),
        event_type: TraceEventType::StageFailed,
        step_id: Some("explain".to_string()),
        plan_revision: 1,
        payload: json!({
            "reason": "verification remained blocked after the bounded explanation failed"
        }),
        recorded_at: 1_716_115_525_000,
    });
    trace.events.push(TraceEvent {
        event_id: "event-stage-retry".to_string(),
        event_type: TraceEventType::StageRetryScheduled,
        step_id: Some("explain".to_string()),
        plan_revision: 1,
        payload: json!({
            "reason": "retry inspect after refreshing evidence"
        }),
        recorded_at: 1_716_115_526_000,
    });

    let summary = summarize_trace("/tmp/assistant-delight-timeline.json", &trace)?;
    let inspect_timeline = summary
        .inspect_timeline
        .as_ref()
        .ok_or_else(|| fail("expected inspect timeline closure"))?;
    inspect_timeline.validate().map_err(fail)?;

    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    for needle in [
        "inspect_timeline_headline: timeline preserves",
        "inspect_timeline_line: review_trigger: reasoning_profile",
        "inspect_timeline_line: stage_failure: verification remained blocked after the bounded explanation failed",
        "inspect_timeline_line: stage_retry: retry inspect after refreshing evidence",
        "inspect_timeline_terminal_status: failed",
        "inspect_timeline_terminal_reason: timeline stayed blocked after verification failed",
        "inspect_timeline_next_action: boundline inspect --workspace <workspace>",
    ] {
        ensure_contains(&rendered, needle, "inspect closure timeline")?;
    }

    Ok(())
}
