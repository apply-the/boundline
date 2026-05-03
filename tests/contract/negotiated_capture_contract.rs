use std::fs;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::domain::negotiation::NegotiationResolutionState;
use boundline::domain::session::ActiveSessionRecord;

#[test]
fn capture_persists_clarification_backed_negotiation_state_before_planning() {
    let workspace = temp_fixture_workspace("boundline-negotiated-capture-contract");

    let start = run_boundline_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_boundline_in(
        &workspace,
        &["capture", "--goal", "Improve the platform docs and fix whatever tests are broken"],
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));
    let capture_text = terminal_text(&capture);
    assert!(
        capture_text.contains(
            "negotiation_goal_summary: Improve the platform docs and fix whatever tests are broken"
        ),
        "{capture_text}"
    );
    assert!(
        capture_text.contains("negotiation_resolution: pending_clarification"),
        "{capture_text}"
    );
    assert!(
        capture_text.contains(
            "negotiation_acceptance_boundary: deliver the bounded outcome: Improve the platform docs and fix whatever tests are broken"
        ),
        "{capture_text}"
    );

    let session_path = workspace.join(".boundline").join("session.json");
    let record: ActiveSessionRecord =
        serde_json::from_slice(&fs::read(&session_path).unwrap()).unwrap();
    let packet =
        record.negotiation_packet.as_ref().expect("capture should persist a negotiation packet");

    assert_eq!(packet.resolution_state, NegotiationResolutionState::PendingClarification);
    assert!(packet.clarification_headline.is_some());
    assert!(packet.constraints.iter().any(|constraint| constraint.blocks_planning));
}
