# 問題コンテンツの書き方 (全言語共通契約)

このファイルは、問題を生成するすべてのエージェントが従う**契約**である。
バックエンド選定と言語別制約の正本は `docs/decisions/0002-multi-language-execution-backends.md`。

## 1. ファイル配置とスキーマ

```
data/problems/<lang>/<level>.json     # 1 ファイル = 100 問の配列
```

`<lang>` は `rust` / `cpp` / `csharp` / `java` / `python` / `typescript` / `javascript`。
`<level>` は `beginner` / `intermediate` / `advanced`。

1 問のスキーマ (`crates/shared/src/problem.rs` の `Problem` が正本):

| フィールド | 型 | 内容 |
|---|---|---|
| `id` | string | `b001`〜`b100` / `i001`〜`i100` / `a001`〜`a100`。**言語内で一意**であればよい |
| `language` | string | `<lang>` と一致。**必須** (欠けると parse エラー) |
| `level` | string | `<level>` と一致 |
| `title` | string | 日本語の短い題名 |
| `description_md` | string | 問題文 (Markdown)。何を実装するかと入出力例 |
| `starter_code` | string | 利用者に最初に見せるコード。**未完成**であること |
| `hidden_tests` | string | 判定用コード。UI には出さない |
| `answer_code` | string | 模範解答。これ単体 + `hidden_tests` でテストが通ること |
| `explanation_md` | string | なぜそう書くのかの解説 (Markdown) |
| `hints` | string[] | 段階的なヒント 1〜3 個 |
| `tags` | string[] | 日本語の分類タグ 1〜3 個 |

## 2. 判定契約 — これが全言語で守るべき唯一の約束

提出コードは `<user_code>` + 区切りコメント + `<hidden_tests>` を**この順で連結**して実行する
(`compose_submission`)。ユーザーコードを先頭に置くのは、コンパイラ診断の**行番号を
ユーザーが書いた行と一致させる**ためである。この順序は変えてはならない。

`hidden_tests` は次を満たすこと:

- 全検査に通る → **stdout** に `test result: ok` を出し **終了コード 0**
- 検査に落ちる → **stdout** に `test result: FAILED` を出し、
  内訳 `FAILED: <検査名>` を **stderr** に出して **終了コード != 0**

**目印 (`test result: ...`) は必ず stdout に出す。** stderr はコンパイラ診断と
実行時エラーの表示専用である。目印の文字列を 1 文字でも変えると判定が壊れる。

終了コード 0 でも `test result: ok` が出ていなければ**正解にはならない**
(「テストが走らないまま終了した」と判定される)。

**検査は最低 2 件入れること。** 1 件だけだと、たまたま通る実装を正解と誤判定しやすい。
境界値 (0 / 負数 / 空 / 1 要素) を最低 1 件は含める。

**禁止**: 問題を通すために `hidden_tests` を緩めること。`answer_code` が通らないなら
問題そのものを作り直す。

## 3. 言語別テンプレート

以下はすべて実機 (Wandbox / ローカル Docker) で動作確認済みの形。この骨格から外れないこと。

### Rust (既存 300 問と同形式 — 変更しない)

`hidden_tests` は `#[test]` 関数群。判定は `cargo test` の結果を使うので、
`test result: ok` / `FAILED` の目印を自分で書く必要はない。

```rust
#[test]
fn doubles_positive() { assert_eq!(double(2), 4); }
#[test]
fn doubles_zero() { assert_eq!(double(0), 0); }
```

### C++ (`gcc-13.2.0`, `-std=c++17`, ファイル名 `prog.cc`)

`starter_code` / `answer_code` は関数だけを定義し、**`main` を書かない** (`main` は harness 側)。
必要な `#include` はユーザーコード側に書く。

```cpp
// ===== hidden_tests =====
#include <iostream>
static int knock_failed = 0;
static void knock_check(bool cond, const char* name) {
    if (!cond) { std::cerr << "FAILED: " << name << "\n"; ++knock_failed; }
}
int main() {
    knock_check(add(1, 2) == 3, "add_positive");
    knock_check(add(-1, 1) == 0, "add_zero");
    if (knock_failed > 0) { std::cout << "test result: FAILED\n"; return 1; }
    std::cout << "test result: ok\n";
    return 0;
}
```

### Java (`openjdk-jdk-22+36`, ファイル名 `prog.java`)

**クラスを `public` にしてはいけない。** ファイル名が `prog.java` 固定のため
`class X is public, should be declared in a file named X.java` で落ちる。
これは Wandbox の都合なので、`description_md` では触れない (誤った Java 知識を与えないため)。
ユーザーコードは `class Solution`、harness は `class Main` を使う。

```java
// ===== hidden_tests =====
class Main {
    static int failed = 0;
    static void check(boolean cond, String name) {
        if (!cond) { System.err.println("FAILED: " + name); failed++; }
    }
    public static void main(String[] args) {
        check(Solution.add(1, 2) == 3, "add_positive");
        check(Solution.add(-1, 1) == 0, "add_zero");
        if (failed > 0) { System.out.println("test result: FAILED"); System.exit(1); }
        System.out.println("test result: ok");
    }
}
```

### C# (`dotnetcore-6.0.425`, ファイル名 `prog.cs`)

`using` はファイル先頭にしか置けず harness は後ろに来るので、**harness は完全修飾で書く**
(`System.Console.WriteLine`)。ユーザーコードは `class Solution`、harness は `class KnockTests`。
トップレベルステートメントは使わない (エントリポイントが衝突する)。

```csharp
// ===== hidden_tests =====
class KnockTests
{
    static int failed = 0;
    static void Check(bool cond, string name)
    {
        if (!cond) { System.Console.Error.WriteLine("FAILED: " + name); failed++; }
    }
    static int Main()
    {
        Check(Solution.Add(1, 2) == 3, "add_positive");
        Check(Solution.Add(-1, 1) == 0, "add_zero");
        if (failed > 0) { System.Console.WriteLine("test result: FAILED"); return 1; }
        System.Console.WriteLine("test result: ok");
        return 0;
    }
}
```

### Python (`cpython-3.13.8`, ファイル名 `prog.py`)

```python
# ===== hidden_tests =====
import sys

_failed = 0

def _check(cond, name):
    global _failed
    if not cond:
        print("FAILED: " + name, file=sys.stderr)
        _failed += 1

_check(add(1, 2) == 3, "add_positive")
_check(add(-1, 1) == 0, "add_zero")

if _failed > 0:
    print("test result: FAILED")
    sys.exit(1)
print("test result: ok")
```

例外は**捕まえない**。未実装の `starter_code` が投げる `NotImplementedError` の
本物のトレースバックを利用者に見せる (それがこのアプリの価値)。

### JavaScript (`nodejs-20.17.0`, ファイル名 `prog.js`, CommonJS)

```javascript
// ===== hidden_tests =====
let __failed = 0;
function __check(cond, name) {
  if (!cond) { console.error("FAILED: " + name); __failed++; }
}
__check(add(1, 2) === 3, "add_positive");
__check(add(-1, 1) === 0, "add_zero");
if (__failed > 0) { console.log("test result: FAILED"); process.exit(1); }
console.log("test result: ok");
```

`import` / `export` は使えない (CommonJS)。必要なら `require` を使う。

### TypeScript (`typescript-5.6.2`, ファイル名 `prog.ts`)

**Node の型定義が無い。** `process` をそのまま使うと `TS2580` になるので、
harness の先頭で自分で宣言する。`console` は既定の lib に含まれるのでそのまま使える。

```typescript
// ===== hidden_tests =====
declare const process: { exit(code: number): never };
let __failed = 0;
function __check(cond: boolean, name: string): void {
  if (!cond) { console.error("FAILED: " + name); __failed++; }
}
__check(add(1, 2) === 3, "add_positive");
__check(add(-1, 1) === 0, "add_zero");
if (__failed > 0) { console.log("test result: FAILED"); process.exit(1); }
console.log("test result: ok");
```

TypeScript の問題は**型を主題にできる**のが強み (型エラーは実物の `TS2322` が出る)。

## 4. starter_code の作り方

- **必ず未完成**にする。`starter_code` のまま実行したらテストが落ちること (verifier が検査する)
- 未実装部分は言語ごとの慣用的な「穴」にする:

| 言語 | 穴の書き方 |
|---|---|
| Rust | `todo!()` |
| C++ | `return {};` + `// TODO:` コメント |
| Java | `return 0;` / `return null;` + `// TODO:` |
| C# | `return 0;` / `return null;` + `// TODO:` |
| Python | `raise NotImplementedError` |
| JavaScript | `// TODO:` (何も返さない = `undefined`) |
| TypeScript | `throw new Error("TODO");` (戻り値型を満たすため) |

- 関数シグネチャ・クラス名は `answer_code` と一致させる (`hidden_tests` が呼ぶため)
- `starter_code` と `answer_code` が同一になってはいけない (verifier が検査する)

## 5. 難易度の目安

| レベル | 中身 |
|---|---|
| 初級 (b) | 言語の基本文法・標準ライブラリの基本。1 関数で解ける |
| 中級 (i) | データ構造・イテレーション・エラー処理・その言語固有の中核概念 (所有権 / GC / 型システム / 非同期の基礎など) |
| 上級 (a) | 設計判断を伴うもの。ジェネリクス・トレイト/インタフェース・並行性・パフォーマンス・その言語の落とし穴 |

各言語の**その言語らしさ**を主題にする。7 言語で同じ問題を翻訳しただけの集合にしない。

## 6. 品質ゲート

収録前に必ず verifier を通す。1 問でも落ちたら**その問題を直す**。

```bash
cargo run -p verifier -- --lang <lang> --file <batch.json> --level <level>
```

verifier は各問について次を機械検査する:

1. `answer_code` + `hidden_tests` を実行して**通る** (exit 0 かつ stdout に `test result: ok`)
2. `starter_code` + `hidden_tests` を実行して**落ちる**
3. スキーマ整合 — id 形式・id 重複・`language`/`level` がファイルのパスと一致・空フィールド・
   `starter_code != answer_code`
4. `hidden_tests` が**最低 2 件**の検査を含む (Rust は `#[test]` が 2 個以上、
   他言語は成功/失敗の目印を出力していること)
5. **使い回しの検出** — 同一ファイル内で `title` が重複しない・`answer_code` が重複しない・
   `description_md` が最低 80 文字ある

5 がある理由: 「1 問を連番でコピーした 100 問」は 1〜4 をすべて満たしてしまう。
中身が違うことは人間が見ないと分からないので、機械で取れる最低限だけは取る。

実行はローカル Docker (版は Wandbox に一致) で行う。**Wandbox に一括で投げてはいけない。**

verifier には 1 問あたりの実行時間上限がある (無限ループを書いた問題でバッチ全体が
止まらないため)。上限を超えた問題は「未検証」ではなく**失敗**として数える。
