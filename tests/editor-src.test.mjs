// ビルド済みバンドル (assets/js/editor.js) の公開契約テスト。
// 実行: npm run test:editor   (= node --test tests/editor-src.test.mjs)
// バンドル再生成 (npm ci && npm run build:editor) の後に必ず叩く。
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

import { EditorState } from "@codemirror/state";
import { StreamLanguage, ensureSyntaxTree } from "@codemirror/language";
import { rust } from "@codemirror/lang-rust";
import { cpp } from "@codemirror/lang-cpp";
import { java } from "@codemirror/lang-java";
import { python } from "@codemirror/lang-python";
import { javascript } from "@codemirror/lang-javascript";
import { csharp as csharpStreamParser } from "@codemirror/legacy-modes/mode/clike";

const bundlePath = fileURLToPath(new URL("../assets/js/editor.js", import.meta.url));

// crates/shared/src/language.rs の Language::slug() と一致していること。
const SLUGS = ["rust", "cpp", "csharp", "java", "python", "typescript", "javascript"];

/**
 * バンドル (IIFE) を隔離した偽 window / document / clearTimeout の上で評価し、
 * 公開 API と「clearTimeout が呼ばれたか」を観測できるようにする。
 * document.getElementById は常に null を返すので mount は DOM 無しで false を返す
 * (= 各メソッド先頭の clearTimeout は、その早期 return より前に走る必要がある)。
 */
async function loadBundle() {
  const src = await readFile(bundlePath, "utf8");
  const cleared = [];
  const realClearTimeout = globalThis.clearTimeout;
  const realWindow = globalThis.window;
  const realDocument = globalThis.document;

  globalThis.window = {};
  // @codemirror/view はモジュール評価時にブラウザ判定で
  // document.documentElement.style を読むので、そこまでは用意しておく。
  globalThis.document = {
    documentElement: { style: {} },
    getElementById: () => null,
  };
  globalThis.clearTimeout = (id) => {
    cleared.push(id);
    return realClearTimeout(id);
  };
  try {
    new Function(src)();
  } finally {
    globalThis.clearTimeout = realClearTimeout;
  }
  const api = globalThis.window.RustKnocksEditor;
  const restore = () => {
    globalThis.window = realWindow;
    globalThis.document = realDocument;
  };
  // API を呼ぶ間だけ偽 clearTimeout を差し込むヘルパ
  const observing = (fn) => {
    const before = cleared.length;
    globalThis.clearTimeout = (id) => {
      cleared.push(id);
      return realClearTimeout(id);
    };
    try {
      return { result: fn(), clearedCount: cleared.length - before };
    } finally {
      globalThis.clearTimeout = realClearTimeout;
    }
  };
  return { api, observing, restore };
}

test("bundle exposes the RustKnocksEditor API on window", async () => {
  const { api, restore } = await loadBundle();
  try {
    assert.ok(api, "window.RustKnocksEditor が定義されていない");
    for (const method of [
      "mount",
      "getValue",
      "setValue",
      "focus",
      "setOnRun",
      "setOnSave",
      "setOnChange",
      "setLanguage",
      "getLanguage",
    ]) {
      assert.equal(typeof api[method], "function", `${method} がない`);
    }
    assert.equal(api.getValue(), "", "未マウント時の getValue は空文字を返す");
    assert.equal(api.getLanguage(), "rust", "既定の言語は rust");
  } finally {
    restore();
  }
});

test("setLanguage が 7 slug すべてを受け付ける", async () => {
  const { api, restore } = await loadBundle();
  try {
    for (const slug of SLUGS) {
      assert.equal(api.setLanguage(slug), true, `${slug} が拒否された`);
      assert.equal(api.getLanguage(), slug, `${slug} が現在言語に反映されていない`);
    }
  } finally {
    restore();
  }
});

test("未知の slug では例外を投げず、現在の言語を維持する", async () => {
  const { api, restore } = await loadBundle();
  try {
    api.setLanguage("python");
    for (const bogus of ["", "ruby", "RUST", "c++", "rust ", null, undefined, 42, {}]) {
      let ret;
      assert.doesNotThrow(() => {
        ret = api.setLanguage(bogus);
      }, `setLanguage(${JSON.stringify(bogus)}) が例外を投げた`);
      assert.equal(ret, false, `未知の slug ${JSON.stringify(bogus)} が受理された`);
      assert.equal(api.getLanguage(), "python", "未知の slug で現在言語が変わった");
    }
  } finally {
    restore();
  }
});

test("mount / setValue / setLanguage は先頭で clearTimeout を呼ぶ", async () => {
  const { api, observing, restore } = await loadBundle();
  try {
    // mount: DOM が無く false で早期 return する経路でも clearTimeout は済んでいること
    const mounted = observing(() => api.mount("no-such-element", "fn main() {}"));
    assert.equal(mounted.result, false, "DOM が無いので mount は false を返すはず");
    assert.ok(mounted.clearedCount >= 1, "mount が clearTimeout を呼んでいない");

    // setValue: view が無く早期 return する経路でも同様
    const set = observing(() => api.setValue("fn main() {}"));
    assert.ok(set.clearedCount >= 1, "setValue が clearTimeout を呼んでいない");

    // setLanguage: 既知の slug
    const known = observing(() => api.setLanguage("java"));
    assert.equal(known.result, true);
    assert.ok(known.clearedCount >= 1, "setLanguage が clearTimeout を呼んでいない");

    // setLanguage: 未知の slug でも先頭で呼ぶ (現状維持でも debounce は捨てる)
    const unknown = observing(() => api.setLanguage("ruby"));
    assert.equal(unknown.result, false);
    assert.ok(unknown.clearedCount >= 1, "未知 slug の setLanguage が clearTimeout を呼んでいない");

    // 同じ slug への再設定 (早期 return する経路) でも呼ぶ
    const same = observing(() => api.setLanguage("java"));
    assert.equal(same.result, true);
    assert.ok(same.clearedCount >= 1, "同一 slug の setLanguage が clearTimeout を呼んでいない");
  } finally {
    restore();
  }
});

test("バンドルが再マウントではなく Compartment で切り替えている (ソースの回帰ガード)", async () => {
  const srcPath = fileURLToPath(new URL("../assets/js/editor-src.mjs", import.meta.url));
  const src = await readFile(srcPath, "utf8");
  assert.match(src, /Compartment/, "Compartment を使っていない");
  assert.match(src, /reconfigure/, "reconfigure による切替になっていない");
  // setLanguage の中で destroy / new EditorView をしていないこと
  const setLanguageBody = src.slice(src.indexOf("setLanguage(slug)"), src.indexOf("getLanguage()"));
  assert.doesNotMatch(setLanguageBody, /destroy\(\)/, "setLanguage がエディタを destroy している");
  assert.doesNotMatch(setLanguageBody, /new EditorView/, "setLanguage がエディタを再マウントしている");
});

// ---- 言語モードの実体テスト (固定した依存版が本当にハイライトできるか) ----

const MODES = {
  rust: [rust(), 'fn main() {\n    let x: u32 = 1;\n    println!("{}", x);\n}\n'],
  cpp: [cpp(), "#include <iostream>\nint main() {\n    std::cout << 1;\n    return 0;\n}\n"],
  csharp: [
    StreamLanguage.define(csharpStreamParser),
    "class Main {\n    static void Main() {\n        System.Console.WriteLine(1);\n    }\n}\n",
  ],
  java: [java(), "class Main {\n    static void main(String[] a) {\n        System.out.println(1);\n    }\n}\n"],
  python: [python(), "def main():\n    print(1)\n"],
  typescript: [javascript({ typescript: true }), "const x: number = 1;\nconsole.log(x);\n"],
  javascript: [javascript(), "const x = 1;\nconsole.log(x);\n"],
};

function parse(extension, doc) {
  const state = EditorState.create({ doc, extensions: [extension] });
  const tree = ensureSyntaxTree(state, doc.length, 10000);
  assert.ok(tree, "構文木が得られない");
  let errors = 0;
  let nodes = 0;
  tree.iterate({
    enter: (n) => {
      nodes += 1;
      if (n.type.isError) errors += 1;
    },
  });
  return { tree, errors, nodes };
}

for (const [slug, [extension, doc]] of Object.entries(MODES)) {
  test(`${slug} の言語モードが正しいコードをエラーなく解析する`, () => {
    const { errors, nodes } = parse(extension, doc);
    assert.equal(errors, 0, `${slug}: 正しいコードにエラーノードが出た`);
    assert.ok(nodes > 3, `${slug}: 構文木が実質空 (nodes=${nodes})`);
  });
}

test("Rust ハイライトが生きている: 壊れたコードはエラーノードになる", () => {
  // 「エラー 0 件」だけだと素通しのパーサでも通ってしまうので、
  // 壊れたコードで必ずエラーが出ることを対にして確認する。
  const { errors } = parse(rust(), "fn main( { let = ; }");
  assert.ok(errors > 0, "壊れた Rust コードでエラーノードが出ない (パーサが効いていない)");
});

test("Rust の構文木がキーワード / 関数を認識している", () => {
  const doc = 'fn main() {\n    let x: u32 = 1;\n    println!("{}", x);\n}\n';
  const { tree } = parse(rust(), doc);
  const names = new Set();
  tree.iterate({ enter: (n) => names.add(n.name) });
  // lang-rust のノード名。ここが変わったら依存版の上げ方を見直すサイン。
  assert.ok(names.has("FunctionItem"), `FunctionItem が無い: ${[...names].join(",")}`);
  assert.ok(names.has("LetDeclaration"), `LetDeclaration が無い: ${[...names].join(",")}`);
  assert.ok(names.has("MacroInvocation"), `MacroInvocation が無い: ${[...names].join(",")}`);
});
