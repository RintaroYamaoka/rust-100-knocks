# ⚙️ 100本ノック

ブラウザで解くコーディング練習アプリ。**7 言語 × 各300問 (計2100問)**。
アプリ本体はフロントエンドもバックエンドも Rust で書かれています。

- **Rust / C++ / C# / Java / Python / TypeScript / JavaScript** — 各言語 初級・中級・上級 100 問ずつ
- **本物のコンパイラのエラー** — 実物の `rustc` / `gcc` / `javac` / Roslyn / `tsc` / CPython / Node が出した
  診断をそのまま表示。作文したエラーメッセージは 1 つもありません
- **CodeMirror 6 エディタ** — 言語ごとのシンタックスハイライト、`Ctrl+Enter` で即実行
- **正誤判定** — 各問題の隠しテストを実行して自動判定 (✓ 正解 / △ テスト失敗 / ✗ コンパイルエラー)
- **回答例と丁寧な解説** — ネタバレ防止ゲートつき
- **進捗管理と絞り込み** — 言語ごとに独立。未回答のみ / 未正解のみ / 正解済み、検索、達成率 (localStorage 保存)
- **Zed 風の可変レイアウト** — サイドバー・問題ペイン・コンソールの境界をドラッグでリサイズ (保存される)

## 技術スタック

| 層 | 技術 |
|---|---|
| フロントエンド | [Leptos](https://leptos.dev/) (CSR) → WebAssembly、[Trunk](https://trunkrs.dev/) ビルド |
| エディタ | CodeMirror 6 (バンドル済み + wasm-bindgen interop) |
| バックエンド | Vercel 公式 Rust ランタイム (`vercel_runtime` 2.x) |
| コード実行 | Rust → [Rust Playground](https://play.rust-lang.org/) / 他 6 言語 → [Wandbox](https://wandbox.org/) へのプロキシ |
| 問題データ | リポジトリ内 JSON (全問題を `verifier` が実コンパイラで実行検証済み) |

実行バックエンドの選定理由と言語ごとの制約は
[ADR 0002](docs/decisions/0002-multi-language-execution-backends.md) が正本です。

## 開発

```bash
rustup target add wasm32-unknown-unknown

vercel dev             # http://127.0.0.1:3000 — /api/execute を動かす
trunk serve            # http://127.0.0.1:8080 — フロント (API は vercel dev へプロキシ)

cargo test --workspace --exclude app   # 契約層・プロキシ・verifier のテスト
cargo test -p app                      # フロントの純ロジック

cargo run -p verifier                  # 全 2100 問を実コンパイラで検証
cargo run -p verifier -- --lang cpp    # 1 言語だけ
```

`verifier` は Docker で言語ごとのコンパイラを動かします。初回は次のイメージが必要です:

```bash
docker pull gcc:13 eclipse-temurin:22-jdk mcr.microsoft.com/dotnet/sdk:6.0 python:3.13 node:20
printf 'FROM node:20\nRUN npm install -g typescript@5.6.2\n' | docker build -t knocks-ts:5.6.2 -
```

## アーキテクチャ

```
crates/shared    言語定義・問題スキーマ・実行API契約・進捗モデル (全 crate が依存する契約層)
crates/app       Leptos フロントエンド (wasm32)
crates/verifier  問題品質検証ハーネス (Docker で実コンパイラを回す)
api/execute.rs   Vercel Rust Function — Playground / Wandbox への振り分けプロキシ
data/problems/<言語>/<難易度>.json   問題データ (21 ファイル)
docs/problem-authoring.md            問題を書くときの契約 (全言語共通)
```

正誤判定はサーバーに状態を持ちません。ユーザーコードと隠しテストを結合して実行し、
**終了コードと標準出力の目印**だけで結果を分類します。「終了コード 0」だけでは正解にせず、
テストが最後まで走った目印 (`test result: ok`) があることを必要条件にしています
(そうしないと、ユーザーコードが判定テストより先に `exit(0)` するだけで「正解」になってしまうため)。

## デプロイ

Vercel にそのままデプロイできます (`vercel.json` + `scripts/build-frontend.sh`)。
静的フロント + `api/execute` の Rust Function 構成です。

リポジトリ直下に `build.sh` を置いてはいけません (Rust ランタイムが関数ビルド前フックとして
自動実行してしまいます)。フロントのビルドスクリプトを `scripts/` に置いているのはそのためです。
経緯は [インシデント記録](docs/bootstrap/incidents/2026-08-25-vercel-rust-deploy-failures.md) にあります。
