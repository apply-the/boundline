use serde_json::json;
use synod::domain::flow::{
    FLOW_METADATA_KEY, attach_stage_metadata, built_in_flow, supported_flow_names,
};

#[test]
fn built_in_flow_registry_returns_supported_flows() {
    assert_eq!(supported_flow_names(), &["bug-fix", "change", "delivery"]);
    assert_eq!(built_in_flow("bug-fix").unwrap().stages.len(), 3);
    assert_eq!(built_in_flow("delivery").unwrap().stages.len(), 4);
    assert!(built_in_flow("missing").is_none());
}

#[test]
fn stage_metadata_is_attached_to_object_inputs() {
    let flow = built_in_flow("change").unwrap();
    let input = attach_stage_metadata(json!({"goal": "Ship change"}), flow, 1).unwrap();

    assert_eq!(input[FLOW_METADATA_KEY]["flow_name"], json!("change"));
    assert_eq!(input[FLOW_METADATA_KEY]["stage_id"], json!("implement"));
    assert_eq!(input[FLOW_METADATA_KEY]["stage_index"], json!(1));
    assert_eq!(input[FLOW_METADATA_KEY]["total_stages"], json!(3));
}
