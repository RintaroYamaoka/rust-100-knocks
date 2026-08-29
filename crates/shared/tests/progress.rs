use shared::language::Language;
use shared::problem::{Level, Problem};
use shared::progress::{
    filter_problems, matches_filter, migrate_legacy_keys, passed_count, progress_key, status_of,
    ProblemStatus, ProgressEntry, ProgressMap, StatusFilter,
};

fn problem_in(lang: Language, id: &str, title: &str, tags: &[&str]) -> Problem {
    Problem {
        id: id.to_string(),
        language: lang,
        level: Level::Beginner,
        title: title.to_string(),
        description_md: String::new(),
        starter_code: String::new(),
        hidden_tests: String::new(),
        answer_code: String::new(),
        explanation_md: String::new(),
        hints: vec![],
        tags: tags.iter().map(|t| t.to_string()).collect(),
    }
}

fn problem(id: &str, title: &str, tags: &[&str]) -> Problem {
    problem_in(Language::Rust, id, title, tags)
}

fn progress_with(entries: &[(&str, ProblemStatus)]) -> ProgressMap {
    entries
        .iter()
        .map(|(key, st)| {
            (
                key.to_string(),
                ProgressEntry {
                    status: *st,
                    saved_code: None,
                    updated_at_ms: 0.0,
                },
            )
        })
        .collect()
}

#[test]
fn progress_key_is_namespaced_by_language() {
    assert_eq!(progress_key(&problem("b001", "x", &[])), "rust/b001");
    assert_eq!(
        progress_key(&problem_in(Language::Cpp, "b001", "x", &[])),
        "cpp/b001"
    );
}

#[test]
fn same_id_in_two_languages_does_not_collide() {
    // b001 は 7 言語すべてに存在する。名前空間が無いと進捗が混ざる
    let rust = problem_in(Language::Rust, "b001", "x", &[]);
    let java = problem_in(Language::Java, "b001", "x", &[]);
    let map = progress_with(&[("rust/b001", ProblemStatus::Passed)]);
    assert_eq!(status_of(&map, &rust), ProblemStatus::Passed);
    assert_eq!(status_of(&map, &java), ProblemStatus::Unanswered);
}

#[test]
fn status_of_unknown_problem_is_unanswered() {
    assert_eq!(
        status_of(&ProgressMap::new(), &problem("b001", "x", &[])),
        ProblemStatus::Unanswered
    );
}

#[test]
fn migrate_legacy_keys_copies_flat_ids_under_rust() {
    let mut map = progress_with(&[
        ("b001", ProblemStatus::Passed),
        ("i042", ProblemStatus::Attempted),
        ("cpp/b001", ProblemStatus::Attempted),
    ]);
    let migrated = migrate_legacy_keys(&mut map);
    assert_eq!(migrated, 2);
    assert_eq!(map.get("rust/b001").unwrap().status, ProblemStatus::Passed);
    assert_eq!(map.get("rust/i042").unwrap().status, ProblemStatus::Attempted);
    // 既に名前空間つきのキーは触らない
    assert_eq!(map.get("cpp/b001").unwrap().status, ProblemStatus::Attempted);
    // 旧キーは残す (切り戻したとき旧コードが読めるように)
    assert_eq!(map.get("b001").unwrap().status, ProblemStatus::Passed);
}

#[test]
fn migrate_legacy_keys_does_not_overwrite_a_newer_migrated_entry() {
    // updated_at_ms が同じなら既存 (移行済み) を優先する
    let mut map = progress_with(&[
        ("b001", ProblemStatus::Attempted),
        ("rust/b001", ProblemStatus::Passed),
    ]);
    migrate_legacy_keys(&mut map);
    assert_eq!(map.get("rust/b001").unwrap().status, ProblemStatus::Passed);
}

#[test]
fn migrate_legacy_keys_is_idempotent() {
    // 2 回目以降は何も増えない (updated_at_ms が同じなので既存が勝つ)
    let mut map = progress_with(&[("b001", ProblemStatus::Passed)]);
    assert_eq!(migrate_legacy_keys(&mut map), 1);
    assert_eq!(migrate_legacy_keys(&mut map), 0);
    // 旧キーと新キーの 2 件で安定する
    assert_eq!(map.len(), 2);
    assert!(map.contains_key("b001") && map.contains_key("rust/b001"));
}

#[test]
fn matches_filter_semantics() {
    use ProblemStatus::*;
    use StatusFilter::*;
    assert!(matches_filter(Unanswered, All));
    assert!(matches_filter(Unanswered, OnlyUnanswered));
    assert!(!matches_filter(Passed, OnlyUnanswered));
    assert!(matches_filter(Attempted, OnlyAttempted));
    assert!(!matches_filter(Unanswered, OnlyAttempted));
    assert!(matches_filter(Passed, OnlyPassed));
    assert!(!matches_filter(Attempted, OnlyPassed));
}

#[test]
fn filter_problems_by_status_and_query() {
    let problems = vec![
        problem("b001", "変数束縛", &["variables"]),
        problem("b002", "所有権の移動", &["ownership"]),
        problem("b003", "借用", &["ownership", "borrow"]),
    ];
    let map = progress_with(&[("rust/b001", ProblemStatus::Passed)]);

    let unanswered = filter_problems(&problems, &map, StatusFilter::OnlyUnanswered, "");
    assert_eq!(
        unanswered.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        vec!["b002", "b003"]
    );

    let by_tag = filter_problems(&problems, &map, StatusFilter::All, "OWNER");
    assert_eq!(by_tag.len(), 2);
    let by_title = filter_problems(&problems, &map, StatusFilter::All, "借用");
    assert_eq!(by_title.len(), 1);
    let by_id = filter_problems(&problems, &map, StatusFilter::All, "b001");
    assert_eq!(by_id.len(), 1);
}

#[test]
fn passed_count_counts_only_listed_problems() {
    let problems = vec![problem("b001", "a", &[]), problem("b002", "b", &[])];
    let map = progress_with(&[
        ("rust/b001", ProblemStatus::Passed),
        ("rust/a001", ProblemStatus::Passed),
        // 別言語の正解は数えない
        ("cpp/b002", ProblemStatus::Passed),
    ]);
    assert_eq!(passed_count(&problems, &map), 1);
}

#[test]
fn progress_entry_roundtrips_json() {
    let e = ProgressEntry {
        status: ProblemStatus::Attempted,
        saved_code: Some("fn f() {}".into()),
        updated_at_ms: 123.0,
    };
    let s = serde_json::to_string(&e).unwrap();
    let back: ProgressEntry = serde_json::from_str(&s).unwrap();
    assert_eq!(back.status, ProblemStatus::Attempted);
    assert_eq!(back.saved_code.as_deref(), Some("fn f() {}"));
}

// ---- 切り戻しても進捗が消えないこと ----

#[test]
fn migration_does_not_destroy_the_legacy_map() {
    // 旧キーを削除すると、切り戻したとき旧コードが読めるものが何も無くなる。
    // 移行は「新しい形を作る」だけで、元は残す
    let mut map = progress_with(&[("b001", ProblemStatus::Passed), ("i042", ProblemStatus::Attempted)]);
    migrate_legacy_keys(&mut map);
    assert!(map.contains_key("b001"), "旧キーが消えている (切り戻すと進捗が全滅する)");
    assert!(map.contains_key("rust/b001"), "新キーが作られていない");
    assert_eq!(map.get("b001").unwrap().status, ProblemStatus::Passed);
    assert_eq!(map.get("rust/b001").unwrap().status, ProblemStatus::Passed);
}

#[test]
fn migration_keeps_the_newer_entry_on_conflict() {
    // 切り戻していた間に旧コードが b001 を更新し、その後また前進した場合。
    // updated_at_ms を見ないと、戻っていた間の学習がすべて捨てられる
    let mut map = ProgressMap::new();
    map.insert(
        "rust/b001".into(),
        ProgressEntry { status: ProblemStatus::Attempted, saved_code: Some("古い".into()), updated_at_ms: 100.0 },
    );
    map.insert(
        "b001".into(),
        ProgressEntry { status: ProblemStatus::Passed, saved_code: Some("新しい".into()), updated_at_ms: 200.0 },
    );
    migrate_legacy_keys(&mut map);
    let e = map.get("rust/b001").unwrap();
    assert_eq!(e.status, ProblemStatus::Passed, "新しい方が採られていない");
    assert_eq!(e.saved_code.as_deref(), Some("新しい"));
}

#[test]
fn migration_keeps_the_migrated_entry_when_it_is_newer() {
    let mut map = ProgressMap::new();
    map.insert(
        "rust/b001".into(),
        ProgressEntry { status: ProblemStatus::Passed, saved_code: Some("新しい".into()), updated_at_ms: 300.0 },
    );
    map.insert(
        "b001".into(),
        ProgressEntry { status: ProblemStatus::Attempted, saved_code: Some("古い".into()), updated_at_ms: 50.0 },
    );
    migrate_legacy_keys(&mut map);
    assert_eq!(map.get("rust/b001").unwrap().saved_code.as_deref(), Some("新しい"));
}
