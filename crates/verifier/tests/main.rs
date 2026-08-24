//! CLI 入口のスモーク: データファイルのパース + 静的検証の配線が生きていることを確認する。

use shared::problem::Level;
use verifier::{load_problems_str, validate_static};

#[test]
fn load_and_validate_sample_json() {
    let json = r##"[{
        "id": "b001",
        "level": "beginner",
        "title": "t",
        "description_md": "d",
        "starter_code": "pub fn answer() -> i32 { 0 }",
        "hidden_tests": "#[test]\nfn t() { assert_eq!(answer(), 42); }",
        "answer_code": "pub fn answer() -> i32 { 42 }",
        "explanation_md": "e"
    }]"##;
    let problems = load_problems_str(json).unwrap();
    assert_eq!(problems.len(), 1);
    assert!(validate_static(&problems, Level::Beginner).is_empty());
}

#[test]
fn load_rejects_broken_json() {
    assert!(load_problems_str("{ not json").is_err());
}
