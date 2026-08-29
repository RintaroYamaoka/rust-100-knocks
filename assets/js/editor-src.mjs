// CodeMirror 6 エディタの glue 層。wasm (Leptos) 側は window.RustKnocksEditor 経由で操作する。
// これはバンドルの「ソース」。配信物 assets/js/editor.js は次で再生成する (依存は package.json に固定):
//   npm ci && npm run build:editor
import { basicSetup } from "codemirror";
import { Compartment } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { indentUnit, StreamLanguage } from "@codemirror/language";
import { rust } from "@codemirror/lang-rust";
import { cpp } from "@codemirror/lang-cpp";
import { java } from "@codemirror/lang-java";
import { python } from "@codemirror/lang-python";
import { javascript } from "@codemirror/lang-javascript";
import { csharp as csharpStreamParser } from "@codemirror/legacy-modes/mode/clike";
import { oneDark } from "@codemirror/theme-one-dark";

// 言語モードの正本は crates/shared/src/language.rs の Language::slug()。
// キーは rust / cpp / csharp / java / python / typescript / javascript の 7 種。
// indent: 改行時の自動インデント幅 (Rust は rustfmt / Zed / VSCode に合わせて 4 スペース)。
const LANGUAGES = {
  rust: { mode: () => rust(), indent: "    " },
  cpp: { mode: () => cpp(), indent: "    " },
  // C# は公式パッケージが無いので legacy-modes の clike を StreamLanguage で使う。
  csharp: { mode: () => StreamLanguage.define(csharpStreamParser), indent: "    " },
  java: { mode: () => java(), indent: "    " },
  python: { mode: () => python(), indent: "    " },
  typescript: { mode: () => javascript({ typescript: true }), indent: "  " },
  javascript: { mode: () => javascript(), indent: "  " },
};

const DEFAULT_LANGUAGE = "rust";

// 言語切替は必ずこの Compartment の reconfigure で行う。
// エディタを再マウント (destroy → new EditorView) すると、debounce 中の下書きが
// 切替後の問題に紛れ込む・スクロール位置と履歴が飛ぶ、といった実害が出る。
const languageConf = new Compartment();

const state = {
  view: null,
  language: DEFAULT_LANGUAGE,
  onRun: null,
  onSave: null,
  onChange: null,
  changeTimer: null,
};

// 保留中の onChange debounce を捨てる。mount / setValue / setLanguage は
// 「別の問題・別の言語の内容に入れ替える」操作なので、必ず先頭でこれを呼ぶ。
// 呼ばないと切替直前 600ms 以内の編集が切替後の問題の下書きとして保存される。
function cancelPendingChange() {
  clearTimeout(state.changeTimer);
  state.changeTimer = null;
}

function notifyChange(doc) {
  clearTimeout(state.changeTimer);
  state.changeTimer = setTimeout(() => {
    if (state.onChange) state.onChange(doc);
  }, 600);
}

function languageExtensions(slug) {
  const spec = LANGUAGES[slug] || LANGUAGES[DEFAULT_LANGUAGE];
  return [spec.mode(), indentUnit.of(spec.indent)];
}

const api = {
  mount(parentId, initialCode, languageSlug) {
    cancelPendingChange();
    if (typeof languageSlug === "string" && LANGUAGES[languageSlug]) {
      state.language = languageSlug;
    }
    const parent = document.getElementById(parentId);
    if (!parent) return false;
    if (state.view) {
      state.view.destroy();
      state.view = null;
    }
    state.view = new EditorView({
      parent,
      doc: initialCode,
      extensions: [
        // 独自 keymap を basicSetup より先に置き、Mod-Enter (既定: 空行挿入) を実行に奪う
        keymap.of([
          { key: "Mod-Enter", run: () => (state.onRun && state.onRun(), true) },
          { key: "Mod-s", run: () => (state.onSave && state.onSave(), true), preventDefault: true },
          indentWithTab,
        ]),
        basicSetup,
        languageConf.of(languageExtensions(state.language)),
        oneDark,
        EditorView.updateListener.of((u) => {
          if (u.docChanged) notifyChange(u.state.doc.toString());
        }),
        EditorView.theme({
          "&": { height: "100%", fontSize: "14px" },
          ".cm-scroller": { overflow: "auto", fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace" },
        }),
      ],
    });
    return true;
  },
  // 言語モードを切り替える。slug は crates/shared の Language::slug() と同じ 7 種。
  // 未知の slug は現状維持して false を返す (例外は投げない)。
  setLanguage(slug) {
    cancelPendingChange();
    if (typeof slug !== "string" || !LANGUAGES[slug]) return false;
    if (state.language === slug) return true;
    state.language = slug;
    const v = state.view;
    // 未マウントでも state.language は更新しておく (次の mount がこの言語で立ち上がる)
    if (v) v.dispatch({ effects: languageConf.reconfigure(languageExtensions(slug)) });
    return true;
  },
  getLanguage() {
    return state.language;
  },
  setValue(code) {
    cancelPendingChange();
    const v = state.view;
    if (!v) return;
    v.dispatch({ changes: { from: 0, to: v.state.doc.length, insert: code } });
  },
  getValue() {
    return state.view ? state.view.state.doc.toString() : "";
  },
  focus() {
    if (state.view) state.view.focus();
  },
  setOnRun(cb) { state.onRun = cb; },
  setOnSave(cb) { state.onSave = cb; },
  setOnChange(cb) { state.onChange = cb; },
};

window.RustKnocksEditor = api;
