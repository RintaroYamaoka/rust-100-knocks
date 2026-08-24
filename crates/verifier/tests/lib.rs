use shared::problem::{Level, Problem};
use verifier::{run_problem, validate_static};

fn problem(id: &str, level: Level) -> Problem {
    Problem {
        id: id.to_string(),
        level,
        title: "テスト用".into(),
        description_md: "説明".into(),
        starter_code: "pub fn answer() -> i32 {\n    0 // TODO\n}".into(),
        hidden_tests: "#[test]\nfn passes() { assert_eq!(answer(), 42); }".into(),
        answer_code: "pub fn answer() -> i32 { 42 }".into(),
        explanation_md: "解説".into(),
        hints: vec![],
        tags: vec!["test".into()],
    }
}

#[test]
fn validate_static_accepts_wellformed_problem() {
    let issues = validate_static(&[problem("b001", Level::Beginner)], Level::Beginner);
    assert!(issues.is_empty(), "{issues:?}");
}

#[test]
fn validate_static_rejects_duplicate_ids() {
    let ps = vec![problem("b001", Level::Beginner), problem("b001", Level::Beginner)];
    let issues = validate_static(&ps, Level::Beginner);
    assert!(issues.iter().any(|i| i.message.contains("重複")));
}

#[test]
fn validate_static_rejects_wrong_id_prefix_and_level() {
    let issues = validate_static(&[problem("i001", Level::Beginner)], Level::Beginner);
    assert!(issues.iter().any(|i| i.message.contains("id")));

    let issues = validate_static(&[problem("b001", Level::Advanced)], Level::Beginner);
    assert!(issues.iter().any(|i| i.message.contains("level")));
}

#[test]
fn validate_static_requires_test_attribute_in_hidden_tests() {
    let mut p = problem("b001", Level::Beginner);
    p.hidden_tests = "fn not_a_test() {}".into();
    let issues = validate_static(&[p], Level::Beginner);
    assert!(issues.iter().any(|i| i.message.contains("#[test]")));
}

#[test]
fn validate_static_rejects_empty_required_fields() {
    let mut p = problem("b001", Level::Beginner);
    p.explanation_md = String::new();
    let issues = validate_static(&[p], Level::Beginner);
    assert!(issues.iter().any(|i| i.message.contains("explanation_md")));
}

/// 実 cargo での実行検証: answer は通り、starter は落ちる (= 問題として成立している)。
#[test]
fn run_problem_answer_passes_and_starter_fails() {
    let scratch = std::env::temp_dir().join(format!("verifier-test-{}", std::process::id()));
    let p = problem("b001", Level::Beginner);

    let answer = run_problem(&scratch, &p.answer_code, &p.hidden_tests).unwrap();
    assert!(answer.passed, "answer が通らない: {}", answer.output);

    let starter = run_problem(&scratch, &p.starter_code, &p.hidden_tests).unwrap();
    assert!(!starter.passed, "starter がそのまま通ってしまう");

    std::fs::remove_dir_all(&scratch).ok();
}
