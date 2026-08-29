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
fn migrate_legacy_keys_moves_flat_ids_under_rust() {
    let mut map = progress_with(&[
        ("b001", ProblemStatus::Passed),
        ("i042", ProblemStatus::Attempted),
        ("cpp/b001", ProblemStatus::Attempted),
    ]);
    let moved = migrate_legacy_keys(&mut map);
    assert_eq!(moved, 2);
    assert_eq!(map.get("rust/b001").unwrap().status, ProblemStatus::Passed);
    assert_eq!(map.get("rust/i042").unwrap().status, ProblemStatus::Attempted);
    // 既に名前空間つきのキーは触らない
    assert_eq!(map.get("cpp/b001").unwrap().status, ProblemStatus::Attempted);
    assert!(map.get("b001").is_none());
}

#[test]
fn migrate_legacy_keys_does_not_overwrite_existing_migrated_entry() {
    let mut map = progress_with(&[
        ("b001", ProblemStatus::Attempted),
        ("rust/b001", ProblemStatus::Passed),
    ]);
    migrate_legacy_keys(&mut map);
    assert_eq!(map.get("rust/b001").unwrap().status, ProblemStatus::Passed);
}

#[test]
fn migrate_legacy_keys_is_idempotent() {
    let mut map = progress_with(&[("b001", ProblemStatus::Passed)]);
    assert_eq!(migrate_legacy_keys(&mut map), 1);
    assert_eq!(migrate_legacy_keys(&mut map), 0);
    assert_eq!(map.len(), 1);
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
