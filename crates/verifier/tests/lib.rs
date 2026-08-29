use shared::language::Language;
use shared::problem::{Level, Problem};
use verifier::{validate_static, MIN_DESCRIPTION_CHARS};

/// 収録済み 300 問の最短の説明が 116 文字なので、閾値 (80) を余裕で超える長さにしておく。
fn description() -> String {
    "この問題では整数を 2 倍にして返す関数を実装します。関数の最後の式がそのまま戻り値になること、\
     セミコロンを付けると文になって `()` が返ってしまうことを確認しましょう。\
     負数と 0 の境界値も忘れずに確かめてください。"
        .to_string()
}

fn rust_problem(id: &str) -> Problem {
    Problem {
        id: id.to_string(),
        language: Language::Rust,
        level: Level::Beginner,
        title: format!("テスト用 {id}"),
        description_md: description(),
        starter_code: "pub fn answer() -> i32 {\n    todo!()\n}".into(),
        hidden_tests: "#[test]\nfn a() { assert_eq!(answer(), 42); }\n#[test]\nfn b() { assert!(answer() > 0); }".into(),
        answer_code: format!("pub fn answer() -> i32 {{ 42 }} // {id}"),
        explanation_md: "解説".into(),
        hints: vec![],
        tags: vec!["test".into()],
    }
}

fn cpp_problem(id: &str) -> Problem {
    Problem {
        id: id.to_string(),
        language: Language::Cpp,
        level: Level::Beginner,
        title: format!("テスト用 {id}"),
        description_md: description(),
        starter_code: "int add(int a, int b) {\n    return 0; // TODO\n}".into(),
        hidden_tests: "#include <iostream>\nstatic int f=0;\nstatic void check(bool c,const char*n){ if(!c){ std::cerr<<\"FAILED: \"<<n<<\"\\n\"; f++; } }\nint main(){ check(add(1,2)==3,\"a\"); check(add(0,0)==0,\"b\"); if(f){ std::cout<<\"test result: FAILED\\n\"; return 1;} std::cout<<\"test result: ok\\n\"; return 0; }".into(),
        answer_code: format!("int add(int a, int b) {{ return a + b; }} // {id}"),
        explanation_md: "解説".into(),
        hints: vec![],
        tags: vec!["test".into()],
    }
}

fn messages(issues: &[verifier::ProblemIssue]) -> String {
    issues.iter().map(|i| format!("[{}] {}", i.id, i.message)).collect::<Vec<_>>().join("\n")
}

#[test]
fn accepts_wellformed_rust_problem() {
    let issues = validate_static(&[rust_problem("b001")], Language::Rust, Level::Beginner);
    assert!(issues.is_empty(), "{}", messages(&issues));
}

#[test]
fn accepts_wellformed_cpp_problem() {
    let issues = validate_static(&[cpp_problem("b001")], Language::Cpp, Level::Beginner);
    assert!(issues.is_empty(), "{}", messages(&issues));
}

#[test]
fn rejects_language_mismatch_against_the_path() {
    // パスが正本。cpp のファイルに rust の問題が紛れ込んだら止める
    let issues = validate_static(&[rust_problem("b001")], Language::Cpp, Level::Beginner);
    assert!(messages(&issues).contains("language"), "{}", messages(&issues));
}

#[test]
fn rejects_level_mismatch() {
    let issues = validate_static(&[rust_problem("b001")], Language::Rust, Level::Advanced);
    assert!(!issues.is_empty());
}

#[test]
fn rejects_duplicate_ids() {
    let issues = validate_static(
        &[rust_problem("b001"), rust_problem("b001")],
        Language::Rust,
        Level::Beginner,
    );
    assert!(messages(&issues).contains("id が重複"), "{}", messages(&issues));
}

#[test]
fn rejects_duplicate_titles() {
    // 1 問をコピーした 100 問は、実行検査だけなら全部通ってしまう
    let mut a = rust_problem("b001");
    let mut b = rust_problem("b002");
    a.title = "同じ題名".into();
    b.title = "同じ題名".into();
    let issues = validate_static(&[a, b], Language::Rust, Level::Beginner);
    assert!(messages(&issues).contains("title が"), "{}", messages(&issues));
}

#[test]
fn rejects_duplicate_answer_code() {
    let mut a = rust_problem("b001");
    let mut b = rust_problem("b002");
    a.answer_code = "pub fn answer() -> i32 { 42 }".into();
    b.answer_code = "pub fn answer() -> i32 { 42 }".into();
    let issues = validate_static(&[a, b], Language::Rust, Level::Beginner);
    assert!(messages(&issues).contains("answer_code が"), "{}", messages(&issues));
}

#[test]
fn rejects_short_description() {
    let mut p = rust_problem("b001");
    p.description_md = "みじかい".into();
    let issues = validate_static(&[p], Language::Rust, Level::Beginner);
    assert!(
        messages(&issues).contains(&MIN_DESCRIPTION_CHARS.to_string()),
        "{}",
        messages(&issues)
    );
}

#[test]
fn rejects_identical_starter_and_answer() {
    let mut p = rust_problem("b001");
    p.starter_code = p.answer_code.clone();
    let issues = validate_static(&[p], Language::Rust, Level::Beginner);
    assert!(messages(&issues).contains("同一"), "{}", messages(&issues));
}

#[test]
fn rust_requires_at_least_two_tests() {
    let mut p = rust_problem("b001");
    p.hidden_tests = "#[test]\nfn only() { assert!(true); }".into();
    let issues = validate_static(&[p], Language::Rust, Level::Beginner);
    assert!(messages(&issues).contains("#[test]"), "{}", messages(&issues));
}

#[test]
fn non_rust_requires_both_markers() {
    // Rust の #[test] を要求する検査をそのまま他言語に当てると全問落ちる。
    // 言語別に「何をもってテストとみなすか」を切り替える
    let mut p = cpp_problem("b001");
    p.hidden_tests = p.hidden_tests.replace("test result: ok", "ぜんぶOK");
    let issues = validate_static(&[p], Language::Cpp, Level::Beginner);
    assert!(messages(&issues).contains("成功の目印"), "{}", messages(&issues));

    let mut p = cpp_problem("b001");
    p.hidden_tests = p.hidden_tests.replace("test result: FAILED", "だめ");
    let issues = validate_static(&[p], Language::Cpp, Level::Beginner);
    assert!(messages(&issues).contains("失敗の目印"), "{}", messages(&issues));
}

#[test]
fn non_rust_is_not_asked_for_rust_test_attribute() {
    let issues = validate_static(&[cpp_problem("b001")], Language::Cpp, Level::Beginner);
    assert!(!messages(&issues).contains("#[test]"), "{}", messages(&issues));
}

#[test]
fn rejects_empty_hidden_tests() {
    let mut p = rust_problem("b001");
    p.hidden_tests = String::new();
    let issues = validate_static(&[p], Language::Rust, Level::Beginner);
    assert!(messages(&issues).contains("hidden_tests が空"), "{}", messages(&issues));
}

#[test]
fn rejects_bad_id_format() {
    let issues = validate_static(&[rust_problem("beginner-1")], Language::Rust, Level::Beginner);
    assert!(messages(&issues).contains("id の形式"), "{}", messages(&issues));
}
