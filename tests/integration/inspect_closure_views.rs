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
fn inspect_context_and_council_views_render_human_facing_closure_lines()
-> Result<(), Box<dyn Error>> {
    let trace = load_delight_trace_fixture()?;
    let summary = summarize_trace("/tmp/assistant-delight-trace.json", &trace)?;

    let inspect_context =
        summary.inspect_context.as_ref().ok_or_else(|| fail("expected inspect context closure"))?;
    inspect_context.validate().map_err(fail)?;

    let inspect_council =
        summary.inspect_council.as_ref().ok_or_else(|| fail("expected inspect council closure"))?;
    inspect_council.validate().map_err(fail)?;

    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    for needle in [
        "inspect_context_headline: Active session state and trace evidence are available for bounded explanation work",
        "inspect_context_line: context_primary_inputs: .boundline/session.json, src/cli/output.rs",
        "inspect_context_next_action: boundline inspect --workspace <workspace>",
        "inspect_council_headline: council activity was recorded for this trace",
        "inspect_council_line: review_vote: bounded review accepted the projection",
        "inspect_council_source_attribution: review_timeline",
        "inspect_council_next_action: boundline inspect --workspace <workspace>",
    ] {
        ensure_contains(&rendered, needle, "inspect closure views")?;
    }

    Ok(())
}
