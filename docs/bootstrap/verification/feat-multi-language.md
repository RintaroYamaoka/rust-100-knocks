# 動作検証: feat/multi-language (WO-0001)

DoD の各行に外部オラクルを与えて記録する。OPEN が 1 行でも残っている間は統合しない。
オラクルの定義は WO-0001 の 9 節が正本。

最終更新: 2026-08-29

| # | 条件 | 状態 | 証拠 |
|---|---|---|---|
| D1 | shared が Language を公開・Problem に language・compose_submission が言語別区切り | **CLOSED** | `cargo test --workspace --exclude app` 103 passed。`compose_submission_uses_python_comment_syntax` を含む |
| D2 | /api/execute が 7 言語で実コンパイラ出力を返す | **PARTIAL** | ローカル同等サーバー経由で Rust=Playground / 6言語=Wandbox の実診断を確認。**Vercel Function 本体 (`api/execute.rs`) は未実測** — 下記「残る穴」参照 |
| D3 | C# の MSBuild ノイズ除去、error/warning 行は残る | **CLOSED** | `csharp_noise_is_removed_entirely_on_success` / `csharp_diagnostics_survive_noise_removal` / `csharp_noise_removal_keeps_any_line_mentioning_error_or_warning` |
| D4 | 5 種の Outcome が 7 言語で正しく分類される | **CLOSED** | shared のテスト 33 件。固定値は 2026-08-29 に Playground / Wandbox から実測した本物の出力 |
| D5 | verifier の docker run がバッチあたり 2 回以内 | **CLOSED** | `a_whole_batch_needs_exactly_one_container` (7言語)、`container_count_does_not_grow_with_problem_count`。実行時も「コンテナ起動 1 回」と報告 |
| D6 | 21 ファイル × 各 100 問 | **OPEN** | コンテンツ生成中 |
| D7 | 全 2100 問で answer 通過 / starter 失敗、件数が数値で確認できる | **OPEN** | Rust 300 問は `--expect 300` で緑。残り 1800 問は生成中 |
| D8 | 既存 Rust 300 問が無変更 (移動と language 付与のみ) | **CLOSED (例外1件)** | `git diff --numstat` が 3 ファイルとも「追加 100 / 削除 0」。**ただし a006 は差し替えた** — 新設の重複検査が a001 との同一問題を検出したため (WO 11.5 節 E1 に記録) |
| D9 | 言語切替で一覧とエディタが追従、未収録言語は出ない | **CLOSED** | Playwright: セレクタの option が `["rust"]` のみ (他 6 言語は未収録)。レベル切替と問題選択も動作 |
| D10 | 進捗が言語ごとに独立、旧 Rust 進捗が失われない | **CLOSED** | shared 10 件 + app の進捗キーテスト 4 件 (`migrate_legacy_keys` の冪等性・二重移行しない・言語間で下書きが混ざらない) |
| D11 | trunk build --release 成功、dist に 21 個の問題 JSON | **PARTIAL** | ビルドは成功。JSON は現在 3 個 (Rust のみ) — コンテンツ生成待ち |
| D12 | 実ブラウザで 7 言語それぞれ正解判定 | **PARTIAL** | Rust 完了 (スクリーンショット `D12-rust.png`)。残り 6 言語はデータ待ち |
| D13 | 実ブラウザで実診断が出て、エラー行が着色される | **PARTIAL** | Rust 完了。`error[E0308]` が赤・`-->` が青・E0308 がリンク (`D13-rust.png`)、`.line-error` 6 行 |
| D14 | 無作為抽出した問題が starter で不正解・answer で正解 | **PARTIAL** | Rust 3/3 (b094 / i039 / a093、seed 固定で再現可能)。残り 6 言語はデータ待ち |
| D15 | preview デプロイでも 7 言語が実診断を返す | **OPEN** | Vercel CLI が別チームスコープ (`propagate-webcreation`) を要求する。スコープの選択は利用者の判断 |
| D16 | 「テスト未実行で exit 0」が正解にならない | **CLOSED** | `exit_zero_without_ok_marker_is_not_passed` / `empty_output_with_exit_zero_is_not_passed` / app 側 1 件。加えて verifier に実コンテナで `sys.exit(0)` を投げて検出されることを実測 |
| D17 | 生成した問題に使い回しが無い | **CLOSED (機構)** | `validate_static` の title / answer_code 重複検査 + `merge-batches` の同検査 (テスト 10 件)。実データへの適用は D7 と同時 |
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
