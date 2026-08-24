// 生成バッチを data/problems/<level>.json に統合する使い捨てスクリプト
// usage: node scripts/_merge-batches.mjs <level> <batchDir>
import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const [level, batchDir] = process.argv.slice(2);
if (!level || !batchDir) {
  console.error("usage: node scripts/_merge-batches.mjs <level> <batchDir>");
  process.exit(1);
}

const target = `data/problems/${level}.json`;
const existing = JSON.parse(readFileSync(target, "utf8"));
const batchFiles = readdirSync(batchDir)
  .filter((f) => f.startsWith(`${level}-`) && f.endsWith(".json"))
  .sort();

const merged = new Map(existing.map((p) => [p.id, p]));
for (const f of batchFiles) {
  const arr = JSON.parse(readFileSync(join(batchDir, f), "utf8"));
  for (const p of arr) {
    if (merged.has(p.id)) {
      console.error(`duplicate id ${p.id} in ${f}`);
      process.exit(1);
    }
    merged.set(p.id, p);
  }
}

const out = [...merged.values()].sort((a, b) => a.id.localeCompare(b.id));
// 連番検査
out.forEach((p, i) => {
  const expect = `${p.id[0]}${String(i + 1).padStart(3, "0")}`;
  if (p.id !== expect) {
    console.error(`id 欠番/順序異常: ${p.id} (期待 ${expect})`);
    process.exit(1);
  }
});
writeFileSync(target, JSON.stringify(out, null, 2) + "\n");
console.log(`${target}: ${out.length} 問 (batches: ${batchFiles.length})`);
