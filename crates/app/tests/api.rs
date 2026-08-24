use app::api::error_message_from_body;

#[test]
fn extracts_error_field_from_json_body() {
    assert_eq!(
        error_message_from_body(429, r#"{"error":"混雑しています"}"#),
        "混雑しています"
    );
}

#[test]
fn falls_back_to_status_code_when_body_is_not_json() {
    let msg = error_message_from_body(502, "<html>Bad Gateway</html>");
    assert!(msg.contains("502"));
}
