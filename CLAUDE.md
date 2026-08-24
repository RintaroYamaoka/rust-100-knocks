# CLAUDE.md

## プロジェクト概要

Rust コーディング練習アプリ「Rust 100本ノック」。ブラウザエディタで問題を解き、Rust Playground API 経由で実際の rustc エラー・テスト結果を返す。初級/中級/上級 各100問(計300問)、回答例と解説つき。Vercel にデプロイ(静的 WASM フロント + Rust Functions)。

## 技術スタック

- 言語: Rust(フロント・バック共通)。フロントは Leptos (CSR) を wasm32-unknown-unknown + Trunk でビルド
- エディタ: CodeMirror 6(`assets/vendor/` に vendor、wasm-bindgen JS interop)
- バックエンド: Vercel Rust Functions (`vercel_runtime`) — `api/execute.rs` が play.rust-lang.org/execute をプロキシ
- 問題データ: `data/problems/*.json`(静的配信)。スキーマは `crates/shared`
- 進捗保存: ブラウザ localStorage(サーバー状態なし)
- テスト: cargo test(workspace)。問題コンテンツの品質検証は `crates/verifier`

## 開発コマンド

```bash
# テスト実行 (全件)
cargo test --workspace --exclude app

# 問題コンテンツ検証 (全問題の answer_code + hidden_tests をローカル cargo で実行)
cargo run -p verifier

# 開発サーバー起動 (フロント)
trunk serve

# API ローカル実行
vercel dev
```

## ディレクトリマップ

```
.
├── crates/shared/    # 問題スキーマ・API契約・進捗モデル (front/back/verifier 共有)
├── crates/app/       # Leptos フロントエンド (wasm32)
├── crates/verifier/  # 問題品質検証ハーネス
├── api/              # Vercel Rust Functions (execute.rs)
├── data/problems/    # 問題データ JSON (beginner/intermediate/advanced)
├── assets/           # CSS / JS glue / CodeMirror vendor
└── docs/             # bootstrap 規律 (handoffs/incidents/sprint/verification) + ADR
```

## アーキテクチャ概略

- 依存方向: `app` / `api` / `verifier` → `shared`。逆依存禁止。API 契約 (`ExecuteRequest/Response`) と問題スキーマ (`Problem`) の変更は必ず `shared` で行う
- 正誤判定はサーバー側に状態を持たない: ユーザーコード + `hidden_tests` を結合して Playground(tests mode)で実行し、結果種別(コンパイルエラー/テスト失敗/全パス)をフロントで解釈する
- `crates/app` は wasm32 専用ターゲット。純ロジック(フィルタ・進捗集計・出力パース)は host でもテストできるよう UI から分離して書く

## 既知の地雷

- Playground API は非公式・レート制限あり。問題コンテンツの検証は必ず `verifier`(ローカル cargo)で行い、Playground に一括負荷をかけない
- Vercel ビルドで `cargo install trunk` は遅すぎる。`scripts/build-frontend.sh` は prebuilt バイナリをダウンロードする
- リポジトリ直下に `build.sh` を置いてはならない: vercel-rust ランタイムが関数ビルド前フックとして自動実行してしまう (2026-08-25 のデプロイ失敗の原因)
- `crates/app` を `cargo test --workspace` に含めると wasm 前提コードが host ビルドで壊れることがある。テストは `--exclude app` で回し、app の純ロジックは shared 側に置く
