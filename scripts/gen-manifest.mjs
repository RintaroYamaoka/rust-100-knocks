// 収録済み言語のマニフェストを data/problems/index.json に生成する。
//
// フロントはこれ 1 本でセレクタの中身を決める。以前は起動時に 21 本の HEAD を
// 逐次投げていたが、揃うまで数秒かかり、その間 Rust しか選べなかった。
//
// usage: node scripts/gen-manifest.mjs
import { existsSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const LANGS = ["rust", "cpp", "csharp", "java", "python", "typescript", "javascript"];
const LEVELS = ["beginner", "intermediate", "advanced"];
const DIR = "data/problems";

/**
 * @param {(relPath: string) => boolean} exists `<lang>/<level>.json` が在るか
 */
export function buildManifest(exists) {
  const languages = [];
  for (const slug of LANGS) {
    const levels = LEVELS.filter((lv) => exists(`${slug}/${lv}.json`));
    if (levels.length > 0) languages.push({ slug, levels });
  }
  return { languages };
}

function main() {
  const m = buildManifest((rel) => existsSync(join(DIR, rel)));
  const target = join(DIR, "index.json");
  writeFileSync(target, JSON.stringify(m, null, 2) + "\n");
  const complete = m.languages.filter((l) => l.levels.length === 3);
  console.log(`✓ ${target}: ${complete.length} 言語が 3 レベル揃っている`);
  for (const l of m.languages) {
    if (l.levels.length !== 3) console.warn(`  ⚠ ${l.slug}: ${l.levels.join(",")} のみ (セレクタに出ない)`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
