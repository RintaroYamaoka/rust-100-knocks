//! 一覧の絞り込みロジックは shared::progress 側で単体テスト済み。
//! ここではコンポーネント層が host でコンパイルできることのみ担保する。

#[test]
fn list_component_compiles_on_host() {
    // list モジュールがリンクされていればよい
}
