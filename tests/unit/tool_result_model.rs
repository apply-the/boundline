use synod::domain::tool_result::{ToolResult, ToolResultError};

#[test]
fn new_creates_result_with_defaults() {
    let result = ToolResult::new("cargo", "cargo test", true, 200);
    assert_eq!(result.tool_id, "cargo");
    assert_eq!(result.invocation, "cargo test");
    assert!(result.success);
    assert_eq!(result.duration_ms, 200);
    assert!(result.exit_code.is_none());
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
    assert!(result.diff.is_none());
}

#[test]
fn builder_methods_chain() {
    let result = ToolResult::new("cargo", "cargo check", false, 50)
        .with_exit_code(1)
        .with_stdout("compiling...")
        .with_stderr("error: expected `;`")
        .with_diff("- old\n+ new");
    assert_eq!(result.exit_code, Some(1));
    assert_eq!(result.stdout, "compiling...");
    assert_eq!(result.stderr, "error: expected `;`");
    assert_eq!(result.diff.as_deref(), Some("- old\n+ new"));
}

#[test]
fn validate_rejects_empty_tool_id() {
    let result = ToolResult::new("", "cargo test", true, 100);
    assert!(matches!(result.validate(), Err(ToolResultError::MissingToolId)));
}

#[test]
fn validate_rejects_empty_invocation() {
    let result = ToolResult::new("cargo", "", true, 100);
    assert!(matches!(result.validate(), Err(ToolResultError::MissingInvocation)));
}

#[test]
fn validate_accepts_valid_result() {
    let result = ToolResult::new("cargo", "cargo test", true, 100);
    assert!(result.validate().is_ok());
}

#[test]
fn tool_result_round_trips_through_json() {
    let result =
        ToolResult::new("cargo", "cargo check", true, 123).with_exit_code(0).with_stdout("ok");
    let json = serde_json::to_string(&result).unwrap();
    let parsed: ToolResult = serde_json::from_str(&json).unwrap();
    assert_eq!(result, parsed);
}

#[test]
fn tool_result_deserializes_with_missing_optional_fields() {
    let json = r#"{"tool_id":"x","invocation":"y","duration_ms":0,"success":true}"#;
    let parsed: ToolResult = serde_json::from_str(json).unwrap();
    assert!(parsed.exit_code.is_none());
    assert!(parsed.stdout.is_empty());
    assert!(parsed.diff.is_none());
}
