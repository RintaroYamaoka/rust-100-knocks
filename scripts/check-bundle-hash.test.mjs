// node --test scripts/check-bundle-hash.test.mjs
//
// index.html の ?v= が assets/js/editor.js の実際の md5 と一致することを検査する。
// ずれると、旧バンドルをキャッシュに持つ再訪者が「新 wasm × 旧 glue」になり、
// setLanguage が無いので **7 言語すべてが Rust のハイライトで描かれる**。
// 人手で更新する運用にすると必ず忘れるので、テストで固定する。
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";

const BUNDLE = "assets/js/editor.js";
const HTML = "index.html";

function bundleHash() {
  return createHash("md5").update(readFileSync(BUNDLE)).digest("hex").slice(0, 8);
}

function declaredHash() {
  const html = readFileSync(HTML, "utf8");
  const m = html.match(/editor\.js\?v=([0-9a-f]+)/);
  return m ? m[1] : null;
}

test("index.html の ?v= がバンドルの md5 と一致する", () => {
  const declared = declaredHash();
  assert.ok(declared, `${HTML} に editor.js?v=<hash> が無い`);
  assert.equal(
    declared,
    bundleHash(),
    `バンドルを再生成したら index.html の ?v= も更新すること ` +
      `(npm run build:editor が正しい値を表示する)`
  );
});

test("配信するバンドルが setLanguage を持っている", () => {
  // ?v= が合っていても中身が古ければ意味がない
  const js = readFileSync(BUNDLE, "utf8");
  assert.ok(js.includes("setLanguage"), "バンドルに setLanguage が無い");
});
