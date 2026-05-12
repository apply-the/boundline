use serde_json::Value;

pub fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "expected output to contain `{needle}`\n\noutput:\n{haystack}"
        );
    }
}

pub fn assert_json_field_eq(value: &Value, pointer: &str, expected: &str) {
    let actual = value.pointer(pointer).and_then(Value::as_str);
    assert_eq!(actual, Some(expected), "json pointer `{pointer}` mismatch in {value}");
}

pub fn assert_same_next_command(cli: &Value, chat: &Value) {
    let cli_next = cli.pointer("/next_command").and_then(Value::as_str);
    let chat_next = chat.pointer("/next_command").and_then(Value::as_str);
    assert_eq!(cli_next, chat_next, "CLI/chat next_command mismatch");
}
