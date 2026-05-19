use std::error::Error;
use std::io;

use boundline::cli::inspect::summarize_trace;
use boundline::cli::output::render_trace_summary;

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
fn inspect_closure_contract_surfaces_context_council_and_timeline_lines()
-> Result<(), Box<dyn Error>> {
    let trace = load_delight_trace_fixture()?;
    let summary = summarize_trace("/tmp/assistant-delight-trace.json", &trace)?;
    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    for needle in [
        "inspect_context_headline: Active session state and trace evidence are available for bounded explanation work",
        "inspect_context_line: context_summary: Active session state and trace evidence are available for bounded explanation work",
        "inspect_context_source_attribution: session_state: .boundline/session.json",
        "inspect_council_headline: council activity was recorded for this trace",
        "inspect_council_line: review_trigger: reasoning_profile",
        "inspect_council_source_attribution: review_timeline",
        "inspect_timeline_headline: timeline preserves",
        "inspect_timeline_line: review_trigger: reasoning_profile",
        "inspect_timeline_terminal_status: succeeded",
        "inspect_timeline_terminal_reason: bounded explanation closed with an accepted next step",
    ] {
        ensure_contains(&rendered, needle, "inspect closure contract")?;
    }

    Ok(())
}
