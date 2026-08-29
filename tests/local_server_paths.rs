//! ローカル検証サーバーの純ロジック (静的ファイルのパス解決)。
//!
//! このサーバーが存在する理由そのものが「判定ロジックを二重に持たない」なので、
//! 実行まわりのテストは `tests/execute.rs` と shared 側が持つ。ここで固定するのは
//! dist の外に出られないことだけ。

use std::path::Path;

// バイナリのモジュールを直接テストできないので、同じ実装を参照する。
#[path = "../tools/local_server_paths.rs"]
mod paths;

use paths::{percent_decode, resolve};

#[test]
fn serves_index_for_root() {
    let p = resolve(Path::new("/dist"), "/").unwrap();
    assert_eq!(p, Path::new("/dist/index.html"));
}

#[test]
fn serves_nested_problem_data() {
    let p = resolve(Path::new("/dist"), "/data/problems/cpp/beginner.json").unwrap();
    assert_eq!(p, Path::new("/dist/data/problems/cpp/beginner.json"));
}

#[test]
fn rejects_parent_directory_traversal() {
    // dist の外へ出られないこと
    assert!(resolve(Path::new("/dist"), "/../etc/passwd").is_none());
    assert!(resolve(Path::new("/dist"), "/data/../../etc/passwd").is_none());
    assert!(resolve(Path::new("/dist"), "/a/b/../../../secret").is_none());
}

#[test]
fn rejects_encoded_traversal() {
    // %2e%2e = ".." を decode してから検査すること
    assert!(resolve(Path::new("/dist"), "/%2e%2e/etc/passwd").is_none());
    assert!(resolve(Path::new("/dist"), "/data/%2E%2E/%2E%2E/etc/passwd").is_none());
}

#[test]
fn rejects_empty_segments() {
    assert!(resolve(Path::new("/dist"), "/data//problems").is_none());
}

#[test]
fn percent_decode_handles_utf8_and_malformed_input() {
    assert_eq!(percent_decode("a%2Fb"), "a/b");
    assert_eq!(percent_decode("%E6%97%A5"), "日");
    // 壊れたエスケープはそのまま残す (panic しない)
    assert_eq!(percent_decode("100%"), "100%");
    assert_eq!(percent_decode("%ZZ"), "%ZZ");
}
