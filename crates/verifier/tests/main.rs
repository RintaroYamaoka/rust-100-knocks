//! CLI 入口のスモーク: データファイルのパース + 静的検証の配線が生きていることを確認する。

use shared::language::Language;
use shared::problem::Level;
use verifier::{load_problems_str, validate_static};

#[test]
fn load_and_validate_sample_json() {
    let json = r##"[{
        "id": "b001",
        "language": "rust",
        "level": "beginner",
        "title": "2倍にする関数",
        "description_md": "整数 n を受け取り 2 倍にして返す関数を完成させましょう。Rust では関数の最後の式がそのまま戻り値になるので、return もセミコロンも要りません。負数と 0 の場合も確かめてください。",
        "starter_code": "pub fn answer() -> i32 { todo!() }",
        "hidden_tests": "#[test]\nfn a() { assert_eq!(answer(), 42); }\n#[test]\nfn b() { assert!(answer() > 0); }",
        "answer_code": "pub fn answer() -> i32 { 42 }",
        "explanation_md": "e"
    }]"##;
    let problems = load_problems_str(json).unwrap();
    assert_eq!(problems.len(), 1);
    let issues = validate_static(&problems, Language::Rust, Level::Beginner);
    assert!(
        issues.is_empty(),
        "{}",
        issues.iter().map(|i| i.message.clone()).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn load_rejects_broken_json() {
    assert!(load_problems_str("{ not json").is_err());
}

#[test]
fn load_rejects_problem_without_language() {
    // language が無いデータを既定値で通すと、検査が 1 段抜ける
    let json = r##"[{
        "id": "b001",
        "level": "beginner",
        "title": "t",
        "description_md": "d",
        "starter_code": "a",
        "hidden_tests": "b",
        "answer_code": "c",
        "explanation_md": "e"
    }]"##;
    assert!(load_problems_str(json).is_err());
}
