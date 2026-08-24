//! editor は wasm 専用の JS interop 層。host ではスタブがコンパイルされることのみ担保する
//! (実 DOM 上の挙動は tests/editor-src.test.mjs と Playwright スクリーンショットで確認)。

#[test]
fn editor_stub_compiles_on_host() {
    app::editor::set_value("x");
    assert_eq!(app::editor::get_value(), "");
}
