//! crate 公開面のスモーク: 主要モジュールが host からリンク可能であること。

#[test]
fn public_modules_are_linkable() {
    let _ = app::md::render_markdown("x");
    let _ = app::console::split_error_codes("x");
    let _ = app::api::error_message_from_body(500, "");
}
