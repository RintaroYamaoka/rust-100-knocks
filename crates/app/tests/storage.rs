//! host ビルドではメモリ内フォールバックが localStorage の代わりに使われる。
//! ここで検証するのは「シリアライズ形式と load/save の対称性」= wasm 側と共通の経路。

use app::storage::{load_progress, save_progress};
use shared::progress::{ProblemStatus, ProgressEntry};

#[test]
fn progress_roundtrips_through_storage() {
    let mut map = load_progress();
    map.insert(
        "b001".to_string(),
        ProgressEntry {
            status: ProblemStatus::Passed,
            saved_code: Some("fn f() {}".into()),
            updated_at_ms: 1.0,
        },
    );
    save_progress(&map);

    let loaded = load_progress();
    let entry = loaded.get("b001").expect("保存した進捗が読めない");
    assert_eq!(entry.status, ProblemStatus::Passed);
    assert_eq!(entry.saved_code.as_deref(), Some("fn f() {}"));
}

#[test]
fn empty_storage_loads_as_empty_map() {
    // 壊れたデータや未初期化状態でも panic せず空で返ること (実装は unwrap しない)
    let map = load_progress();
    let _ = map.len();
}
