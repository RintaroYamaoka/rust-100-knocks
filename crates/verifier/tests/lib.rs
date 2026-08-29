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
        hidden_tests: "#include <iostream>\nstatic int f=0;\nstatic void check(bool c,const char*n){ if(!c){ std::cerr<<\"FAILED: \"<<n<<\"\\n\"; f++; } }\nint main(){ check(add(1,2)==3,\"add_positive\"); check(add(0,0)==0,\"add_zero\"); if(f){ std::cout<<\"test result: FAILED\\n\"; return 1;} std::cout<<\"test result: ok\\n\"; return 0; }".into(),
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

#[test]
fn distinct_check_names_does_not_count_the_markers() {
    use verifier::distinct_check_names;
    // 目印はヘルパ関数の中に 1 回ずつ現れる定数なので、検査の件数にならない
    let only_helper = r#"
static void chk(bool c, const char* n) { if (!c) { std::cerr << "FAILED: " << n; } }
std::cout << "test result: ok";
std::cout << "test result: FAILED";
"#;
    assert_eq!(distinct_check_names(only_helper), 0, "目印を検査として数えている");
}

#[test]
fn distinct_check_names_counts_each_check() {
    use verifier::distinct_check_names;
    let two = r#"chk(add(1,2)==3, "positive"); chk(add(0,0)==0, "zero");"#;
    assert_eq!(distinct_check_names(two), 2);
    let one = r#"chk(add(1,2)==3, "positive");"#;
    assert_eq!(distinct_check_names(one), 1);
}

#[test]
fn a_single_check_is_rejected_for_non_rust() {
    // 「FAILED: の出現回数」で数えていたときは、検査が何件でも常に 1 だったので
    // この検査が素通りしていた (= 検査していない検査)
    let mut p = cpp_problem("b001");
    p.hidden_tests = concat!(
        "#include <iostream>\n",
        "static int f=0;\n",
        "static void check(bool c,const char*n){ if(!c){ std::cerr<<\"FAILED: \"<<n; f++; } }\n",
        "int main(){ check(add(1,2)==3,\"add_positive\");",
        " if(f){ std::cout<<\"test result: FAILED\"; return 1;}",
        " std::cout<<\"test result: ok\"; return 0; }"
    )
    .to_string();
    let issues = validate_static(&[p], Language::Cpp, Level::Beginner);
    assert!(messages(&issues).contains("検査が 1 件"), "{}", messages(&issues));
}

#[test]
fn file_with_wrong_problem_count_is_rejected() {
    use verifier::validate_static_with_expected;
    // 空ファイルや 50 問しかないファイルが「問題なし」で通らないこと
    let issues = validate_static_with_expected(&[], Language::Cpp, Level::Beginner, Some(100));
    assert!(messages(&issues).contains("問題数が 0 件"), "{}", messages(&issues));

    let issues = validate_static_with_expected(
        &[cpp_problem("b001")],
        Language::Cpp,
        Level::Beginner,
        Some(100),
    );
    assert!(messages(&issues).contains("問題数が 1 件"), "{}", messages(&issues));
}

// ---- 難易度をまたぐ重複 ----

#[test]
fn cross_level_duplicate_titles_are_detected() {
    use verifier::validate_across_levels;
    // validate_static はファイル単位なので、初級と中級に同じ問題を置いても素通りする。
    // 実際に既存 Rust 300 問で b077/i058 と b083/i062 がこれで見逃されていた
    let mut b = rust_problem("b001");
    let mut i = rust_problem("i001");
    b.title = "文字列を反転する".into();
    i.title = "文字列を反転する".into();
    i.level = Level::Intermediate;
    let issues = validate_across_levels(&[b, i]);
    assert!(messages(&issues).contains("title"), "{}", messages(&issues));
}

#[test]
fn cross_level_duplicate_answers_are_detected() {
    use verifier::validate_across_levels;
    let mut b = rust_problem("b001");
    let mut i = rust_problem("i001");
    b.answer_code = "pub fn f() -> i32 { 42 }".into();
    i.answer_code = "pub fn f() -> i32 { 42 }".into();
    i.level = Level::Intermediate;
    let issues = validate_across_levels(&[b, i]);
    assert!(messages(&issues).contains("answer_code"), "{}", messages(&issues));
}

#[test]
fn distinct_problems_across_levels_are_accepted() {
    use verifier::validate_across_levels;
    let b = rust_problem("b001");
    let mut i = rust_problem("i001");
    i.level = Level::Intermediate;
    assert!(validate_across_levels(&[b, i]).is_empty());
}
