// CodeMirror 6 エディタの glue 層。wasm (Leptos) 側は window.RustKnocksEditor 経由で操作する。
// これはバンドルの「ソース」。配信物 assets/js/editor.js は esbuild で生成する:
//   npx esbuild assets/js/editor-src.mjs --bundle --format=iife --minify --outfile=assets/js/editor.js
import { basicSetup } from "codemirror";
import { EditorView, keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { indentUnit } from "@codemirror/language";
import { rust } from "@codemirror/lang-rust";
import { oneDark } from "@codemirror/theme-one-dark";

const state = { view: null, onRun: null, onSave: null, onChange: null, changeTimer: null };

function notifyChange(doc) {
  clearTimeout(state.changeTimer);
  state.changeTimer = setTimeout(() => {
    if (state.onChange) state.onChange(doc);
  }, 600);
}

const api = {
  mount(parentId, initialCode) {
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
        // Rust 標準 (rustfmt / Zed / VSCode) に合わせて改行時の自動インデントを 4 スペースに
        indentUnit.of("    "),
        rust(),
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
  setValue(code) {
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
