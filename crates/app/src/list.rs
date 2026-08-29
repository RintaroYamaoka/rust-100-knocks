//! 左ペイン: 状態フィルタ・検索・問題一覧。絞り込みロジックは shared::progress に委譲。

use leptos::prelude::*;
use shared::problem::Problem;
use shared::progress::{
    filter_problems, progress_key, status_of, ProblemStatus, ProgressMap, StatusFilter,
};

const FILTERS: [(StatusFilter, &str); 4] = [
    (StatusFilter::All, "すべて"),
    (StatusFilter::OnlyUnanswered, "未回答"),
    (StatusFilter::OnlyAttempted, "未正解"),
    (StatusFilter::OnlyPassed, "正解済"),
];

fn status_badge(status: ProblemStatus) -> impl IntoView {
    let (class, mark) = match status {
        ProblemStatus::Passed => ("status-badge passed", "✓"),
        ProblemStatus::Attempted => ("status-badge attempted", "△"),
        ProblemStatus::Unanswered => ("status-badge unanswered", "・"),
    };
    view! { <span class=class>{mark}</span> }
}

#[component]
pub fn Sidebar(
    problems: Memo<Vec<Problem>>,
    progress: RwSignal<ProgressMap>,
    filter: RwSignal<StatusFilter>,
    query: RwSignal<String>,
    selected_id: Signal<Option<String>>,
    load_error: RwSignal<Option<String>>,
    on_select: Callback<Problem>,
) -> impl IntoView {
    let filtered = Memo::new(move |_| {
        let ps = problems.get();
        let q = query.get();
        progress.with(|m| {
            filter_problems(&ps, m, filter.get(), &q)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
    });

    view! {
        <aside class="sidebar">
            <div class="sidebar-controls">
                <input
                    class="search-input"
                    type="search"
                    placeholder="検索 (id / タイトル / タグ)"
                    prop:value=move || query.get()
                    on:input=move |ev| query.set(event_target_value(&ev))
                />
                <div class="status-filters">
                    {FILTERS
                        .into_iter()
                        .map(|(f, label)| {
                            view! {
                                <button
                                    class="status-filter"
                                    class:active=move || filter.get() == f
                                    on:click=move |_| filter.set(f)
                                >
                                    {label}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <div class="problem-list">
                {move || {
                    load_error
                        .get()
                        .map(|e| view! { <div class="list-empty">{e}</div> })
                }}
                <For
                    each=move || filtered.get()
                    // 言語をまたいで一意なキーで引く。素の id だと `b001` が全言語に
                    // あるので、言語を切り替えても For が行を再利用してしまい、
                    // 前の言語のタイトルが残る
                    key=|p| progress_key(p)
                    children=move |p: Problem| {
                        let key = progress_key(&p);
                        // 進捗は必ず Problem 経由で引く (素の id では言語をまたいで衝突する)
                        let p_for_status = p.clone();
                        let pid_label = p.id.clone();
                        let title = p.title.clone();
                        view! {
                            <button
                                class="problem-item"
                                class:selected=move || selected_id.get().as_deref() == Some(key.as_str())
                                on:click=move |_| on_select.run(p.clone())
                            >
                                {move || progress.with(|m| status_badge(status_of(m, &p_for_status)))}
                                <span class="pid">{pid_label}</span>
                                <span class="ptitle">{title}</span>
                            </button>
                        }
                    }
                />
                {move || {
                    (filtered.get().is_empty() && load_error.get().is_none())
                        .then(|| view! { <div class="list-empty">"条件に合う問題がありません"</div> })
                }}
            </div>
        </aside>
    }
}
