# 🦀 Rust 100本ノック

ブラウザで解く Rust コーディング練習アプリ。**フロントエンドもバックエンドも Rust** で書かれています。

- **初級 / 中級 / 上級 × 各100問 (計300問)** — カリキュラム順に並んだ演習問題
- **本物の rustc エラー** — Rust Playground でコンパイル・実行し、`E0308` などの実際のエラーがそのまま表示。エラーコードクリックで公式解説へ
- **CodeMirror 6 エディタ** — Rust シンタックスハイライト、`Ctrl+Enter` で即実行
- **正誤判定** — 各問題の隠しテストを実行して自動判定 (✓ 正解 / △ 挑戦中)
- **回答例と丁寧な解説** — ネタバレ防止ゲートつき
- **進捗管理と絞り込み** — 未回答のみ / 未正解のみ / 正解済み、検索、達成率 (localStorage 保存)

## 技術スタック

| 層 | 技術 |
|---|---|
| フロントエンド | [Leptos](https://leptos.dev/) (CSR) → WebAssembly、[Trunk](https://trunkrs.dev/) ビルド |
| エディタ | CodeMirror 6 (vendored bundle + wasm-bindgen interop) |
| バックエンド | Vercel Rust Functions ([vercel-rust](https://github.com/vercel-community/rust)) |
| コード実行 | [Rust Playground](https://play.rust-lang.org/) API プロキシ |
| 問題データ | リポジトリ内 JSON (全問題は `verifier` によりローカル cargo で実行検証済み) |

## 開発

```bash
rustup target add wasm32-unknown-unknown
# trunk をインストールして
trunk serve            # http://127.0.0.1:8080 (実行APIは Playground へ dev プロキシ)

cargo test --workspace # 全テスト
cargo run -p verifier  # 全問題の answer/starter を実 cargo で検証
```

## アーキテクチャ

```
crates/shared    問題スキーマ・実行API契約・進捗モデル (全 crate が依存する契約層)
crates/app       Leptos フロントエンド (wasm32)
crates/verifier  問題品質検証ハーネス (answer が通り、starter が通らないことを機械検証)
api/execute.rs   Vercel Rust Function — Playground への許可リスト検証つきプロキシ
data/problems/   問題データ (beginner / intermediate / advanced)
```

正誤判定はサーバーに状態を持ちません: ユーザーコード + 隠しテストを結合して Playground の tests モードで実行し、結果 (全パス / テスト失敗 / コンパイルエラー) をフロントで分類します。

## デプロイ

Vercel にそのままデプロイできます (`vercel.json` + `scripts/build-frontend.sh`)。静的フロント + `api/execute` の Rust Function 構成です。
(注意: リポジトリ直下の `build.sh` は vercel-rust が関数ビルドフックとして実行するため、フロントのビルドスクリプトは `scripts/` に置いています)
