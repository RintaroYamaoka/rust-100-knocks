// 生成バッチを data/problems/<言語>/<難易度>.json に統合する。
//
// usage: node scripts/merge-batches.mjs <lang> [level]
//        node scripts/merge-batches.mjs --all
//        (--clean を付けると、統合に成功したバッチディレクトリを削除する)
//
// バッチは data/problems/<lang>/_batches/<level>-NN.json に置かれている前提。
// 統合時に「連番・重複なし・件数一致・language/level 整合・使い回しなし」を検査し、
// 1 つでも崩れていたら**書き出さずに失敗する** (壊れたデータを収録しないため)。

import { readFileSync, writeFileSync, readdirSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";

export class MergeError extends Error {}

const LEVELS = [
  { slug: "beginner", prefix: "b" },
  { slug: "intermediate", prefix: "i" },
  { slug: "advanced", prefix: "a" },
];

const LANGS = ["rust", "cpp", "csharp", "java", "python", "typescript", "javascript"];

/**
 * バッチ配列を 1 本のリストに統合して検査する。ファイル I/O はしない (テスト可能にするため)。
 * @param {object[][]} batches 各バッチの問題配列
 * @param {string} lang ディレクトリ名 (= language の正本)
 * @param {{slug: string, prefix: string}} level
 * @param {number} expected 期待件数
 */
export function mergeBatches(batches, lang, level, expected) {
  const byId = new Map();
  for (const batch of batches) {
    for (const p of batch) {
      if (byId.has(p.id)) {
        throw new MergeError(`id が重複しています: ${p.id}`);
      }
      byId.set(p.id, p);
    }
  }

  const out = [...byId.values()].sort((a, b) => a.id.localeCompare(b.id));

  if (out.length !== expected) {
    throw new MergeError(`件数が期待と一致しません (期待 ${expected} / 実際 ${out.length})`);
  }

  const titles = new Map();
  const answers = new Map();
  out.forEach((p, i) => {
    const want = `${level.prefix}${String(i + 1).padStart(3, "0")}`;
    if (p.id !== want) {
      throw new MergeError(`id の連番が崩れています (欠番の疑い): ${want} が来るべき位置に ${p.id}`);
    }
    if (p.language !== lang) {
      throw new MergeError(`[${p.id}] language が "${p.language}" ですがディレクトリは "${lang}" です`);
    }
    if (p.level !== level.slug) {
      throw new MergeError(`[${p.id}] level が "${p.level}" ですがファイルは "${level.slug}" です`);
    }
    const prevTitle = titles.get(p.title);
    if (prevTitle) {
      throw new MergeError(`[${p.id}] title が ${prevTitle} と重複しています: 「${p.title}」`);
    }
    titles.set(p.title, p.id);

    const prevAnswer = answers.get(p.answer_code);
    if (prevAnswer) {
      throw new MergeError(`[${p.id}] answer_code が ${prevAnswer} と完全に同一です`);
    }
    answers.set(p.answer_code, p.id);
  });

  return out;
}

function readBatches(lang, level) {
  const dir = join("data/problems", lang, "_batches");
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => f.startsWith(`${level.slug}-`) && f.endsWith(".json"))
    .sort()
    .map((f) => JSON.parse(readFileSync(join(dir, f), "utf8")));
}

function mergeOne(lang, level, expected) {
  const batches = readBatches(lang, level);
  if (batches.length === 0) {
    return { skipped: true };
  }
  const out = mergeBatches(batches, lang, level, expected);
  const target = join("data/problems", lang, `${level.slug}.json`);
  mkdirSync(join("data/problems", lang), { recursive: true });
  // 既存の Rust データと同じシリアライズ形式 (2 スペース / 非 ASCII をエスケープしない)
  writeFileSync(target, JSON.stringify(out, null, 2) + "\n");
  return { count: out.length, batches: batches.length, target };
}

/// 統合に成功したバッチを片付ける。
/// trunk は data/ を丸ごと dist へコピーするので、_batches を残すと中間成果物が本番に載る。
function cleanBatches(lang) {
  const dir = join("data/problems", lang, "_batches");
  if (existsSync(dir)) rmSync(dir, { recursive: true, force: true });
}

function main() {
  const args = process.argv.slice(2).filter((a) => a !== "--clean");
  const clean = process.argv.slice(2).includes("--clean");
  const expected = 100;
  let langs;
  let levels = LEVELS;

  if (args[0] === "--all") {
    langs = LANGS;
  } else if (args[0]) {
    if (!LANGS.includes(args[0])) {
      console.error(`不明な言語: ${args[0]} (${LANGS.join(" / ")})`);
      process.exit(1);
    }
    langs = [args[0]];
    if (args[1]) {
      const lv = LEVELS.find((l) => l.slug === args[1]);
      if (!lv) {
        console.error(`不明な難易度: ${args[1]}`);
        process.exit(1);
      }
      levels = [lv];
    }
  } else {
    console.error("usage: node scripts/merge-batches.mjs <lang> [level] | --all");
    process.exit(1);
  }

  let failed = 0;
  let merged = 0;
  for (const lang of langs) {
    for (const level of levels) {
      try {
        const r = mergeOne(lang, level, expected);
        if (r.skipped) {
          console.log(`- ${lang}/${level.slug}: バッチなし (スキップ)`);
        } else {
          console.log(`✓ ${r.target}: ${r.count} 問 (バッチ ${r.batches} 個)`);
          merged++;
        }
      } catch (e) {
        console.error(`✗ ${lang}/${level.slug}: ${e.message}`);
        failed++;
      }
    }
  }
  if (clean && failed === 0) {
    for (const lang of langs) cleanBatches(lang);
    console.log("バッチディレクトリを削除しました (--clean)");
  }
  console.log(`---\n統合 ${merged} ファイル / 失敗 ${failed} 件`);
  process.exit(failed === 0 ? 0 : 1);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
