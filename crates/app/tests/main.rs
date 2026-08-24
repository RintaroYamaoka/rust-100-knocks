//! app crate は wasm32 専用 UI。host で走るテストは純ロジック (shared 側) に置き、
//! ここでは host ビルドが壊れていないことのコンパイル時スモークのみ担保する。

#[test]
fn app_crate_compiles_on_host() {
    // このテストがリンクできている時点で app の host コンパイルは成立している。
}
