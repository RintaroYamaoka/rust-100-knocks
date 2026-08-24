use shared::problem::{compose_submission, Level, Problem};

fn sample_json() -> &'static str {
    r##"{
        "id": "b001",
        "level": "beginner",
        "title": "変数束縛",
        "description_md": "変数 `x` を定義しよう",
        "starter_code": "fn answer() -> i32 {\n    // TODO\n}",
        "hidden_tests": "#[test]\nfn passes() { assert_eq!(answer(), 42); }",
        "answer_code": "fn answer() -> i32 { 42 }",
        "explanation_md": "let で束縛します"
    }"##
}

#[test]
fn problem_deserializes_with_default_hints_and_tags() {
    let p: Problem = serde_json::from_str(sample_json()).unwrap();
    assert_eq!(p.id, "b001");
    assert_eq!(p.level, Level::Beginner);
    assert!(p.hints.is_empty());
    assert!(p.tags.is_empty());
}

#[test]
fn problem_roundtrips_through_json() {
    let p: Problem = serde_json::from_str(sample_json()).unwrap();
    let s = serde_json::to_string(&p).unwrap();
    let back: Problem = serde_json::from_str(&s).unwrap();
    assert_eq!(p, back);
}

#[test]
fn level_serializes_lowercase() {
    assert_eq!(serde_json::to_string(&Level::Advanced).unwrap(), "\"advanced\"");
}

#[test]
fn level_metadata() {
    assert_eq!(Level::Beginner.file_name(), "beginner.json");
    assert_eq!(Level::Intermediate.label_ja(), "中級");
    assert_eq!(Level::ALL.len(), 3);
}

#[test]
fn compose_submission_contains_both_parts() {
    let joined = compose_submission("fn answer() {}", "#[test]\nfn t() {}");
    assert!(joined.starts_with("fn answer() {}"));
    assert!(joined.contains("#[test]"));
    // ユーザーコードとテストの間に区切りコメントが入る
    assert!(joined.contains("判定用テスト"));
}
