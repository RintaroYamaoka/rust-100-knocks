//! ルートコンポーネント: 状態の持ち主と UI 配線。純ロジックは各モジュール / shared 側に置く。

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::playground::{classify, ExecuteRequest, Outcome};
use shared::problem::{compose_submission, Level, Problem};
use shared::progress::{
    passed_count, status_of, ProblemStatus, ProgressEntry, ProgressMap, StatusFilter,
};

use crate::console::{ConsolePane, RunState};
use crate::list::Sidebar;
use crate::problem_view::ProblemPane;
use crate::{api, editor, storage};

/// 判定結果による進捗ステータス遷移。一度 Passed になったら失敗しても降格しない。
pub fn next_status(current: ProblemStatus, outcome: Outcome) -> ProblemStatus {
    match (current, outcome) {
        (_, Outcome::Passed) => ProblemStatus::Passed,
        (ProblemStatus::Passed, _) => ProblemStatus::Passed,
        _ => ProblemStatus::Attempted,
    }
}

fn code_for(p: &Problem, map: &ProgressMap) -> String {
    map.get(&p.id)
        .and_then(|e| e.saved_code.clone())
        .unwrap_or_else(|| p.starter_code.clone())
}

#[component]
pub fn App() -> impl IntoView {
    let level = RwSignal::new(Level::Beginner);
    let cache: RwSignal<HashMap<Level, Vec<Problem>>> = RwSignal::new(HashMap::new());
    let load_error: RwSignal<Option<String>> = RwSignal::new(None);
    let selected: RwSignal<Option<Problem>> = RwSignal::new(None);
    let filter = RwSignal::new(StatusFilter::All);
    let query = RwSignal::new(String::new());
    let progress: RwSignal<ProgressMap> = RwSignal::new(storage::load_progress());
    let run_state: RwSignal<RunState> = RwSignal::new(RunState::Idle);
    let revealed: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let editor_ready = RwSignal::new(false);
    let mount_started = RwSignal::new(false);

    // レベル切替時に問題データをフェッチ (キャッシュ済みなら何もしない)
    Effect::new(move |_| {
        let lv = level.get();
        if cache.with_untracked(|c| c.contains_key(&lv)) {
            return;
        }
        spawn_local(async move {
            match api::fetch_problems(lv).await {
                Ok(ps) => {
                    load_error.set(None);
                    cache.update(|c| {
                        c.insert(lv, ps);
                    });
                }
                Err(e) => load_error.set(Some(e)),
            }
        });
    });

    let problems = Memo::new(move |_| cache.with(|c| c.get(&level.get()).cloned().unwrap_or_default()));

    // 下書き保存 (エディタ内容が starter と同じなら保存しない)
    let save_draft_code = move |code: String| {
        let Some(p) = selected.get_untracked() else {
            return;
        };
        progress.update(|m| {
            let e = m.entry(p.id.clone()).or_insert(ProgressEntry {
                status: ProblemStatus::Unanswered,
                saved_code: None,
                updated_at_ms: 0.0,
            });
            e.saved_code = if code.trim().is_empty() || code == p.starter_code {
                None
            } else {
                Some(code)
            };
            e.updated_at_ms = storage::now_ms();
        });
        progress.with_untracked(storage::save_progress);
    };
    let save_draft = move || {
        if editor_ready.get_untracked() {
            save_draft_code(editor::get_value());
        }
    };

    let select_problem = Callback::new(move |p: Problem| {
        save_draft();
        run_state.set(RunState::Idle);
        let code = progress.with_untracked(|m| code_for(&p, m));
        selected.set(Some(p));
        if editor_ready.get_untracked() {
            editor::set_value(&code);
            editor::focus();
        }
    });

    // 問題データ到着時 / レベル切替時: 未選択か別レベルの問題が残っていたら先頭を自動選択
    Effect::new(move |_| {
        let ps = problems.get();
        let needs_select = selected.with_untracked(|s| {
            s.as_ref().is_none_or(|p| p.level != level.get_untracked())
        });
        if needs_select {
            if let Some(first) = ps.first() {
                select_problem.run(first.clone());
            }
        }
    });

    // 実行して判定
    let run = move || {
        let Some(p) = selected.get_untracked() else {
            return;
        };
        if run_state.with_untracked(|s| matches!(s, RunState::Running)) || !editor_ready.get_untracked()
        {
            return;
        }
        let code = editor::get_value();
        save_draft_code(code.clone());
        run_state.set(RunState::Running);
        let submission = compose_submission(&code, &p.hidden_tests);
        let req = ExecuteRequest::judge(&submission);
        let pid = p.id.clone();
        spawn_local(async move {
            match api::execute(&req).await {
                Ok(resp) => {
                    let outcome = classify(&resp);
                    progress.update(|m| {
                        let cur = status_of(m, &pid);
                        let e = m.entry(pid.clone()).or_insert(ProgressEntry {
                            status: ProblemStatus::Unanswered,
                            saved_code: None,
                            updated_at_ms: 0.0,
                        });
                        e.status = next_status(cur, outcome);
                        e.updated_at_ms = storage::now_ms();
                    });
                    progress.with_untracked(storage::save_progress);
                    run_state.set(RunState::Done { resp, outcome });
                }
                Err(e) => run_state.set(RunState::Failed(e)),
            }
        });
    };

    let reset_code = move |_| {
        if let Some(p) = selected.get_untracked() {
            if editor_ready.get_untracked() {
                editor::set_value(&p.starter_code);
                save_draft_code(p.starter_code.clone());
            }
        }
    };

    let nav = move |delta: i64| {
        let ps = problems.get_untracked();
        let Some(cur) = selected.with_untracked(|s| s.as_ref().map(|p| p.id.clone())) else {
            return;
        };
        if let Some(idx) = ps.iter().position(|p| p.id == cur) {
            let target = idx as i64 + delta;
            if target >= 0 && (target as usize) < ps.len() {
                select_problem.run(ps[target as usize].clone());
            }
        }
    };

    // エディタは常設 DOM に一度だけマウントし、glue script のロードを待つ
    Effect::new(move |_| {
        if mount_started.get_untracked() {
            return;
        }
        mount_started.set(true);
        let on_mounted: Rc<dyn Fn()> = Rc::new(move || {
            editor_ready.set(true);
            let code = selected
                .with_untracked(|s| s.as_ref().map(|p| progress.with_untracked(|m| code_for(p, m))));
            editor::set_value(&code.unwrap_or_else(|| "// 左の一覧から問題を選んでください".to_string()));
            editor::on_run(run);
            editor::on_save(move || save_draft());
            editor::on_change(save_draft_code);
            editor::focus();
        });
        editor::mount_retrying("editor-host", String::new(), on_mounted);
    });

    let selected_id = Signal::derive(move || selected.with(|s| s.as_ref().map(|p| p.id.clone())));
    let passed_in_level = Memo::new(move |_| {
        let ps = problems.get();
        progress.with(|m| passed_count(&ps, m))
    });
    let progress_pct = move || {
        let total = problems.with(|p| p.len());
        if total == 0 {
            0.0
        } else {
            passed_in_level.get() as f64 / total as f64 * 100.0
        }
    };

    view! {
        <div class="app">
            <header class="header">
                <div class="brand">
                    <span class="mark">"⚙️"</span>
                    <span>"Rust " <span class="knocks">"100本ノック"</span></span>
                </div>
                <div class="level-tabs">
                    {Level::ALL
                        .iter()
                        .map(|lv| {
                            let lv = *lv;
                            view! {
                                <button
                                    class="level-tab"
                                    class:active=move || level.get() == lv
                                    on:click=move |_| level.set(lv)
                                >
                                    {lv.label_ja()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="header-progress">
                    <span>{move || format!("{} / {} 問クリア", passed_in_level.get(), problems.with(|p| p.len()))}</span>
                    <div class="progress-track">
                        <div class="progress-fill" style:width=move || format!("{}%", progress_pct())></div>
                    </div>
                </div>
            </header>
            <div class="main">
                <Sidebar
                    problems=problems
                    progress=progress
                    filter=filter
                    query=query
                    selected_id=selected_id
                    load_error=load_error
                    on_select=select_problem
                />
                <ProblemPane problem=selected.into() progress=progress revealed=revealed/>
                <section class="workbench">
                    <div class="workbench-toolbar">
                        <button
                            class="run-btn"
                            prop:disabled=move || {
                                matches!(run_state.get(), RunState::Running) || selected.with(|s| s.is_none())
                            }
                            on:click=move |_| run()
                        >
                            {move || {
                                if matches!(run_state.get(), RunState::Running) {
                                    view! { <span class="spinner"></span> }.into_any()
                                } else {
                                    view! { <span>"▶"</span> }.into_any()
                                }
                            }}
                            "実行して判定"
                        </button>
                        <span class="toolbar-hint">"Ctrl+Enter"</span>
                        <div class="toolbar-spacer"></div>
                        <div class="nav-btns">
                            <button class="nav-btn" on:click=move |_| nav(-1)>"← 前の問題"</button>
                            <button class="nav-btn" on:click=move |_| nav(1)>"次の問題 →"</button>
                        </div>
                        <button class="reset-btn" on:click=reset_code title="コードを初期状態に戻す">"リセット"</button>
                    </div>
                    <div class="editor-host" id="editor-host"></div>
                    <ConsolePane
                        state=Signal::derive(move || run_state.get())
                        on_next=Callback::new(move |()| nav(1))
                    />
                </section>
            </div>
        </div>
    }
}
