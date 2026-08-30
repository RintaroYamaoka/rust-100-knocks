use app::mobile::{pane_after_problem_change, MobilePane};

#[test]
fn slugs_are_unique_and_stable() {
    // CSS の `.main[data-pane="..."]` セレクタがこの文字列に依存している
    let slugs: Vec<&str> = MobilePane::ALL.iter().map(|p| p.slug()).collect();
    assert_eq!(slugs, vec!["list", "problem", "code"]);
}

#[test]
fn labels_are_present_for_every_pane() {
    for p in MobilePane::ALL {
        assert!(!p.label_ja().is_empty());
    }
}

#[test]
fn all_covers_every_variant() {
    // 下部タブは ALL から作るので、増やしたときにタブから漏れないことを固定する
    assert_eq!(MobilePane::ALL.len(), 3);
    assert!(MobilePane::ALL.contains(&MobilePane::List));
    assert!(MobilePane::ALL.contains(&MobilePane::Problem));
    assert!(MobilePane::ALL.contains(&MobilePane::Code));
}

#[test]
fn moving_to_another_problem_shows_the_statement() {
    // 一覧から選んでも、前後移動でも、まず問題文を見せる
    assert_eq!(pane_after_problem_change(), MobilePane::Problem);
}
