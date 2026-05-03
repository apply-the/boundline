use boundline::domain::flow::{FlowValidationError, SessionFlowState};

#[test]
fn session_flow_state_advances_across_known_stages() {
    let mut state = SessionFlowState {
        flow_name: "bug-fix".to_string(),
        current_stage_id: "investigate".to_string(),
        current_stage_index: 0,
        total_stages: 3,
    };

    assert!(state.advance().unwrap());
    assert_eq!(state.current_stage_id, "implement");
    assert_eq!(state.current_stage_index, 1);

    assert!(state.advance().unwrap());
    assert_eq!(state.current_stage_id, "verify");
    assert_eq!(state.current_stage_index, 2);

    assert!(!state.advance().unwrap());
}

#[test]
fn session_flow_state_rejects_mismatched_stage_identity() {
    let state = SessionFlowState {
        flow_name: "bug-fix".to_string(),
        current_stage_id: "verify".to_string(),
        current_stage_index: 0,
        total_stages: 3,
    };

    assert!(matches!(state.validate(), Err(FlowValidationError::StageIdMismatch { .. })));
}
