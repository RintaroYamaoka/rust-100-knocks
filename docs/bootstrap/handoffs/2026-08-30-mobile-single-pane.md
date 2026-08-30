# 2026-08-30-mobile-single-pane: スマホ幅を 1 画面 1 ペインに作り替える

セッション期間: `2026-08-30 18:35` 〜 `2026-08-30 22:20`
本 doc の目的: **次の Claude (= 別ターミナル / 翌日の自分 / 別 session) が cold restore できる状態**を残す。

---

## 1 行で言うと

> スマホ幅 (≤820px と横向き) を「1 画面 1 ペイン + 下部タブ」に作り替え、本番
> <https://100-cord-knocks.vercel.app> で実測確認 (横スクロール 0 / b001 を解いて ✓正解 /
> page error 0)。ついでにヘッダーの進捗が初回デプロイからずっと `0 / 0` で固まっていた
> Leptos の購読バグを修正。3 commit を main に push 済み (`1feaab0` / `59e2a5f` / `4b5040f`)。

## 残課題

| 項目 | 状況 | 対応案 |
|---|---|---|
| iOS 実機での確認 | **未実施**。検証は Chromium のデバイスエミュレーションのみ。ソフトキーボード表示時の見え方 (`interactive-widget=resizes-content` は Chrome/Android のみ効く) と、ノッチの `env(safe-area-inset-*)` は実機でしか見えない | 実機 Safari で本番を開き、① エディタにフォーカス → 下部タブと「実行して判定」がキーボードに潰されないか ② 横向きでノッチに文字が潜らないか ③ 入力欄フォーカスで画面が拡大しないか、を見る |
| `Memo` 読み順バグの横展開 | ヘッダーの 1 箇所しか直していない。同じ形 (未計算の Memo を先に読み、同じクロージャでその材料も読む) が他に無いかは未確認 | `crates/app/src` で `.get()` と `.with(` を同一クロージャで読む箇所を洗い、**データ到着の前後で値が変わること**を実ブラウザで確認する (存在確認で止めない) |
| タブレット中間幅 (821〜1100px) | 従来の「3 ペイン縦積み + ページスクロール」のまま。スマホほど破綻していないが、一覧 300px → 問題 60vh → エディタ 90vh と積むので回遊性は低い | 1 画面 1 ペインの閾値を上げるか、2 ペイン (問題 \| コード) 構成を足すか。今回は触っていない |

## バックグラウンドプロセス

- `cargo run --bin local-server -- dist 8081` — log: セッション scratchpad — 状態: **killed** (停止済み)
- 本番デプロイ待ちの polling — 状態: completed (新ビルドの配信を確認済み)

## 触ったファイル

### 永続化したい (= すべて commit + push 済み)

- `crates/app/src/mobile.rs` (新規) — スマホで全画面表示するペインの状態。表示自体は CSS が持つ
- `crates/app/tests/mobile.rs` (新規) — slug/ラベル/遷移の固定 (CSS セレクタが slug に依存している)
- `assets/css/main.css` — 末尾の `mobile shell` ブロックが本体 (ヘッダー圧縮 / ペイン切替 / 折り返し / 下部タブ / 横向き)
- `crates/app/src/app.rs` — `LevelTabs` 部品化・`data-pane`・下部タブ・**進捗表示の読み順修正**
- `crates/app/src/console.rs` — コンソールの開閉 (スマホのみ効く)
- `crates/app/src/list.rs` — 一覧ペイン内のレベルタブ / 検索欄の `enterkeyhint`
- `crates/app/src/lib.rs` — 下部タブのアイコン SVG
- `index.html` — `viewport-fit=cover` + `interactive-widget=resizes-content`
- `CLAUDE.md`, `docs/bootstrap/incidents/2026-08-30-header-progress-frozen.md`

### untracked / ephemeral

- セッション scratchpad の Playwright ハーネス (`flow.mjs` / `regress.mjs` / `prod-verify.mjs`) — repo 外。
  **消えて良い**。再現手順は下の「検証手順」に書いてある

## 重要な memory / docs references

1. `CLAUDE.md` の「既知の地雷」 — スマホは 1 画面 1 ペイン / `Memo` と材料の読み順 / エディタ折り返しに必要な CSS / 入力要素を 16px 未満にしない
2. `docs/bootstrap/incidents/2026-08-30-header-progress-frozen.md` — 進捗が `0 / 0` で固まった原因と、すり抜けた理由
3. `docs/bootstrap/incidents/2026-08-30-language-selector-slow-network.md` — ブラウザ検証で `waitForTimeout` を使わない (今回もこの方針で測っている)
4. memory `reference-wsl-playwright-libs` — WSL で Chromium を起動する sudo 不要の手順

## 検証手順

```bash
# 1. ユニット
cargo test -p app && cargo test --workspace --exclude app && npm test

# 2. ローカルで実物を見る (別ターミナルで trunk build → local-server)
trunk build && cargo run --bin local-server -- dist 8081   # http://127.0.0.1:8081

# 3. 本番が新ビルドかどうか
curl -s https://100-cord-knocks.vercel.app/ | grep -c 'interactive-widget=resizes-content'
```

期待: テストは全 green / 手順 3 は `1`。

ブラウザ実測 (Playwright, iPhone 13 エミュレーション) で見るのは以下。**待ち時間ではなく値の変化で判定する**:

- `document.documentElement.scrollWidth - clientWidth === 0` (全幅で横スクロールなし)
- `.main` の `data-pane` が `list → problem → code` と切り替わる。言語・レベルの切替では**動かない**
- `.progress-count` がデータ到着前後で `0 / 0` → `0 / 100` に変わり、正解すると `1 / 100` になる
- `.cm-content` の computed が `16px / pre-wrap`
- デスクトップ 1440px で 3 ペインの寸法 (300 / 560 / 578, console 260) が従来どおり

## 次セッションへの起動文 (= コピペ用)

```
docs/bootstrap/handoffs/2026-08-30-mobile-single-pane.md を読んで状況把握してから、
残課題の「Memo 読み順バグの横展開」を進めて。
(iOS 実機確認は人間側のタスクなので、指示があるまで待つ)
```
