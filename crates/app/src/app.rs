//! ルートコンポーネント: 状態の持ち主と UI 配線。純ロジックは各モジュール / shared 側に置く。

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::language::Language;
use shared::playground::{classify, ExecuteRequest, Outcome};
use shared::problem::{compose_submission, Level, Problem};
use shared::progress::{
    passed_count, progress_key, saved_code_of, status_of, ProblemStatus, ProgressEntry, ProgressMap,
    StatusFilter,
};

use crate::console::{ConsolePane, RunState};
use crate::lang::{initial_language, languages_with_full_data, resolve_selection, selector_languages};
use crate::layout::{load_layout, LayoutSizes, SplitTarget};
use crate::list::Sidebar;
use crate::problem_view::ProblemPane;
use crate::splitter::Splitter;
use crate::{api, editor, storage};

/// 判定結果による進捗ステータス遷移。一度 Passed になったら失敗しても降格しない。
pub fn next_status(current: ProblemStatus, outcome: Outcome) -> ProblemStatus {
    match (current, outcome) {
        (_, Outcome::Passed) => ProblemStatus::Passed,
        (ProblemStatus::Passed, _) => ProblemStatus::Passed,
        _ => ProblemStatus::Attempted,
    }
}

/// エディタに載せるコード。下書きがあればそれ、無ければ starter_code。
///
/// 下書きの取り出しは `saved_code_of` (= `progress_key`) 経由に固定してある。
/// 素の `id` で引くと、`b001` を共有する 7 言語の下書きが混ざる。
pub fn code_for(p: &Problem, map: &ProgressMap) -> String {
    saved_code_of(map, p)
        .map(String::from)
        .unwrap_or_else(|| p.starter_code.clone())
}

/// 選択中の言語・レベルの問題データを覚えておくキー。
type CacheKey = (Language, Level);

#[component]
pub fn App() -> impl IntoView {
    let language = RwSignal::new(initial_language(storage::load_language().as_deref()));
    let level = RwSignal::new(Level::Beginner);
    let cache: RwSignal<HashMap<CacheKey, Vec<Problem>>> = RwSignal::new(HashMap::new());
    let load_error: RwSignal<Option<String>> = RwSignal::new(None);
    let selected: RwSignal<Option<Problem>> = RwSignal::new(None);
    let filter = RwSignal::new(StatusFilter::All);
    let query = RwSignal::new(String::new());
    // 旧フラットキー (Rust 専用時代の `b001`) はここで `rust/b001` に移行される
    let progress: RwSignal<ProgressMap> = RwSignal::new(storage::load_progress_migrated());
    let run_state: RwSignal<RunState> = RwSignal::new(RunState::Idle);
    let revealed: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let editor_ready = RwSignal::new(false);
    // 進捗を保存できなかったか (localStorage のクォータ超過など)。
    // 握り潰すと、画面上は成功と区別が付かないまま進捗が消える
    let storage_failed = RwSignal::new(false);
    let mount_started = RwSignal::new(false);
    // データが 3 レベル揃っていることを確認できた言語 (確認前は空)
    let available: RwSignal<Vec<Language>> = RwSignal::new(Vec::new());
    let probe_started = RwSignal::new(false);

    // 起動時に一度だけ、各言語のデータが 3 レベル揃っているかを確認する。
    // 揃っていない言語をセレクタに出すと、選んだ瞬間に空の一覧を見せることになる。
    Effect::new(move |_| {
        if probe_started.get_untracked() {
            return;
        }
        probe_started.set(true);
        spawn_local(async move {
            let mut found = HashSet::new();
            for lang in Language::ALL {
                for lv in Level::ALL {
                    if !api::problems_exist(lang, lv).await {
                        // 1 レベルでも欠けたらその言語は出さないので、残りは問わない
                        break;
                    }
                    found.insert((lang, lv));
                }
            }
            let langs = languages_with_full_data(&found);
            // データの無い言語が選ばれていたら有効な言語へ寄せる (確認が
            // 効かなかった = 空のときは動かさない)
            let resolved = resolve_selection(language.get_untracked(), &langs);
            available.set(langs);
            if resolved != language.get_untracked() {
                language.set(resolved);
            }
        });
    });

    // 言語 / レベル切替時に問題データをフェッチ (キャッシュ済みなら何もしない)
    Effect::new(move |_| {
        let key = (language.get(), level.get());
        if cache.with_untracked(|c| c.contains_key(&key)) {
            load_error.set(None);
            return;
        }
        spawn_local(async move {
            match api::fetch_problems(key.0, key.1).await {
                Ok(ps) => {
                    load_error.set(None);
                    cache.update(|c| {
                        c.insert(key, ps);
                    });
                }
                Err(e) => load_error.set(Some(e)),
            }
        });
    });

    let problems =
        Memo::new(move |_| cache.with(|c| c.get(&(language.get(), level.get())).cloned().unwrap_or_default()));
    // 選択中の言語のデータが実際に読めているか (セレクタのフォールバック判定に使う)
    let current_loaded = Memo::new(move |_| {
        let lang = language.get();
        cache.with(|c| c.keys().any(|(l, _)| *l == lang))
    });
    let visible_languages = Memo::new(move |_| {
        selector_languages(&available.get(), language.get(), current_loaded.get())
    });

    // 下書き保存 (エディタ内容が starter と同じなら保存しない)
    let save_draft_code = move |code: String| {
        let Some(p) = selected.get_untracked() else {
            return;
        };
        progress.update(|m| {
            let e = m.entry(progress_key(&p)).or_insert(ProgressEntry {
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
        if !progress.with_untracked(storage::save_progress) {
            storage_failed.set(true);
        }
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
        let lang = p.language;
        selected.set(Some(p));
        if editor_ready.get_untracked() {
            // 言語モードを先に切り替えてから中身を入れる。どちらも再マウントせず、
            // JS 側で debounce 中の変更通知を破棄する契約 (前の言語のコードが
            // 切替後の下書きを上書きしないようにするため)。
            editor::set_language(lang);
            editor::set_value(&code);
            editor::focus();
        }
    });

    // 問題データ到着時 / 言語・レベル切替時:
    // 未選択か、いま表示している言語・レベルと違う問題が残っていたら先頭を自動選択
    Effect::new(move |_| {
        let ps = problems.get();
        let (lang, lv) = (language.get(), level.get());
        let needs_select = selected.with_untracked(|s| {
            s.as_ref()
                .is_none_or(|p| p.level != lv || p.language != lang)
        });
        if needs_select {
            if let Some(first) = ps.first() {
                select_problem.run(first.clone());
            }
        }
    });

    // 言語の選択を localStorage に残す (次回起動時に復元する)
    Effect::new(move |prev: Option<Language>| {
        let lang = language.get();
        if prev.is_some_and(|p| p == lang) {
            return lang;
        }
        storage::save_language(lang);
        lang
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
        let submission = compose_submission(p.language, &code, &p.hidden_tests);
        let req = ExecuteRequest::judge(p.language, &submission);
        spawn_local(async move {
            match api::execute(&req).await {
                Ok(resp) => {
                    let outcome = classify(&resp);
                    progress.update(|m| {
                        let cur = status_of(m, &p);
                        let e = m.entry(progress_key(&p)).or_insert(ProgressEntry {
                            status: ProblemStatus::Unanswered,
                            saved_code: None,
                            updated_at_ms: 0.0,
                        });
                        e.status = next_status(cur, outcome);
                        e.updated_at_ms = storage::now_ms();
                    });
                    if !progress.with_untracked(storage::save_progress) {
            storage_failed.set(true);
        }
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
            let sel = selected.get_untracked();
            let lang = sel.as_ref().map_or_else(|| language.get_untracked(), |p| p.language);
            editor::set_language(lang);
            let code = sel
                .as_ref()
                .map(|p| progress.with_untracked(|m| code_for(p, m)));
            editor::set_value(&code.unwrap_or_else(|| {
                format!("{} 左の一覧から問題を選んでください", lang.line_comment())
            }));
            editor::on_run(run);
            editor::on_save(move || save_draft());
            editor::on_change(save_draft_code);
            editor::focus();
        });
        let initial_lang = selected
            .get_untracked()
            .map_or_else(|| language.get_untracked(), |p| p.language);
        editor::mount_retrying("editor-host", String::new(), initial_lang, on_mounted);
    });

    let layout: RwSignal<LayoutSizes> = RwSignal::new(load_layout());
    // ドラッグ中は iframe/エディタがポインタイベントを奪わないよう .main に resizing を付ける
    let layout_dragging = RwSignal::new(false);
    provide_context(layout_dragging);

    // 一覧の選択表示も言語込みのキーで突き合わせる (切替直後に別言語の同 id を
    // 選択中として光らせないため)
    let selected_id = Signal::derive(move || selected.with(|s| s.as_ref().map(progress_key)));
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
                    <span class="mark" inner_html=crate::GEAR_SVG></span>
                    <span><span class="knocks">"100本ノック"</span></span>
                </div>
                <select
                    class="lang-select"
                    title="練習する言語"
                    prop:value=move || {
                        // 一覧にも依存させる: option が後から (存在確認の完了後に) 増えたとき、
                        // ここを再評価しないと select の表示が先頭の言語にずれる
                        let _ = visible_languages.get();
                        language.get().slug().to_string()
                    }
                    on:change=move |ev| {
                        if let Some(l) = Language::from_slug(&event_target_value(&ev)) {
                            language.set(l);
                        }
                    }
                >
                    <For
                        each=move || visible_languages.get()
                        key=|l| l.slug()
                        children=move |l: Language| {
                            view! {
                                <option value=l.slug() selected=move || language.get() == l>
                                    {l.label()}
                                </option>
                            }
                        }
                    />
                </select>
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
                {move || {
                    storage_failed
                        .get()
                        .then(|| {
                            view! {
                                <span class="storage-warning" title="ブラウザの保存領域がいっぱいの可能性があります">
                                    "⚠ 進捗を保存できませんでした"
                                </span>
                            }
                        })
                }}
                <div class="header-progress">
                    <span>{move || format!("{} / {} 問クリア", passed_in_level.get(), problems.with(|p| p.len()))}</span>
                    <div class="progress-track">
                        <div class="progress-fill" style:width=move || format!("{}%", progress_pct())></div>
                    </div>
                </div>
            </header>
            <div class="main" class:resizing=move || layout_dragging.get() style=move || layout.with(|l| l.css_vars())>
                <Sidebar
                    problems=problems
                    progress=progress
                    filter=filter
                    query=query
                    selected_id=selected_id
                    load_error=load_error
                    on_select=select_problem
                />
                <Splitter target=SplitTarget::Sidebar sizes=layout/>
                <ProblemPane problem=selected.into() progress=progress revealed=revealed/>
                <Splitter target=SplitTarget::Problem sizes=layout/>
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
                    <Splitter target=SplitTarget::Console sizes=layout/>
                    <ConsolePane
                        state=Signal::derive(move || run_state.get())
                        language=Signal::derive(move || {
                            // 表示中の問題の言語が正 (セレクタ操作の直後でも実行結果と食い違わない)
                            selected.with(|s| s.as_ref().map_or_else(|| language.get(), |p| p.language))
                        })
                        on_next=Callback::new(move |()| nav(1))
                    />
                </section>
            </div>
        </div>
    }
}
