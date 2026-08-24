# 2026-08-25-initial-build: Rust 100本ノックを 0 から本番デプロイまで構築

セッション期間: `2026-08-25 00:10` 〜 `2026-08-25 03:00`
本 doc の目的: **次の Claude (= 別ターミナル / 翌日の自分 / 別 session) が cold restore できる状態**を残す。

---

## 1 行で言うと

> Leptos(wasm)+Vercel 公式 Rust ランタイム構成で 300 問 (初級/中級/上級 各100) 全問 verifier 検証済み、本番 https://rust-100-knocks.vercel.app にデプロイ完了。本番で `/api/execute` (拒否 400 / 正解 / コンパイルエラー) とブラウザ E2E (b001 を解いて ✓・進捗保存・絞り込み) を確認済み。

## 残課題

| 項目 | 状況 | 対応案 |
|---|---|---|
| 問題コンテンツの人間レビュー | 300 問は機械検証 (answer 通過 / starter 不通過) のみ。文章品質・難易度勾配は未レビュー | 各レベル数問ずつ読んで違和感があれば `data/problems/*.json` を直接編集 → `cargo run -p verifier` |
| モバイル実機 | 1100px 以下は縦積み (Playwright 900px 幅で確認済み)。タッチ操作・実機は未確認 | 実機で開いてエディタ入力とスクロールを確認 |
| Playground レート制限時の UX | 429 はメッセージ表示のみ。連打防止や自動リトライは無し | 必要なら実行ボタンにクールダウンを付ける |
| Vercel CLI 認証 | `vercel whoami` が `propagateaiwebcreation` スコープで Not authorized | `vercel login` し直すか、個人スコープに `vercel switch` |

## バックグラウンドプロセス

- `trunk serve` (port 8080) — 状態: セッション終了で停止して良い
- 問題生成 Workflow (`wf_091f72ab-141`) — 状態: completed (全 30 バッチ納品済み)

## 触ったファイル

### 永続化したい

全て commit/push 済み (main `5d95a99` 以降)。主要:

- `crates/shared/` — 問題スキーマ / 実行 API 契約 (Playground 互換) / 進捗モデルと絞り込み
- `crates/app/` — Leptos UI。`layout.rs` + `splitter.rs` が Zed 風ドラッグ分割、`console.rs` が rustc 出力の色分け
- `api/execute.rs` — 公式ランタイム (`service_fn` + hyper 1) の Playground プロキシ
- `crates/verifier/` — 問題品質ゲート。`--file/--level/--scratch` で単一ファイル検証可
- `data/problems/{beginner,intermediate,advanced}.json` — 各 100 問
- `scripts/build-frontend.sh` — Vercel 用フロントビルド (musl trunk)。**`build.sh` に改名禁止**
- `scripts/_merge-batches.mjs` — 生成バッチを data/ に統合 (id 連番検査つき)

### untracked / ephemeral

scratchpad (`/tmp/claude-1000/.../scratchpad`) 配下、消えて良い:

- `scripts/_shot.mjs` / `_drag.mjs` / `_prod.mjs` — Playwright スクショ / ドラッグ検証 / 本番 E2E
- `batches/*.json` — 生成バッチ (data/ に統合済み)
- `localdebs/` — WSL で headless Chromium を動かすための libnspr4/libnss3 (LD_LIBRARY_PATH で渡す)

## 重要な memory / docs references

1. `CLAUDE.md` — 構成・コマンド・**既知の地雷 (デプロイ関連 3 件)**
2. `docs/bootstrap/incidents/2026-08-25-vercel-rust-deploy-failures.md` — デプロイ 4 段階失敗の原因と修正
3. `docs/decisions/0001-playground-proxy-execution.md` — 実行エンジンを Playground プロキシにした理由
4. memory `project-rust-100-knocks` — ユーザー決定事項 (ロゴは歯車、DB なし 等)

## 検証手順

```bash
# 本番 API: 許可リスト拒否 → 400、正常 → 200 で {"success":true,...}
U=https://rust-100-knocks.vercel.app/api/execute
curl -s -o /dev/null -w "%{http_code}\n" -X POST $U -H 'content-type: application/json' \
  -d '{"channel":"evil","mode":"debug","edition":"2024","crateType":"lib","tests":true,"code":""}'
curl -s -X POST $U -H 'content-type: application/json' \
  -d '{"channel":"stable","mode":"debug","edition":"2024","crateType":"lib","tests":true,"code":"pub fn f() -> i32 { 1 }\n#[test]\nfn t() { assert_eq!(f(), 1); }"}'

# ローカル全テスト + 300 問検証
cargo test --workspace
cargo run -p verifier            # 期待: 検証 300 問 / 問題あり 0 件
```

期待: 400 / `{"success":true,...}` / 全 test ok / 問題あり 0 件

## 次セッションへの起動文 (= コピペ用)

```
docs/bootstrap/handoffs/2026-08-25-initial-build.md を読んで状況把握してから、
残課題の「/api/execute 本番動作」の検証手順を実行し、通っていれば本番 E2E (b001 を解く) に進んで。
```
