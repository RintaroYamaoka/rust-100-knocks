//! 公開 API を通した統合スモーク: 問題 → 提出コード合成 → 実行結果分類までの一連の流れ。

use shared::playground::{classify, ExecuteRequest, ExecuteResponse, Outcome};
use shared::problem::compose_submission;

#[test]
fn submission_flow_composes_and_classifies() {
    let user_code = "fn answer() -> i32 { 41 }";
    let hidden_tests = "#[test]\nfn t() { assert_eq!(answer(), 42); }";
    let submission = compose_submission(user_code, hidden_tests);
    let req = ExecuteRequest::judge(&submission);
    assert!(req.code.contains("fn answer"));
    assert!(req.tests);

    // Playground が返す「テスト失敗」形の応答を分類できる
    let resp = ExecuteResponse {
        success: false,
        stdout: "test result: FAILED. 0 passed; 1 failed".into(),
        stderr: String::new(),
    };
    assert_eq!(classify(&resp), Outcome::TestsFailed);
}
