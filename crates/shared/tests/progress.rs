use shared::problem::{Level, Problem};
use shared::progress::{
    filter_problems, matches_filter, passed_count, status_of, ProblemStatus, ProgressEntry,
    ProgressMap, StatusFilter,
};

fn problem(id: &str, title: &str, tags: &[&str]) -> Problem {
    Problem {
        id: id.to_string(),
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

fn progress_with(entries: &[(&str, ProblemStatus)]) -> ProgressMap {
    entries
        .iter()
        .map(|(id, st)| {
            (
                id.to_string(),
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
fn status_of_unknown_id_is_unanswered() {
    let map = ProgressMap::new();
    assert_eq!(status_of(&map, "b001"), ProblemStatus::Unanswered);
}

#[test]
fn status_of_reads_stored_status() {
    let map = progress_with(&[("b001", ProblemStatus::Passed), ("b002", ProblemStatus::Attempted)]);
    assert_eq!(status_of(&map, "b001"), ProblemStatus::Passed);
    assert_eq!(status_of(&map, "b002"), ProblemStatus::Attempted);
}

#[test]
fn matches_filter_semantics() {
    use ProblemStatus::*;
    use StatusFilter::*;
    assert!(matches_filter(Unanswered, All));
    assert!(matches_filter(Unanswered, OnlyUnanswered));
    assert!(!matches_filter(Passed, OnlyUnanswered));
    // 「未正解」= 挑戦したがまだ正解していない
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
    let map = progress_with(&[("b001", ProblemStatus::Passed)]);

    let unanswered = filter_problems(&problems, &map, StatusFilter::OnlyUnanswered, "");
    assert_eq!(
        unanswered.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        vec!["b002", "b003"]
    );

    // クエリは id / title / tags に対して大文字小文字を無視してマッチ
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
    // 別レベルの正解 (a001) は数えない
    let map = progress_with(&[("b001", ProblemStatus::Passed), ("a001", ProblemStatus::Passed)]);
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
