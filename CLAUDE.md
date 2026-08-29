# CLAUDE.md

## プロジェクト概要

コーディング練習アプリ「100本ノック」。ブラウザエディタで問題を解き、実物のコンパイラ/ランタイムの
エラー・テスト結果を返す。**Rust / C++ / C# / Java / Python / TypeScript / JavaScript の 7 言語**、
各言語 初級/中級/上級 各100問 (計2100問)、回答例と解説つき。
Vercel にデプロイ(静的 WASM フロント + Rust Functions)。

## 技術スタック

- 言語: アプリ本体は Rust。フロントは Leptos (CSR) を wasm32-unknown-unknown + Trunk でビルド
- エディタ: CodeMirror 6(`assets/js/editor.js` は esbuild 生成物、ソースは `editor-src.mjs`)。
  言語モードは `window.RustKnocksEditor.setLanguage(slug)` で切り替える
- バックエンド: Vercel **公式** Rust ランタイム (`vercel_runtime = "2"`, hyper 1 ベース)。
  `api/execute.rs` が **Rust → play.rust-lang.org / 他 6 言語 → wandbox.org** に振り分ける。
  `vercel.json` に `functions.runtime` は書かない (Cargo.toml の `[[bin]]` を自動検出)
- 問題データ: `data/problems/<言語>/<難易度>.json`(静的配信)。スキーマは `crates/shared`
- 進捗保存: ブラウザ localStorage(サーバー状態なし)。キーは `<言語>/<問題id>`
- テスト: cargo test(workspace)。問題コンテンツの品質検証は `crates/verifier`

## 開発コマンド

```bash
# テスト実行
cargo test --workspace --exclude app   # 契約層・プロキシ・verifier
cargo test -p app                      # フロントの純ロジック (host でコンパイルできる範囲)

# 問題コンテンツ検証 (実コンパイラを Docker で回す)
cargo run -p verifier                            # 全 2100 問
cargo run -p verifier -- --lang cpp              # 1 言語
cargo run -p verifier -- --expect 2100           # 件数まで含めて検算

# 開発サーバー
vercel dev             # :3000 で /api/execute
trunk serve            # :8080 でフロント (API は vercel dev へプロキシ)
```

## ディレクトリマップ

```
.
├── crates/shared/    # 言語定義・問題スキーマ・API契約・進捗モデル (front/back/verifier 共有)
├── crates/app/       # Leptos フロントエンド (wasm32)
├── crates/verifier/  # 問題品質検証ハーネス (docker.rs が docker run を組み立てる唯一の場所)
├── api/              # Vercel Rust Functions (execute.rs)
├── data/problems/    # 問題データ JSON (<言語>/<難易度>.json の 21 ファイル)
├── assets/           # CSS / JS glue / CodeMirror バンドル
└── docs/             # ADR + bootstrap 規律 (handoffs/incidents/sprint/verification/commission)
                      # problem-authoring.md = 問題を書くときの全言語共通契約
```

## アーキテクチャ概略

- 依存方向: `app` / `api` / `verifier` → `shared`。逆依存禁止。API 契約 (`ExecuteRequest/Response`)、
  問題スキーマ (`Problem`)、言語定義 (`Language`) の変更は必ず `shared` で行う
- 正誤判定はサーバー側に状態を持たない: ユーザーコード + `hidden_tests` を結合して実行し、
  **終了コードと stdout の目印**で分類する。判定順序と言語別の制約は ADR 0002 が正本
- `crates/app` は wasm32 専用ターゲット。純ロジック(フィルタ・進捗集計・出力パース)は
  host でもテストできるよう UI から分離して書く

## 既知の地雷

- **`Outcome::Passed` の必要条件に `test result: ok` が要る**。終了コードだけで正解にすると、
  ユーザーコードが判定テストより先に `exit(0)` するだけで「正解」になる (テストが 1 件も走らない)
- **`compose_submission` には必ず `Language` を渡す**。区切りコメントを `//` 固定にすると
  Python の問題が実行前に SyntaxError で全滅する
- **進捗は必ず `progress_key` / `&Problem` を取る関数を通す**。素の `p.id` で引いても型は通り、
  症状は「一覧の進捗色が静かに全部消える」だけなので気づけない
- **Java の問題でクラスを `public` にしない**。Wandbox のファイル名が `prog.java` 固定のため
  `class X is public, should be declared in a file named X.java` で落ちる
- **C# は `dotnetcore-6.0.425`**。`dotnetcore-8.0.402` は Wandbox 側で `dotnet new` が
  `File size limit exceeded` で落ちる。成功時も MSBuild の定型出力が出るのでプロキシで除去している
- **Wandbox は既定 User-Agent を 403 で弾く**。過負荷時は `OCI runtime error` を返すので、
  コンパイルエラーとして見せずに再試行 → 503 で返す
- Playground / Wandbox は非公式・レート制限あり。問題コンテンツの検証は必ず `verifier`
  (ローカル Docker) で行い、上流に一括負荷をかけない
- **verifier の検査を弱めない**。通らない問題は検査を消すのではなく問題を作り直す
- Vercel ビルドで `cargo install trunk` は遅すぎる。`scripts/build-frontend.sh` は
  prebuilt バイナリをダウンロードする
- リポジトリ直下に `build.sh` を置いてはならない: Rust ランタイムが関数ビルド前フックとして
  自動実行してしまう (2026-08-25 のデプロイ失敗の原因)
- `vercel-rust` (vercel-community/rust, `vercel_runtime 1.x`) は 2026-01 アーカイブ済み。
  使うと本番で `FUNCTION_INVOCATION_FAILED`。詳細:
  `docs/bootstrap/incidents/2026-08-25-vercel-rust-deploy-failures.md`
- Vercel build image は glibc が古い。prebuilt バイナリ (trunk 等) は musl 版を使う
- `crates/app` を `cargo test --workspace` に含めると wasm 前提コードが host ビルドで壊れることがある。
  テストは `--exclude app` で回し、app の純ロジックは shared 側に置く
- WSL で Playwright を使うときは `libnspr4` 等が不足する。sudo 不要の回避策は
  `apt-get download libnspr4 libnss3 libasound2t64` → `dpkg-deb -x` → `LD_LIBRARY_PATH`
