// node --test scripts/gen-manifest.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { buildManifest } from "./gen-manifest.mjs";

test("実ファイルから収録済み言語を組み立てる", () => {
  const files = new Set([
    "rust/beginner.json", "rust/intermediate.json", "rust/advanced.json",
    "cpp/beginner.json", "cpp/intermediate.json", "cpp/advanced.json",
  ]);
  const m = buildManifest((p) => files.has(p));
  assert.deepEqual(m.languages.map((l) => l.slug), ["rust", "cpp"]);
  assert.deepEqual(m.languages[0].levels, ["beginner", "intermediate", "advanced"]);
});

test("レベルが欠けている言語は levels が短くなる", () => {
  const files = new Set(["java/beginner.json"]);
  const m = buildManifest((p) => files.has(p));
  const java = m.languages.find((l) => l.slug === "java");
  assert.deepEqual(java.levels, ["beginner"]);
});

test("1 ファイルも無い言語は載せない", () => {
  const m = buildManifest(() => false);
  assert.deepEqual(m.languages, []);
});

test("言語の順序は正規の並び", () => {
  const all = new Set();
  for (const l of ["javascript", "rust", "python"]) {
    for (const lv of ["beginner", "intermediate", "advanced"]) all.add(`${l}/${lv}.json`);
  }
  const m = buildManifest((p) => all.has(p));
  assert.deepEqual(m.languages.map((l) => l.slug), ["rust", "python", "javascript"]);
});
