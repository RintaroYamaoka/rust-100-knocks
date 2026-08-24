use app::md::render_markdown;

#[test]
fn renders_code_fence_and_inline_code() {
    let html = render_markdown("説明 `let x = 1;`\n\n```rust\nfn main() {}\n```");
    assert!(html.contains("<code>let x = 1;</code>"));
    assert!(html.contains("<pre><code class=\"language-rust\">"));
}

#[test]
fn renders_headings_lists_tables() {
    let html = render_markdown("## 見出し\n\n- 項目1\n\n| a | b |\n|---|---|\n| 1 | 2 |");
    assert!(html.contains("<h2>見出し</h2>"));
    assert!(html.contains("<li>項目1</li>"));
    assert!(html.contains("<table>"));
}

#[test]
fn escapes_raw_html() {
    // 問題文は自作データだが、生 HTML の混入は描画時に無害化されていること
    let html = render_markdown("<script>alert(1)</script>");
    assert!(!html.contains("<script>alert(1)</script>"));
}
