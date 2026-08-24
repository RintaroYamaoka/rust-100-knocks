//! Leptos フロントエンド。UI 配線は app モジュール、純ロジックは各モジュールに分離し
//! host (非 wasm) でもコンパイル・テストできる形を保つ。

pub mod api;
pub mod app;
pub mod console;
pub mod editor;
pub mod list;
pub mod md;
pub mod problem_view;
pub mod storage;

pub use app::next_status;
