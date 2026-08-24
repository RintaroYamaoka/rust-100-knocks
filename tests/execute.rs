//! /api/execute プロキシの入口契約: 受信ボディのパース → 許可リスト検証。
//! ハンドラ本体は HTTP glue のみで、判定ロジックはすべて shared 側にある。

use shared::playground::{validate, ExecuteRequest, ExecuteResponse};

#[test]
fn incoming_body_parses_as_execute_request() {
    let body = r#"{
        "channel": "stable", "mode": "debug", "edition": "2024",
        "crateType": "lib", "tests": true, "code": "pub fn f() {}"
    }"#;
    let req: ExecuteRequest = serde_json::from_str(body).unwrap();
    assert!(validate(&req).is_ok());
}

#[test]
fn tampered_body_is_rejected_before_forwarding() {
    // フロント以外からの直叩きで channel を偽装しても上流には送らない
    let body = r#"{
        "channel": "../../etc", "mode": "debug", "edition": "2024",
        "crateType": "lib", "tests": true, "code": ""
    }"#;
    let req: ExecuteRequest = serde_json::from_str(body).unwrap();
    assert!(validate(&req).is_err());
}

#[test]
fn upstream_response_parses_ignoring_unknown_fields() {
    // Playground は success/stdout/stderr 以外のフィールドも返す
    let body = r#"{"success": true, "exitDetail": "Exited with status 0", "stdout": "ok", "stderr": ""}"#;
    let resp: ExecuteResponse = serde_json::from_str(body).unwrap();
    assert!(resp.success);
    assert_eq!(resp.stdout, "ok");
}
