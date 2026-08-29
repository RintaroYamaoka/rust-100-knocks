//! 言語選択の純ロジック。
//!
//! 「未完成の言語をセレクタに出さない」「保存した選択を復元する」「Rust 以外に
//! rustc 専用のリンクを張らない」は、いずれも UI を動かさずに決まる規則なので
//! ここで固定する。

use std::collections::HashSet;

use app::lang::{
    backend_label, console_idle_hint, console_running_hint, initial_language,
    languages_with_full_data, links_error_codes, resolve_selection, selector_languages,
};
use shared::language::Language;
use shared::problem::Level;

fn found(pairs: &[(Language, &[Level])]) -> HashSet<(Language, Level)> {
    let mut set = HashSet::new();
    for (lang, levels) in pairs {
        for lv in *levels {
            set.insert((*lang, *lv));
        }
    }
    set
}

const ALL_LEVELS: &[Level] = &[Level::Beginner, Level::Intermediate, Level::Advanced];

#[test]
fn language_needs_all_three_levels_to_appear() {
    let set = found(&[
        (Language::Rust, ALL_LEVELS),
        // 初級だけしか無い言語は未完成なので出さない
        (Language::Python, &[Level::Beginner]),
    ]);
    assert_eq!(languages_with_full_data(&set), vec![Language::Rust]);
}

#[test]
fn available_languages_keep_the_canonical_order() {
    let set = found(&[
        (Language::Javascript, ALL_LEVELS),
        (Language::Rust, ALL_LEVELS),
        (Language::Java, ALL_LEVELS),
    ]);
    assert_eq!(
        languages_with_full_data(&set),
        vec![Language::Rust, Language::Java, Language::Javascript]
    );
}

#[test]
fn no_data_means_no_languages() {
    assert!(languages_with_full_data(&HashSet::new()).is_empty());
}

#[test]
fn initial_language_restores_stored_slug() {
    assert_eq!(initial_language(Some("python")), Language::Python);
}

#[test]
fn initial_language_falls_back_to_rust() {
    // 未保存・壊れた値・かつて存在しない slug、いずれも従来の既定へ
    assert_eq!(initial_language(None), Language::Rust);
    assert_eq!(initial_language(Some("")), Language::Rust);
    assert_eq!(initial_language(Some("cobol")), Language::Rust);
}

#[test]
fn selection_moves_off_a_language_that_has_no_data() {
    let available = [Language::Rust, Language::Python];
    assert_eq!(resolve_selection(Language::Java, &available), Language::Rust);
    // 有効ならそのまま
    assert_eq!(
        resolve_selection(Language::Python, &available),
        Language::Python
    );
}

#[test]
fn selection_is_left_alone_when_availability_is_unknown() {
    // 確認が機能しなかった環境で選択を動かすと、動いていた言語まで巻き添えになる
    assert_eq!(resolve_selection(Language::Rust, &[]), Language::Rust);
}

#[test]
fn selector_shows_the_available_languages() {
    let available = [Language::Rust, Language::Cpp];
    assert_eq!(
        selector_languages(&available, Language::Rust, true),
        vec![Language::Rust, Language::Cpp]
    );
}

#[test]
fn selector_falls_back_to_the_loaded_language_when_probing_failed() {
    assert_eq!(
        selector_languages(&[], Language::Rust, true),
        vec![Language::Rust]
    );
    // 何も読めていなくても**空にはしない**。
    // option が 0 個だと、保存された言語のデータが無くなった利用者に
    // 「別の言語に戻す手段」が UI 上から消えてしまう (症状は「一覧が空」だけで
    // 原因が見えない)。必ず存在が保証されている Rust を残す。
    assert_eq!(
        selector_languages(&[], Language::Rust, false),
        vec![Language::Rust]
    );
}

#[test]
fn error_code_links_are_rust_only() {
    assert!(links_error_codes(Language::Rust));
    for lang in Language::ALL {
        if lang != Language::Rust {
            assert!(!links_error_codes(lang), "{} でリンクを張っている", lang.label());
        }
    }
}

#[test]
fn backend_label_names_the_service_that_actually_runs_the_code() {
    assert_eq!(backend_label(Language::Rust), "Rust Playground");
    for lang in Language::ALL {
        if lang != Language::Rust {
            assert_eq!(backend_label(lang), "Wandbox");
        }
    }
}

#[test]
fn console_wording_is_not_hardcoded_to_rust() {
    // 待機中の文言はどの言語でも成り立つこと
    assert!(!console_idle_hint().contains("rustc"));
    assert!(!console_idle_hint().contains("Rust"));

    // 実行中の文言は選択中の言語に追従すること
    let py = console_running_hint(Language::Python);
    assert!(py.contains("Python"), "{py}");
    assert!(py.contains("Wandbox"), "{py}");
    assert!(!py.contains("Rust"), "{py}");

    let rs = console_running_hint(Language::Rust);
    assert!(rs.contains("Rust Playground"), "{rs}");
}

#[test]
fn selector_never_becomes_empty() {
    // option が 0 個になると、保存された言語のデータが無くなった利用者に
    // 「別の言語に戻す手段」が UI 上から消える
    let none: Vec<Language> = vec![];
    let r = selector_languages(&none, Language::Python, false);
    assert!(!r.is_empty(), "セレクタが空になった");
    assert!(r.contains(&Language::Rust), "既定の Rust が残っていない: {r:?}");

    let r = selector_languages(&none, Language::Rust, false);
    assert_eq!(r, vec![Language::Rust]);
}

#[test]
fn selector_prefers_confirmed_languages_when_available() {
    let avail = vec![Language::Rust, Language::Python];
    assert_eq!(
        selector_languages(&avail, Language::Python, true),
        vec![Language::Rust, Language::Python]
    );
}
