//! 実行結果コンソール。rustc 出力を行種別で色分けし、エラーコードは公式解説へリンクする。

use leptos::prelude::*;
use shared::playground::{classify_line, ExecuteResponse, LineKind, Outcome};

#[derive(Clone, Debug, PartialEq)]
pub enum RunState {
    Idle,
    Running,
    Done { resp: ExecuteResponse, outcome: Outcome },
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsoleSegment {
    Text(String),
    ErrorCode(String),
}

/// 1 行を「エラーコード (E0308 等)」とそれ以外のテキストに分割する。
pub fn split_error_codes(line: &str) -> Vec<ConsoleSegment> {
    let mut segs = Vec::new();
    let mut cursor = 0usize;
    let mut search_from = 0usize;
    while let Some(rel) = line[search_from..].find("error[") {
        let code_start = search_from + rel + "error[".len();
        let Some(end_rel) = line[code_start..].find(']') else { break };
        let code = &line[code_start..code_start + end_rel];
        let valid = code.len() == 5
            && code.starts_with('E')
            && code[1..].chars().all(|c| c.is_ascii_digit());
        if valid {
            if code_start > cursor {
                segs.push(ConsoleSegment::Text(line[cursor..code_start].to_string()));
            }
            segs.push(ConsoleSegment::ErrorCode(code.to_string()));
            cursor = code_start + end_rel;
            search_from = cursor;
        } else {
            search_from = code_start;
        }
    }
    if cursor < line.len() || segs.is_empty() {
        segs.push(ConsoleSegment::Text(line[cursor..].to_string()));
    }
    segs
}

fn line_class(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Error => "line-error",
        LineKind::Warning => "line-warning",
        LineKind::Note => "line-note",
        LineKind::Plain => "line-plain",
    }
}

fn render_stderr_line(line: &str) -> impl IntoView {
    let class = line_class(classify_line(line));
    let segs = split_error_codes(line)
        .into_iter()
        .map(|seg| match seg {
            ConsoleSegment::Text(t) => view! { <span>{t}</span> }.into_any(),
            ConsoleSegment::ErrorCode(c) => {
                let href = format!("https://doc.rust-lang.org/error_codes/{c}.html");
                view! {
                    <a class="error-code-link" href=href target="_blank" rel="noreferrer" title="エラーコードの公式解説を開く">{c}</a>
                }
                .into_any()
            }
        })
        .collect_view();
    view! { <div class=class>{segs}</div> }
}

fn outcome_banner(outcome: Outcome) -> impl IntoView {
    let (class, label) = match outcome {
        Outcome::Passed => ("outcome-banner passed", "✓ 正解!"),
        Outcome::TestsFailed => ("outcome-banner tests-failed", "△ テスト失敗 — もう一歩"),
        Outcome::CompileError => ("outcome-banner compile-error", "✗ コンパイルエラー"),
        Outcome::RuntimeError => ("outcome-banner runtime-error", "✗ 実行時エラー"),
    };
    view! { <span class=class>{label}</span> }
}

#[component]
pub fn ConsolePane(state: Signal<RunState>) -> impl IntoView {
    let head_extra = move || match state.get() {
        RunState::Running => Some(view! { <span class="outcome-banner"><span class="spinner"></span>" コンパイル・実行中…"</span> }.into_any()),
        RunState::Done { outcome, .. } => Some(outcome_banner(outcome).into_any()),
        RunState::Failed(_) => Some(view! { <span class="outcome-banner upstream-error">"! 実行できませんでした"</span> }.into_any()),
        RunState::Idle => None,
    };

    let body = move || match state.get() {
        RunState::Idle => view! {
            <div class="console-placeholder">"コードを書いて「実行して判定」(Ctrl+Enter) を押すと、ここに rustc の出力と判定結果が表示されます。"</div>
        }
        .into_any(),
        RunState::Running => view! {
            <div class="console-placeholder">"Rust Playground でコンパイル・実行しています…"</div>
        }
        .into_any(),
        RunState::Failed(msg) => view! { <div class="line-error">{msg}</div> }.into_any(),
        RunState::Done { resp, outcome } => {
            let stderr_view = (!resp.stderr.trim().is_empty()).then(|| {
                let lines = resp.stderr.lines().map(render_stderr_line).collect_view();
                view! {
                    <div class="console-section-label">"コンパイラ出力 (stderr)"</div>
                    <div>{lines}</div>
                }
            });
            let stdout_view = (!resp.stdout.trim().is_empty()).then(|| {
                view! {
                    <div class="console-section-label">"実行出力 (stdout)"</div>
                    <div class="stdout-block">{resp.stdout.clone()}</div>
                }
            });
            // コンパイルエラー時は stderr を先に、それ以外はテスト結果 (stdout) を先に見せる
            if outcome == Outcome::CompileError {
                view! {
                    {stderr_view}
                    {stdout_view}
                }
                .into_any()
            } else {
                view! {
                    {stdout_view}
                    {stderr_view}
                }
                .into_any()
            }
        }
    };

    view! {
        <div class="console">
            <div class="console-head">"実行結果" {head_extra}</div>
            <div class="console-body">{body}</div>
        </div>
    }
}
