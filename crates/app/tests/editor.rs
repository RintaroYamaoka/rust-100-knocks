//! editor は wasm 専用の JS interop 層。host ではスタブがコンパイルされることのみ担保する
//! (実 DOM 上の挙動は tests/editor-src.test.mjs と Playwright スクリーンショットで確認)。

use shared::language::Language;

#[test]
fn editor_stub_compiles_on_host() {
    app::editor::set_value("x");
    assert_eq!(app::editor::get_value(), "");
}

#[test]
fn set_language_accepts_every_language() {
    // 言語切替はエディタの再マウントではなく setLanguage 経由 (下書きを壊さないため)。
    // host では no-op だが、7 言語すべてが interop の入口を通ることは固定しておく。
    for lang in Language::ALL {
        app::editor::set_language(lang);
    }
}
