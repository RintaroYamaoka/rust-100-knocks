//! /api/execute プロキシの入口契約: 受信ボディのパース → 検証 → 上流リクエストの組み立て。
//! ハンドラ本体は HTTP glue のみで、判定ロジックはすべて shared 側にある。

use shared::language::Language;
use shared::playground::{
    classify, normalize_wandbox, validate, wandbox_request, ExecuteRequest, Outcome,
    PlaygroundRequest, PlaygroundResponse, WandboxResponse,
};

#[test]
fn incoming_body_parses_as_execute_request() {
    let body = r#"{ "language": "rust", "code": "pub fn f() {}" }"#;
    let req: ExecuteRequest = serde_json::from_str(body).unwrap();
    assert_eq!(req.language, Language::Rust);
    assert!(validate(&req).is_ok());
}

#[test]
fn every_language_slug_is_accepted_by_the_wire_format() {
    for l in Language::ALL {
        let body = format!(r#"{{ "language": "{}", "code": "x" }}"#, l.slug());
        let req: ExecuteRequest = serde_json::from_str(&body).unwrap();
        assert_eq!(req.language, l);
    }
}

#[test]
fn unknown_language_is_rejected_at_parse_time() {
    let body = r#"{ "language": "brainfuck", "code": "x" }"#;
    assert!(serde_json::from_str::<ExecuteRequest>(body).is_err());
}

#[test]
fn empty_and_oversized_code_is_rejected_before_forwarding() {
    let empty: ExecuteRequest = serde_json::from_str(r#"{"language":"cpp","code":"  "}"#).unwrap();
    assert!(validate(&empty).is_err());

    let big = ExecuteRequest::judge(Language::Cpp, &"a".repeat(shared::playground::MAX_CODE_BYTES + 1));
    assert!(validate(&big).is_err());
}

#[test]
fn rust_is_routed_to_playground_not_wandbox() {
    assert!(wandbox_request(Language::Rust, "code").is_none());
    let pg = PlaygroundRequest::judge("pub fn f() {}");
    assert!(pg.tests);
    assert_eq!(pg.crate_type, "lib");
    assert_eq!(pg.edition, "2024");
}

#[test]
fn playground_request_serializes_crate_type_as_camel_case() {
    let s = serde_json::to_string(&PlaygroundRequest::judge("x")).unwrap();
    assert!(s.contains("\"crateType\":\"lib\""), "{s}");
}

#[test]
fn non_rust_languages_build_a_wandbox_request_with_the_pinned_compiler() {
    let r = wandbox_request(Language::Cpp, "int main(){}").unwrap();
    assert_eq!(r.compiler, "gcc-13.2.0");
    assert_eq!(r.options.as_deref(), Some("warning,c++17"));
    assert!(!r.save, "save:true は Wandbox にパーマリンクを作ってしまう");

    let r = wandbox_request(Language::Python, "print(1)").unwrap();
    assert_eq!(r.compiler, "cpython-3.13.8");
    assert_eq!(r.options, None);
}

#[test]
fn playground_response_parses_ignoring_unknown_fields() {
    let body = r#"{"success": true, "exitDetail": "Exited with status 0", "stdout": "test result: ok", "stderr": ""}"#;
    let resp: PlaygroundResponse = serde_json::from_str(body).unwrap();
    assert!(resp.success);
    assert_eq!(resp.stdout, "test result: ok");
}

#[test]
fn wandbox_response_parses_ignoring_unknown_fields() {
    let body = r#"{"status":"0","signal":"","compiler_output":"","compiler_error":"","program_output":"test result: ok\n","program_error":"","permlink":"","url":""}"#;
    let raw: WandboxResponse = serde_json::from_str(body).unwrap();
    let resp = normalize_wandbox(Language::Cpp, &raw);
    assert_eq!(classify(&resp), Outcome::Passed);
}

#[test]
fn upstream_transient_error_is_detected_not_shown_as_a_compile_error() {
    // Wandbox は過負荷時にこれを compiler_error に入れて返す。
    // コンパイルエラーとして見せると、正しいコードが赤く出て学習者が混乱する
    let body = r#"{"status":"126","compiler_error":"Error: OCI runtime error: crun: clone: Resource temporarily unavailable"}"#;
    let raw: WandboxResponse = serde_json::from_str(body).unwrap();
    assert!(raw.is_upstream_transient_error());
}

#[test]
fn a_normal_compile_error_is_not_mistaken_for_a_transient_error() {
    let body = r#"{"status":"1","compiler_error":"prog.cc:2:21: error: invalid conversion from 'const char*' to 'int'"}"#;
    let raw: WandboxResponse = serde_json::from_str(body).unwrap();
    assert!(!raw.is_upstream_transient_error());
    assert_eq!(classify(&normalize_wandbox(Language::Cpp, &raw)), Outcome::CompileError);
}
