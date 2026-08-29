# 動作検証: feat/multi-language (WO-0001)

DoD の各行に外部オラクルを与えて記録する。OPEN が 1 行でも残っている間は統合しない。
オラクルの定義は WO-0001 の 9 節が正本。

最終更新: 2026-08-30 (全 2100 問完成・7 言語のブラウザ検証まで通過)

| # | 条件 | 状態 | 証拠 |
|---|---|---|---|
| D1 | shared が Language を公開・Problem に language・compose_submission が言語別区切り | **CLOSED** | `cargo test --workspace --exclude app` 103 passed。`compose_submission_uses_python_comment_syntax` を含む |
| D2 | /api/execute が 7 言語で実コンパイラ出力を返す | **CLOSED (デプロイ以外)** | `api/execute.rs` の `dispatch` を実上流に対して直接走らせる `#[ignore]` テスト 4 本。7 言語すべてで 正解→Passed / 未実装→Passed でない / 壊れたコード→CompileError + 言語固有の診断。残るは「Vercel ランタイム上で動くか」「Vercel から Wandbox へ egress が通るか」のみ |
| D3 | C# の MSBuild ノイズ除去、error/warning 行は残る | **CLOSED** | `csharp_noise_is_removed_entirely_on_success` / `csharp_diagnostics_survive_noise_removal` / `csharp_noise_removal_keeps_any_line_mentioning_error_or_warning` |
| D4 | 5 種の Outcome が 7 言語で正しく分類される | **CLOSED** | shared のテスト 41 件。固定値はすべて実測値。**統合レビューで 1 件の実バグを検出・修正**: cargo test が失敗時に stderr へ出す `error: test failed` を rustc 診断と誤認し、Rust のテスト失敗が全件「コンパイルエラー」と表示されていた |
| D5 | verifier の docker run がバッチあたり 2 回以内 | **CLOSED** | `a_whole_batch_needs_exactly_one_container` (7言語)、`container_count_does_not_grow_with_problem_count`。実行時も「コンテナ起動 1 回」と報告 |
| D6 | 21 ファイル × 各 100 問 | **CLOSED** | 21/21 ファイル・各 100 問。`ls data/problems/*/[abi]*.json` が 21、各ファイル 100 件 |
| D7 | 全 2100 問で answer 通過 / starter 失敗、件数が数値で確認できる | **CLOSED** | `verifier --expect 2100` → 「検証 2100 問 / 問題あり 0 件 / コンテナ起動 18 回」(26分30秒)、終了コード 0 |
| D8 | 既存 Rust 300 問が無変更 (移動と language 付与のみ) | **CLOSED (例外1件)** | `git diff --numstat` が 3 ファイルとも「追加 100 / 削除 0」。**ただし a006 は差し替えた** — 新設の重複検査が a001 との同一問題を検出したため (WO 11.5 節 E1 に記録) |
| D9 | 言語切替で一覧とエディタが追従、未収録言語は出ない | **CLOSED** | Playwright: セレクタの option が `["rust"]` のみ (他 6 言語は未収録)。レベル切替と問題選択も動作 |
| D10 | 進捗が言語ごとに独立、旧 Rust 進捗が失われない | **CLOSED** | shared 10 件 + app の進捗キーテスト 4 件 (`migrate_legacy_keys` の冪等性・二重移行しない・言語間で下書きが混ざらない) |
| D11 | trunk build --release 成功、dist に 21 個の問題 JSON | **CLOSED** | `trunk build --release` 成功、`dist/data/problems/*/*.json` が 21 個 |
| D12 | 実ブラウザで 7 言語それぞれ正解判定 | **CLOSED** | 7/7。各言語のスクリーンショット `D12-<lang>.png` |
| D13 | 実ブラウザで実診断が出て、エラー行が着色される | **CLOSED** | 7/7。rustc `error[E0308]` / gcc `prog.cc:2:13: error:` / Roslyn `error CS0029` / javac `prog.java:3: error:` / CPython `File "prog.py", line 1` / tsc `error TS2322` / Node `SyntaxError`。いずれも `.line-error` で着色 |
| D14 | 無作為抽出した問題が starter で不正解・answer で正解 | **CLOSED** | 21/21 (7 言語 × 3 問、seed 固定で再現可能) |
| D15 | preview デプロイでも 7 言語が実診断を返す | **OPEN (利用者の判断待ち)** | ブランチ push で preview は**ビルド成功** (`5b3dd13f`)。ただし Vercel の Deployment Protection が既定で有効なため、preview への HTTP は 401 (`Protected deployment`) になり自動検証できない。本番 (`rust-100-knocks.vercel.app`) は公開されている。下記「preview 検証の選択肢」参照 |
| D16 | 「テスト未実行で exit 0」が正解にならない | **CLOSED** | `exit_zero_without_ok_marker_is_not_passed` / `empty_output_with_exit_zero_is_not_passed` / app 側 1 件。加えて verifier に実コンテナで `sys.exit(0)` を投げて検出されることを実測 |
| D17 | 生成した問題に使い回しが無い | **CLOSED** | 全 2100 問で題名の重複 0・模範解答の重複 0 を機械確認。難易度をまたぐ重複の検査 (`validate_across_levels`) も追加し、既存 Rust の 2 組を検出して差し替えた |
| D18 | UI から Rust 固定の文言が消えている | **CLOSED** | `index.html` の grep 0 件。app 側の残りは規則を説明したコメントと `match backend()` の Rust 分岐のみ。スクリーンショットでブランドが「100本ノック」、stderr ラベルが「診断出力 (stderr)」であることを確認 |

## 残る穴 (正直な記録)

1. **`api/execute.rs` そのものは実行していない (D2 / D15)**
   `vercel dev` が別チームスコープ (`propagate-webcreation`) を要求し、この個人プロジェクトを
   会社チームにリンクする判断は利用者のもの。代わりに **同じ契約を実装したローカルサーバー**
   (`scratchpad/pw/server.mjs`) で 7 言語の経路を通した。
   Rust 側のプロキシは 11 件の単体テストがあり、判定・詰め替えロジックは `shared` にあって
   実 Wandbox 応答で検証済みだが、**Vercel 上で動く実バイナリは未検証**。
   このリポジトリで唯一起きた事故が「ローカルでは通る」本番専用障害だったので、
   ここは preview デプロイで塞ぐ必要がある。

2. **Wandbox の egress が Vercel から通るか未実測**
   Wandbox は既定 User-Agent を 403 で弾く。プロキシは明示 UA を送るが、
   Vercel の Function から wandbox.org に到達できるかは preview でしか確かめられない。

3. **ローカル Docker と Wandbox の版差**
   パッチ版が一致しない (gcc 13.4.0 / 13.2.0、python 3.13.15 / 3.13.8、node 20.20.2 / 20.17.0、
   dotnet 6.0.428 / 6.0.425)。判定契約のレベルでは 7 言語すべて一致することを実測したが、
   個々の問題が版差で挙動を変える可能性は残る。D14 の無作為抽出がこれを拾う網になっている。


## 統合レビュー (2026-08-29) で判明した実バグ

独立コンテキストの敵対的レビューが 14 件を指摘し、うち 3 件は**この時点で現に壊れていた**。

1. **Rust のテスト失敗が全件「コンパイルエラー」表示** — `cargo test` は失敗時に stderr へ
   `error: test failed, to rerun pass \`--lib\`` を出す。行頭が `error:` なので rustc 診断として
   拾われていた。利用者がいちばん頻繁に見る画面が壊れていた。
   実 Playground で再現 → `harness_ran()` 導入 → 両方向を再実測して修正確認。
2. **TypeScript のフラグが上流に届いていなかった** — Wandbox の `options` は選択肢 ID であって
   生フラグではなく、typescript には選択肢が無い。verifier だけ `--target es2020` を付けていたので、
   `Object.fromEntries` を使う模範解答が「ローカル緑・本番 TS2550」になる状態だった。実測で確認。
3. **「検査は最低 2 件」が常に 1 を数えていた** — ヘルパ関数内の定数 `"FAILED: "` を数えていたため。
   1800 問の唯一の品質ゲートが実質機能していなかった。

いずれも「テストは緑」「ローカルでは動く」状態で潜んでいた。実測とレビューの両方が要る種類のもの。


## preview 検証の選択肢 (D15)

ブランチ push → Vercel preview のビルドは **成功** した (Rust ランタイムの関数ビルドを含む)。
つまり「Vercel でビルドが通るか」は確認できている。残るのは
「Vercel の Function が実際に動くか」「Vercel から wandbox.org への egress が通るか」で、
これには preview への HTTP アクセスが要る。

preview は Vercel の Deployment Protection (既定で有効) により 401 を返す。取り得る道:

1. **Deployment Protection を preview だけ無効にする** — Project Settings → Deployment Protection →
   Vercel Authentication を Off。以後 push のたびに自動で検証できるようになる
2. **Protection Bypass for Automation を発行する** — 同じ画面で secret を作り、
   `x-vercel-protection-bypass` ヘッダで渡す。設定を公開側に緩めずに済む
3. **7 言語が揃ってから main にマージし、本番で検証する** — 設定変更は不要だが、
   実 Function の初回検証が本番になる。このリポジトリで唯一起きた事故が
   「ローカルでは通る」本番専用障害だったことを踏まえると、いちばん危ない道

検証ハーネスは `scratchpad/pw/verify-deploy.mjs` に用意済み。URL を渡せば
7 言語の「正解→Passed / 壊れたコード→CompileError + 言語固有の診断」を一度に確認する。


## 2026-08-30 — 完成時点の記録

全 2100 問 (7 言語 × 3 難易度 × 100 問) が揃い、D15 (本番デプロイでの疎通) を除く
すべての DoD がクローズした。

### 実測サマリ

| 検証 | 結果 |
|---|---|
| 全問の機械検証 | 2100 問 / 問題あり 0 件 / コンテナ起動 18 回 (26分30秒) |
| 重複 | 題名 0・模範解答 0 (難易度をまたぐものを含む) |
| ブラウザ (D14) | 21/21 (7 言語 × 無作為 3 問、seed 固定) |
| ブラウザ (D13) | 7/7 (各言語で実物の診断が出て着色される) |
| テスト | workspace 126 / app 51 / js 25 |
| 実上流への疎通 | 7 言語すべてで 正解→Passed / 未実装→Passed でない / 壊れたコード→CompileError |

### 検証ハーネス自身の欠陥 (2026-08-30 に発見)

ブラウザ検証で typescript/i065 だけが落ちたが、問題は正しく、**ハーネス側の欠陥**だった。
ブラウザ検証用にプロキシを JS で書き直しており、本番に入れた `compiler-option-raw`
(TypeScript の `--target es2020`) がハーネスに入っていなかった。

判定に関わるコードを二重に持つ限り再発するので、検証サーバーを
`tools/local_server.rs` に置き換え、`#[path = "../api/execute.rs"]` で
**本番のプロキシをそのまま取り込む**形にした。プロキシの実装は 1 つだけになった。

置き場所を `api/` にしなかったのは、Vercel の Rust ランタイムが `api/` 配下の `.rs` を
関数として自動検出するため (TCP リスナーがデプロイ対象に混入する)。
`.vercelignore` に `tools/` を入れるのも不可 — `Cargo.toml` の `[[bin]]` が
参照するファイルが消えてビルドが壊れる。

この乖離を捕まえる回帰テストを `tests/local_server.rs` に置いた
(`Object.fromEntries` は `--target es2020` が無いと TS2550 になることを利用)。
