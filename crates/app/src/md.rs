use pulldown_cmark::{html, Event, Options, Parser};

/// Markdown → HTML。生 HTML はテキストへ落として無害化する
/// (問題文・解説は信頼データだが、描画層としては常に安全側に倒す)。
pub fn render_markdown(md: &str) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(md, opts).map(|ev| match ev {
        Event::Html(s) | Event::InlineHtml(s) => Event::Text(s),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}
