// ビルド済みバンドル (assets/js/editor.js) の公開契約テスト。
// 実行: node --test tests/editor-src.test.mjs  (バンドル再生成後に叩く)
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const bundlePath = fileURLToPath(new URL("../assets/js/editor.js", import.meta.url));

test("bundle exposes the RustKnocksEditor API on window", async () => {
  globalThis.window = {};
  const src = await readFile(bundlePath, "utf8");
  // IIFE バンドル: 評価すると window.RustKnocksEditor が生える (mount までは DOM 不要)
  new Function(src)();
  const api = globalThis.window.RustKnocksEditor;
  assert.ok(api, "window.RustKnocksEditor が定義されていない");
  for (const method of ["mount", "getValue", "setValue", "focus", "setOnRun", "setOnSave", "setOnChange"]) {
    assert.equal(typeof api[method], "function", `${method} がない`);
  }
  assert.equal(api.getValue(), "", "未マウント時の getValue は空文字を返す");
});
