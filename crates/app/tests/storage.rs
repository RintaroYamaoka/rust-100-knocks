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
    use app::storage::{raw_get, LEGACY_PROGRESS_KEY};
    // 多言語化前の利用者の進捗は v1 に入っている。それが `rust/i042` として読めること
    app::storage::raw_set(
        LEGACY_PROGRESS_KEY,
        r#"{"i042":{"status":"passed","saved_code":"legacy","updated_at_ms":1.0}}"#,
    );

    let (migrated, save_failed) = load_progress_migrated();
    assert!(!save_failed, "移行を保存できていない");
    let e = migrated.get("rust/i042").expect("旧進捗が失われた");
    assert_eq!(e.status, ProblemStatus::Passed);
    assert_eq!(e.saved_code.as_deref(), Some("legacy"));

    // v1 は無傷 (切り戻したとき前の版がこれを読む)
    let v1 = raw_get(LEGACY_PROGRESS_KEY).expect("v1 が消えている");
    assert!(v1.contains("\"i042\""), "v1 の中身が壊れている: {v1}");

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

// ---- 切り戻しても旧版が読めること ----

#[test]
fn legacy_storage_key_is_left_intact_after_migration() {
    use app::storage::{load_progress_migrated, raw_get, raw_set, LEGACY_PROGRESS_KEY, PROGRESS_KEY};

    // 前の版が書いた進捗
    raw_set(
        LEGACY_PROGRESS_KEY,
        r#"{"b001":{"status":"passed","saved_code":"fn f(){}","updated_at_ms":1.0}}"#,
    );

    let (map, _) = load_progress_migrated();
    assert_eq!(map.get("rust/b001").map(|e| e.status), Some(shared::progress::ProblemStatus::Passed));

    // 新しいキーに書かれ、**旧キーはそのまま残っている**。
    // 消すと、この版を開いた利用者を前の版に戻したとき進捗が全部消える
    assert!(raw_get(PROGRESS_KEY).is_some(), "新しいキーに保存されていない");
    let legacy = raw_get(LEGACY_PROGRESS_KEY).expect("旧キーが消えている (切り戻すと進捗が全滅する)");
    assert!(legacy.contains("\"b001\""), "旧キーの中身が壊れている: {legacy}");
}

#[test]
fn new_key_wins_when_both_exist() {
    use app::storage::{load_progress_migrated, raw_set, LEGACY_PROGRESS_KEY, PROGRESS_KEY};
    raw_set(LEGACY_PROGRESS_KEY, r#"{"b002":{"status":"attempted","updated_at_ms":1.0}}"#);
    raw_set(
        PROGRESS_KEY,
        r#"{"rust/b002":{"status":"passed","updated_at_ms":9.0}}"#,
    );
    let (map, _) = load_progress_migrated();
    assert_eq!(
        map.get("rust/b002").map(|e| e.status),
        Some(shared::progress::ProblemStatus::Passed),
        "新しいキーの内容が旧キーで上書きされている"
    );
}

#[test]
fn v2_holds_only_namespaced_keys() {
    use app::storage::{load_progress, load_progress_migrated, raw_set, LEGACY_PROGRESS_KEY};
    // 旧フラットキーは v1 に残すが、v2 に持ち込むのは無駄
    // (新しい版は "b003" を誰も読まない。下書きを含むので容量が数倍になる)
    raw_set(
        LEGACY_PROGRESS_KEY,
        r#"{"b003":{"status":"passed","saved_code":"長い下書き","updated_at_ms":1.0}}"#,
    );
    let (_, _) = load_progress_migrated();
    let saved = load_progress();
    assert!(saved.contains_key("rust/b003"), "移行されていない");
    assert!(
        !saved.contains_key("b003"),
        "v2 に旧フラットキーが保存されている (容量が無駄に増える)"
    );
}
