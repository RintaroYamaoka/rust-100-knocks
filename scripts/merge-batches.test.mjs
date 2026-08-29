// node --test scripts/merge-batches.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { mergeBatches, MergeError } from "./merge-batches.mjs";

const LEVEL = { slug: "beginner", prefix: "b" };

function problem(n, over = {}) {
  const id = `${LEVEL.prefix}${String(n).padStart(3, "0")}`;
  return {
    id,
    language: "cpp",
    level: "beginner",
    title: `問題 ${id}`,
    description_md: "せつめい".repeat(30),
    starter_code: `int f${n}() { return 0; }`,
    hidden_tests: "test result: ok / test result: FAILED / FAILED: x",
    answer_code: `int f${n}() { return ${n}; }`,
    explanation_md: "かいせつ",
    hints: [],
    tags: ["t"],
    ...over,
  };
}

function batches(counts) {
  // counts: [20, 20, ...] → 連番の問題をバッチに割る
  let n = 0;
  return counts.map((c) => Array.from({ length: c }, () => problem(++n)));
}

test("連続した5バッチが100問に統合される", () => {
  const out = mergeBatches(batches([20, 20, 20, 20, 20]), "cpp", LEVEL, 100);
  assert.equal(out.length, 100);
  assert.equal(out[0].id, "b001");
  assert.equal(out[99].id, "b100");
});

test("バッチの順序が入れ替わっていても id 順に並ぶ", () => {
  const [a, b] = batches([2, 2]);
  const out = mergeBatches([b, a], "cpp", LEVEL, 4);
  assert.deepEqual(out.map((p) => p.id), ["b001", "b002", "b003", "b004"]);
});

test("id が重複したら失敗する", () => {
  const dup = [[problem(1)], [problem(1)]];
  assert.throws(() => mergeBatches(dup, "cpp", LEVEL, 2), MergeError);
});

test("id に欠番があれば失敗する", () => {
  // b001, b003 → b002 が無い
  const gap = [[problem(1), problem(3)]];
  assert.throws(() => mergeBatches(gap, "cpp", LEVEL, 2), /欠番|連番/);
});

test("期待件数と一致しなければ失敗する", () => {
  assert.throws(() => mergeBatches(batches([20]), "cpp", LEVEL, 100), /件数/);
});

test("language がディレクトリと食い違えば失敗する", () => {
  const wrong = [[problem(1, { language: "java" })]];
  assert.throws(() => mergeBatches(wrong, "cpp", LEVEL, 1), /language/);
});

test("level がファイルと食い違えば失敗する", () => {
  const wrong = [[problem(1, { level: "advanced" })]];
  assert.throws(() => mergeBatches(wrong, "cpp", LEVEL, 1), /level/);
});

test("title の重複を検出する", () => {
  // 使い回しの検出。1問をコピーした100問は他の検査を全部通ってしまう
  const dup = [[problem(1, { title: "同じ" }), problem(2, { title: "同じ" })]];
  assert.throws(() => mergeBatches(dup, "cpp", LEVEL, 2), /title/);
});

test("answer_code の重複を検出する", () => {
  const dup = [[problem(1, { answer_code: "same" }), problem(2, { answer_code: "same" })]];
  assert.throws(() => mergeBatches(dup, "cpp", LEVEL, 2), /answer_code/);
});

test("バッチが空でも黙って成功しない", () => {
  assert.throws(() => mergeBatches([], "cpp", LEVEL, 100), /件数/);
});
