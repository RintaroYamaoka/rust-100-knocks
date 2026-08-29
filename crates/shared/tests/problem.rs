use shared::language::Language;
use shared::problem::{compose_submission, problems_rel_path, problems_url, Level, Problem};

fn sample_json() -> &'static str {
    r##"{
        "id": "b001",
        "language": "rust",
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
    assert_eq!(p.language, Language::Rust);
    assert_eq!(p.level, Level::Beginner);
    assert!(p.hints.is_empty());
    assert!(p.tags.is_empty());
}

#[test]
fn problem_without_language_is_rejected() {
    // language は必須。既定値で補うと、cpp のファイルに rust の問題が紛れても
    // parse が通ってしまい、検査が 1 段抜ける
    let no_lang = sample_json().replace("\"language\": \"rust\",", "");
    assert!(serde_json::from_str::<Problem>(&no_lang).is_err());
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
    for l in Level::ALL {
        assert_eq!(Level::from_slug(l.slug()), Some(l));
    }
}

#[test]
fn problem_paths_are_namespaced_by_language() {
    assert_eq!(
        problems_rel_path(Language::Cpp, Level::Beginner),
        "data/problems/cpp/beginner.json"
    );
    assert_eq!(
        problems_url(Language::Rust, Level::Advanced),
        "/data/problems/rust/advanced.json"
    );
    // 21 通りすべてが一意
    let mut seen = std::collections::HashSet::new();
    for lang in Language::ALL {
        for level in Level::ALL {
            assert!(seen.insert(problems_rel_path(lang, level)));
        }
    }
    assert_eq!(seen.len(), 21);
}

#[test]
fn compose_submission_keeps_user_code_first() {
    let joined = compose_submission(Language::Rust, "fn answer() {}", "#[test]\nfn t() {}");
    assert!(joined.starts_with("fn answer() {}"));
    assert!(joined.contains("#[test]"));
    assert!(joined.contains("判定用テスト"));
    // ユーザーコードが先頭にあることで、診断の行番号がユーザーの行と一致する
    assert_eq!(joined.lines().next(), Some("fn answer() {}"));
}

#[test]
fn compose_submission_uses_python_comment_syntax() {
    // `//` を挿入すると Python は SyntaxError になり、その言語の 300 問が全滅する
    let joined = compose_submission(Language::Python, "def f():\n    pass", "assert f() is None");
    assert!(joined.contains("# ===== 判定用テスト"), "{joined}");
    assert!(!joined.contains("// ====="), "{joined}");
}

#[test]
fn compose_submission_uses_slash_comments_for_the_other_languages() {
    for lang in Language::ALL.into_iter().filter(|l| *l != Language::Python) {
        let joined = compose_submission(lang, "code", "tests");
        assert!(joined.contains("// ===== 判定用テスト"), "{}", lang.slug());
    }
}
