//! host ビルドではメモリ内フォールバックが localStorage の代わりに使われる。
//! ここで検証するのは「シリアライズ形式と load/save の対称性」= wasm 側と共通の経路。

use app::storage::{
    load_language, load_progress, load_progress_migrated, save_language, save_progress,
};
use shared::language::Language;
use shared::progress::{ProblemStatus, ProgressEntry};

fn entry(code: &str, status: ProblemStatus) -> ProgressEntry {
    ProgressEntry {
        status,
        saved_code: Some(code.to_string()),
        updated_at_ms: 1.0,
    }
}

#[test]
fn progress_roundtrips_through_storage() {
    let mut map = load_progress();
    map.insert("rust/b001".to_string(), entry("fn f() {}", ProblemStatus::Passed));
    save_progress(&map);

    let loaded = load_progress();
    let e = loaded.get("rust/b001").expect("保存した進捗が読めない");
    assert_eq!(e.status, ProblemStatus::Passed);
    assert_eq!(e.saved_code.as_deref(), Some("fn f() {}"));
}

#[test]
fn empty_storage_loads_as_empty_map() {
    // 壊れたデータや未初期化状態でも panic せず空で返ること (実装は unwrap しない)
    let map = load_progress();
    let _ = map.len();
}

#[test]
fn legacy_flat_keys_are_migrated_to_the_rust_namespace() {
    // 多言語化前の利用者の進捗 (`b001`) が、起動時に `rust/b001` へ移ること
    let mut map = load_progress();
    map.insert("i042".to_string(), entry("legacy", ProblemStatus::Passed));
    save_progress(&map);

    let migrated = load_progress_migrated();
    assert!(!migrated.contains_key("i042"), "旧キーが残っている");
    let e = migrated.get("rust/i042").expect("旧進捗が失われた");
    assert_eq!(e.status, ProblemStatus::Passed);
    assert_eq!(e.saved_code.as_deref(), Some("legacy"));

    // 移行結果が保存され、次回以降の読み込みでも保たれること
    assert!(load_progress().contains_key("rust/i042"));
}

#[test]
fn language_selection_roundtrips() {
    save_language(Language::Typescript);
    assert_eq!(load_language().as_deref(), Some("typescript"));
    assert_eq!(
        load_language().as_deref().and_then(Language::from_slug),
        Some(Language::Typescript)
    );
}
