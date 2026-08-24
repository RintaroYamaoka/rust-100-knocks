//! 中央ペイン: 問題文・ヒント (段階開示)・解答と解説 (ネタバレゲートつき)。

use std::collections::HashSet;

use leptos::prelude::*;
use shared::problem::Problem;
use shared::progress::{status_of, ProblemStatus, ProgressMap};

use crate::md::render_markdown;

/// 解答タブを表示してよいか。正解済みなら常に可、未正解は明示的な開示操作が必要。
pub fn answer_visible(status: ProblemStatus, revealed: bool) -> bool {
    status == ProblemStatus::Passed || revealed
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PaneTab {
    Statement,
    Solution,
}

#[component]
pub fn ProblemPane(
    problem: Signal<Option<Problem>>,
    progress: RwSignal<ProgressMap>,
    revealed: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let tab = RwSignal::new(PaneTab::Statement);
    let hints_shown = RwSignal::new(0usize);

    // 問題が切り替わったらタブとヒント開示をリセット
    Effect::new(move |prev: Option<Option<String>>| {
        let id = problem.with(|p| p.as_ref().map(|p| p.id.clone()));
        if let Some(prev_id) = prev {
            if prev_id != id {
                tab.set(PaneTab::Statement);
                hints_shown.set(0);
            }
        }
        id
    });

    let statement_view = move |p: Problem| {
        let hints = p.hints.clone();
        view! {
            <div class="md" inner_html=render_markdown(&p.description_md)></div>
            {(!hints.is_empty())
                .then(|| {
                    view! {
                        <div class="hint-block">
                            {hints
                                .into_iter()
                                .enumerate()
                                .map(|(i, hint)| {
                                    view! {
                                        {move || {
                                            let shown = hints_shown.get();
                                            if i < shown {
                                                view! { <div class="hint-content md" inner_html=render_markdown(&hint)></div> }
                                                    .into_any()
                                            } else if i == shown {
                                                view! {
                                                    <button
                                                        class="hint-toggle"
                                                        on:click=move |_| hints_shown.update(|n| *n += 1)
                                                    >
                                                        "💡 ヒント " {i + 1} " を見る"
                                                    </button>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })}
        }
    };

    let solution_view = move |p: Problem| {
        let pid = p.id.clone();
        let status = progress.with(|m| status_of(m, &pid));
        let is_revealed = revealed.with(|r| r.contains(&pid));
        if answer_visible(status, is_revealed) {
            view! {
                <div class="answer-section">
                    <h2>"回答例"</h2>
                    <div class="md">
                        <pre><code class="language-rust">{p.answer_code.clone()}</code></pre>
                    </div>
                    <h2>"解説"</h2>
                    <div class="md" inner_html=render_markdown(&p.explanation_md)></div>
                </div>
            }
            .into_any()
        } else {
            let pid_reveal = p.id.clone();
            view! {
                <div class="answer-gate">
                    <div class="gate-note">"まだ正解していません。自力で解いてから見るのがおすすめですが、行き詰まったら開いてもOK。"</div>
                    <button
                        class="reveal-btn"
                        on:click=move |_| {
                            revealed.update(|r| {
                                r.insert(pid_reveal.clone());
                            });
                        }
                    >
                        "回答と解説を表示する"
                    </button>
                </div>
            }
            .into_any()
        }
    };

    view! {
        <section class="problem-pane">
            {move || match problem.get() {
                None => view! {
                    <div class="empty-pane">
                        <div class="empty-inner">
                            <div class="crab">"🦀"</div>
                            <div>"左の一覧から問題を選んでください"</div>
                        </div>
                    </div>
                }
                .into_any(),
                Some(p) => {
                    let tags = p.tags.clone();
                    let level_label = p.level.label_ja();
                    let pid = p.id.clone();
                    let title = p.title.clone();
                    let p_statement = p.clone();
                    let p_solution = p.clone();
                    view! {
                        <div class="pane-tabs">
                            <button
                                class="pane-tab"
                                class:active=move || tab.get() == PaneTab::Statement
                                on:click=move |_| tab.set(PaneTab::Statement)
                            >
                                "問題"
                            </button>
                            <button
                                class="pane-tab"
                                class:active=move || tab.get() == PaneTab::Solution
                                on:click=move |_| tab.set(PaneTab::Solution)
                            >
                                "回答と解説"
                            </button>
                        </div>
                        <div class="problem-body">
                            <div class="problem-heading">
                                <span class="pid">{pid}</span>
                                <h1>{title}</h1>
                            </div>
                            <div class="problem-tags">
                                <span class="tag">{level_label}</span>
                                {tags.into_iter().map(|t| view! { <span class="tag">{t}</span> }).collect_view()}
                            </div>
                            {move || match tab.get() {
                                PaneTab::Statement => statement_view(p_statement.clone()).into_any(),
                                PaneTab::Solution => solution_view(p_solution.clone()).into_any(),
                            }}
                        </div>
                    }
                    .into_any()
                }
            }}
        </section>
    }
}
