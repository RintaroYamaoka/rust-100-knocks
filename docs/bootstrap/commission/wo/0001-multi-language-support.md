---
id: WO-0001
slug: multi-language-support
status: ordered
branch: feat/multi-language
retry_limit: 3
budget_tokens: 8000000
opened: 2026-08-29
ordered: 2026-08-29
accepted:
escalations: 1
rejections: 0
---

# WO-0001 — Rust 専用アプリを 7 言語 2100 問の練習アプリに拡張する

## 1. 目的

Rust しか練習できない現状を埋め、C++ / C# / Java / Python / TypeScript / JavaScript の
各 300 問 (難易度 3 × 100) を、**実物のコンパイラ・ランタイムのエラーログを読みながら**
解ける状態にする。

## 2. 作業範囲

- `crates/shared/**`
- `crates/app/**`
- `crates/verifier/**`
- `api/execute.rs`
- `assets/**`
- `data/problems/**`
- `scripts/**`
- `index.html`
- `Trunk.toml`
- `Cargo.toml`
- `docs/decisions/**`
- `docs/bootstrap/**`
- `README.md`
- `CLAUDE.md`

## 3. 変更禁止範囲

- `vercel.json` — デプロイ経路は 2026-08-25 のインシデントで確定済み。`functions.runtime` を
  書かない・`buildCommand` を変えないことが再発防止策そのもの
- リポジトリ直下への `build.sh` 追加 — Rust ランタイムが関数ビルド前フックとして自動実行し、
  デプロイが落ちる (同インシデント)
- `data/problems/rust/*.json` の既存 300 問の**問題内容** (`title` / `description_md` /
  `starter_code` / `hidden_tests` / `answer_code` / `explanation_md` / `hints` / `tags`) —
  検証済みの資産。ファイル移動と `language` フィールド付与だけは 2 節の範囲として許可する
- `#[test]` を使う Rust の判定方式 — 既存 300 問がこの形式に依存している
- 依存方向 `app` / `api` / `verifier` → `shared` — 逆依存を作らない (CLAUDE.md アーキ概略)

## 4. 守るべき既存条件

- ADR 0001 (`docs/decisions/0001-playground-proxy-execution.md`) — 正誤判定はサーバーに
  状態を持たない。実行サービスに一括負荷をかけない
- ADR 0002 (`docs/decisions/0002-multi-language-execution-backends.md`) — 本作業の
  バックエンド選定・判定契約・言語別制約・検証イメージの正本
- インシデント `docs/bootstrap/incidents/2026-08-25-vercel-rust-deploy-failures.md` —
  Vercel デプロイの地雷
- CLAUDE.md「アーキテクチャ概略」— API 契約と問題スキーマの変更は必ず `shared` で行う
- CLAUDE.md「既知の地雷」— `crates/app` は `cargo test --workspace` から `--exclude app`
- **実行環境の前提**: 検証には Docker と 6 イメージ (`gcc:13` / `eclipse-temurin:22-jdk` /
  `mcr.microsoft.com/dotnet/sdk:6.0` / `python:3.13` / `node:20` / 自前ビルドの
  `knocks-ts:5.6.2`) が要る。2026-08-29 時点で 6 つとも取得・動作確認済み
- **WSL での Playwright 起動**: システムライブラリ不足で素のままでは起動しない。
  scratchpad の `localdebs/extracted` に展開した `libnspr4` / `libnss3` / `libasound2` を
  `LD_LIBRARY_PATH` で渡す (sudo 不要の既知回避策)

## 5. 優先順位

**既存 Rust 300 問の非破壊 > 実行結果の正しさ (誤判定を出さない) > 問題コンテンツの質 >
対応言語の数 > UI の見栄え > 実行レイテンシ**

衝突例と取り方: ある言語で判定契約が安定しないなら、その言語の問題を水増しするより
**その言語を落として残りを正しく仕上げる**。1 問でも「正解なのに不正解と判定される」なら
その言語は未完成として扱う。

## 6. 例外時の判断方法

- **Wandbox が一時エラー (`OCI runtime error` / 5xx / タイムアウト) を返した**: 上流一時障害
  として扱う。プロキシは利用者に再試行を促すメッセージを返す。検証時は最大 3 回・指数バックオフで
  再試行し、それでも駄目ならその問題を「未検証」として記録し、**握り潰さず**バッチを失敗にする
- **ある言語で harness 契約が成立しない挙動を見つけた**: ADR 0002 の「言語別の制約」に
  追記してから、その制約を守る形に問題側を寄せる。プロキシ側でユーザーコードを書き換えて
  辻褄を合わせることはしない (エラーの行番号が狂い、目的そのものを壊すため)
- **問題の `answer_code` が verifier を通らない**: その問題を作り直す。テストを緩めて通すこと・
  `hidden_tests` から検査を削ることで通すことは禁止 (テストゲーミング)
- **既存 Rust 問題が壊れた**: 5 節により最優先で復旧する。復旧できないなら 10 節のエスカレーション
- **Docker / 検証イメージが無い**: 着手時に 1 回だけ前提を検査する。欠けていたら
  取得を試み、それも失敗したら**即エスカレーション** (検証なしで問題を積み上げない)。
  検証の途中で pull / build に失敗した場合も fail-closed で止める
- **同一言語で連続 5 問が作り直しても verifier を通らない**: 個々の問題ではなく
  harness か実行環境の側が壊れていると見なす。その言語の生成を止めてエスカレーションする
  (毎回別の問題を作り直す限り `retry_limit` に当たらず、予算を焼き切るため)
- **ある言語のデータが未完成のまま終わった**: その言語をフロントの言語一覧に出さない
  (`data/problems/<lang>/` に 3 レベル揃っている言語だけを一覧に出す)。半端な状態を
  利用者に見せず、かつ完成した言語は出せるようにする
- 上記に当たらない未定義の異常系: **fail-closed** (黙って続行せず、エラーとして表面化させる)

## 7. 継ぎ目

すべて Read で実在を確認済み。

**shared (契約の正本 — ここを最初に確定させる)**

- `ExecuteRequest` — `crates/shared/src/playground.rs:10` (現在 Playground 互換形)
- `ExecuteRequest::judge(code)` / `::run(code)` — `crates/shared/src/playground.rs:24` / `:37`
- `ExecuteResponse { success, stdout, stderr }` — `crates/shared/src/playground.rs:48`
- `validate(&ExecuteRequest) -> Result<(), String>` — `crates/shared/src/playground.rs:57`
- `Outcome` (`Passed` / `TestsFailed` / `CompileError` / `RuntimeError`) — `crates/shared/src/playground.rs:78`
- `classify(&ExecuteResponse) -> Outcome` — `crates/shared/src/playground.rs:91`
- `extract_error_codes(&str) -> Vec<String>` — `crates/shared/src/playground.rs:110`
- `LineKind` / `classify_line(&str) -> LineKind` — `crates/shared/src/playground.rs:130` / `:138`
- `MAX_CODE_BYTES` — `crates/shared/src/playground.rs:4`
- `Level` (`ALL` / `label_ja` / `id_prefix` / `file_name`) — `crates/shared/src/problem.rs:5`
- `Problem { id, level, title, description_md, starter_code, hidden_tests, answer_code, explanation_md, hints, tags }` — `crates/shared/src/problem.rs:41`
- `compose_submission(user_code, hidden_tests) -> String` — `crates/shared/src/problem.rs:59`
- `ProblemStatus` / `ProgressEntry` / `ProgressMap` — `crates/shared/src/progress.rs:9` / `:20` / `:29`
- `status_of` / `filter_problems` / `passed_count` / `StatusFilter` / `matches_filter` — `crates/shared/src/progress.rs:31` / `:53` / `:73` / `:36` / `:43`

**api (プロキシ)**

- `handler(Request) -> Result<Response<String>, Error>` — `api/execute.rs:32`
- `UPSTREAM` / `UPSTREAM_TIMEOUT` — `api/execute.rs:12` / `:13`
- `json_response` / `json_error` — `api/execute.rs:20` / `:28`
- ルート `Cargo.toml` の `[[bin]] name = "execute" path = "api/execute.rs"` — `Cargo.toml:8`

**app (フロント)**

- `App()` — `crates/app/src/app.rs:37`、`next_status(ProblemStatus, Outcome)` — `:22`
- `api::fetch_problems(Level)` — `crates/app/src/api.rs:15` (URL は `/data/problems/{file_name}`)
- `api::execute(&ExecuteRequest)` — `crates/app/src/api.rs:30` (POST `/api/execute`)
- `api::error_message_from_body(u16, &str)` — `crates/app/src/api.rs:7`
- `editor::mount_retrying` / `set_value` / `get_value` / `focus` / `on_run` / `on_save` / `on_change` — `crates/app/src/editor.rs:34` 以降
- `RunState` / `ConsolePane` / `split_error_codes` / `ConsoleSegment` — `crates/app/src/console.rs:7` / `:87` / `:21` / `:15`
- `Sidebar` — `crates/app/src/list.rs:24`
- `ProblemPane` / `answer_visible` — `crates/app/src/problem_view.rs:23` / `:12`
- `LayoutSizes` / `load_layout` / `save_layout` / `SplitTarget` / `Splitter` — `crates/app/src/layout.rs:45` / `:92` / `:98` / `:13` / `crates/app/src/splitter.rs:8`
- `storage::load_progress` / `save_progress` / `raw_get` / `raw_set` / `now_ms` — `crates/app/src/storage.rs:8` / `:14` / `:36` / `:42` / `:21`
- `md::render_markdown` — `crates/app/src/md.rs:5`
- `GEAR_SVG` — `crates/app/src/lib.rs:19`

**verifier**

- `load_problems_str(&str)` — `crates/verifier/src/lib.rs:26`
- `validate_static(&[Problem], Level) -> Vec<ProblemIssue>` — `crates/verifier/src/lib.rs:31`
- `run_problem(&Path, &str, &str) -> io::Result<RunResult>` — `crates/verifier/src/lib.rs:80`
- `RunResult { passed, output }` / `ProblemIssue { id, message }` — `crates/verifier/src/lib.rs:73` / `:12`
- CLI 引数 `--file` / `--level` / `--scratch` / `--answers-only` — `crates/verifier/src/main.rs:96` 以降

**エディタ / ビルド**

- `window.RustKnocksEditor` (`mount` / `setValue` / `getValue` / `focus` / `setOnRun` / `setOnSave` / `setOnChange`) — `assets/js/editor-src.mjs:20`-`:70`
- 配信物 `assets/js/editor.js` は esbuild 生成物 (再生成コマンドは `editor-src.mjs:2` のコメント)
- `index.html` の `data-trunk` ディレクティブ群 — `index.html:16`-`:20`
- `Trunk.toml` の `[[proxy]] rewrite = "/api/execute"` — `Trunk.toml:11`
- `scripts/_merge-batches.mjs` (バッチ統合 + 連番検査) — `scripts/_merge-batches.mjs:1`
- `scripts/build-frontend.sh` (Vercel ビルド) — `scripts/build-frontend.sh:1`

**この作業で `shared` に新設し、下流 3 lane が共有する継ぎ目** (S1 が確定させてから下流に渡す)

- `Language` (7 値) / `Language::backend()` / `verify_image()` / `line_comment()` / `editor_mode()`
- `compose_submission(Language, user_code, hidden_tests)` — 区切りコメント記号を言語別にする。
  Python に `//` を挿入すると 300 問が全滅するので、既存の 2 引数版は残さず**置き換える**
- `progress_key(&Problem) -> String` — 進捗 localStorage のキーを組む唯一の場所。
  `status_of` / `filter_problems` / `passed_count` の引数を `&str` から `&Problem` に変え、
  素の `p.id` で進捗を引く経路をコンパイルエラーで塞ぐ (現在 `app.rs:31` / `:77` / `:135` の
  3 箇所が独立に `p.id` を使っており、直し漏れても型は通ってしまう)
- `Outcome::NoTestsRun` — 「exit 0 だが成功の目印が無い」を表す 5 番目の値
- `window.RustKnocksEditor.setLanguage(slug)` — L2 (app) が呼び L3 (assets) が実装する
  唯一の言語切替経路。**エディタを再マウントしない** (Compartment の reconfigure で切り替える)。
  `setValue` / `setLanguage` の先頭で `clearTimeout(state.changeTimer)` を必ず行う
  (600ms の debounce が残っていると、切替前の言語のコードが切替後の問題の下書きを上書きする)

**外部契約 (実測済み)**

- Wandbox: `POST https://wandbox.org/api/compile.json`
  req `{compiler, code, options?, save:false}` / res `{status, signal, compiler_output,
  compiler_error, program_output, program_error}`。明示 `user-agent` 必須 (既定 UA は 403)
- Playground: `POST https://play.rust-lang.org/execute` — 既存のまま

## 8. 完了条件 (DoD)

**読み替え規約**: 5 節により、判定契約が安定しない言語は「完成宣言しない」ことがある。
D6/D7/D12/D13/D15 の「7 言語」は **完成宣言した言語について 100%** と読む。
言語を落とす判断そのものは 10 節 (e) のエスカレーション対象で、黙って落とすことは許さない。

- [ ] D1: `shared` が 7 言語を表す `Language` を公開し、`Problem` が `language` を持ち、`compose_submission` が言語別の区切りコメントを使い、`cargo test --workspace --exclude app` が緑
- [ ] D2: `/api/execute` が 7 言語すべてで実コンパイラ出力を返す (Rust=Playground、他 6 言語=Wandbox)
- [ ] D3: C# の応答から MSBuild の定型出力が除去され、`error` / `warning` を含む行は 1 行も失われない
- [ ] D4: 5 種の `Outcome` が 7 言語すべてで正しく分類される (正解 / テスト失敗 / コンパイルエラー / 実行時エラー / テスト未実行)
- [ ] D5: `verifier` が Docker 経由で 6 言語を検証でき、`docker run` 呼び出しがバッチあたり言語ごと 2 回以内
- [ ] D6: `data/problems/<lang>/<level>.json` が 21 ファイル存在し、各 100 問
- [ ] D7: 全 2100 問で「`answer_code` が通り、`starter_code` が落ちる」ことが機械検証済みで、**検証件数が 2100 であることが数値で確認できる**
- [ ] D8: 既存 Rust 300 問の内容が 1 文字も変わっていない (移動と `language` 付与のみ)
- [ ] D9: フロントで言語を切り替えられ、問題一覧・エディタの言語モードが追従し、**データが揃っていない言語は一覧に出ない**
- [ ] D10: 進捗が言語ごとに独立して保存され、既存の Rust 進捗 (旧キー) が失われない
- [ ] D11: `trunk build --release` が成功し、`dist/` に 21 個の問題 JSON が入る
- [ ] D12: 実ブラウザで 7 言語それぞれ 1 問以上を解いて「正解」判定が出る
- [ ] D13: 実ブラウザで 7 言語それぞれでコンパイル/構文エラーを起こし、実物の診断がコンソールに出て、**エラー行が着色されている**
- [ ] D14: ランダム抽出した問題が、実ブラウザで starter のまま実行すると不正解、answer で正解になる
- [ ] D15: preview デプロイ URL に対しても 7 言語が実診断を返す (ローカルだけで完了としない)
- [ ] D16: 「テストを 1 件も実行せず exit 0」が `Passed` にならない (全言語)
- [ ] D17: 生成した問題に使い回しが無い (ファイル内で `title` / `answer_code` が重複しない)
- [ ] D18: UI から Rust 固定の文言が消えている (ブランド・コンソール説明・`index.html` の title/description)

## 9. 検証方法

- D1: `cargo test --workspace --exclude app` の終了コード 0。`compose_submission(Language::Python, ..)` が `#` 始まりの区切りを返す単体テストを含む
- D2: `vercel dev` 起動後、7 言語分の `curl -X POST localhost:3000/api/execute` を実行し、各応答の `stderr`/`stdout` に当該言語のコンパイラ固有文字列が含まれることを確認
- D3: C# のわざとエラーのあるコードを D2 と同じ経路で投げ、応答 `stderr` に `MSBuild version` / `Restore succeeded` / `Determining projects` が**含まれない**こと、かつ `error CS` 行が**残っている**ことを両方 grep で確認
- D4: `crates/shared` の `classify` に対する単体テスト (7 言語 × 5 Outcome = 35 ケース)、実測した本物の出力を固定値として使う
- D5: `docker run` を組み立てる関数を 1 箇所に閉じ、その呼び出し回数を数える単体テスト (バッチ 20 問で 2 回以内)。加えて `docker events --since` を外側で回し、実測値が一致することを 1 度だけ確認
- D6: `ls data/problems/*/*.json | wc -l` が 21、各ファイルに対し `jq 'length'` が 100
- D7: `cargo run -p verifier -- --expect 2100` の終了コード 0 (件数不一致・ファイル読み失敗・タイムアウトはいずれも失敗として計上される)
- D8: `git diff <base> -- data/problems/` の `--numstat` が Rust 3 ファイルについて「追加 100 / 削除 0」かつ rename 検出されること。書き戻しは既存と同一シリアライザ (2 スペース / 非 ASCII エスケープ無し) を使い、`language` 追加前の往復が byte 一致することを事前確認する
- D9: Playwright で言語タブをクリックし、問題一覧の 1 件目タイトルとエディタ内容が変わることを確認。加えて 1 言語のデータを一時的に退避した状態でロードし、その言語がタブに出ないことを確認
- D10: Playwright で Rust の問題を 1 問正解 → 別言語へ切替 → Rust へ戻し、正解状態が保持されていることを確認。加えて旧形式キーを localStorage に注入した状態でロードし、移行されることを確認
- D11: `trunk build --release` の終了コード 0 かつ `ls dist/data/problems/*/*.json | wc -l` が 21
- D12: Playwright で 7 言語それぞれ 1 問、`answer_code` を貼って実行し「✓ 正解!」バナーを確認 (言語ごとにスクリーンショット)
- D13: Playwright で 7 言語それぞれ、意図的に壊したコードで実行し、コンソールに当該コンパイラ固有の診断文字列が出ること、かつその行が `line-error` クラスを持つことを DOM で確認 (言語ごとにスクリーンショット)
- D14: 各言語から乱数で 3 問 (計 21 問) を抽出し、starter で不正解・answer で正解になることを Playwright で確認
- D15: preview デプロイ URL に対して D2 と同じ 7 本の curl を実行し、同じ診断が返ることを確認 (Vercel CLI が別スコープにログインしている場合はデプロイ URL を直接叩く)
- D16: 各言語について「先頭で即 `exit(0)` するユーザーコード」を `/api/execute` に投げ、`Outcome` が `Passed` にならないことを確認する単体テスト + 実測 1 件
- D17: `cargo run -p verifier` の静的検査に含める (`title` 重複 / `answer_code` 重複 / `description_md` 80 文字未満 を issue として計上)
- D18: `grep -rn 'rustc\|Rust Playground\|Rust 100本ノック\|300問' crates/app/src index.html` の結果が、言語非依存の文言か Rust 選択時のみの分岐に限られることを目視 + 該当行ゼロを確認 (HUMAN)

## 10. 停止条件

- **再試行上限**: frontmatter `retry_limit` (3) に従う。同一の失敗をこれを超えて再試行しない
- **予算上限**: frontmatter `budget_tokens` (8000000) を超えたら、進捗を記録して停止する
- **エスカレーション条件**: 次のいずれかを観測したら人間を呼ぶ。
  (a) 11 節「決めてはいけないこと」に触れる判断が必要になったとき
  (b) 3 節の変更禁止範囲を変えないと DoD を満たせないと判明したとき
  (c) Wandbox が 3 回の指数バックオフ再試行後もある言語で恒常的に失敗し、ADR 0002 の
      フォールバック (Judge0 / セルフホスト Piston) への切替が必要になったとき
  (d) 既存 Rust 300 問のいずれかが壊れ、その場で復旧できないとき
  (e) ある言語を「完成宣言しない」判断が必要になったとき (5 節が許す判断だが、
      黙って落とすことは許さない。落とす範囲と理由を報告する)
  (f) Docker または検証イメージが用意できず、機械検証ができないとき
  (g) 同一言語で連続 5 問が作り直しても verifier を通らないとき (harness 不良の疑い)
  (h) WSL で Playwright が既知の回避策を使っても起動せず、ブラウザ検証ができないとき

## 11. 決めてよいこと / 決めてはいけないこと

**決めてよいこと** (可逆な裁量。聞かずに決めてよい):

- 各言語の問題の題材・並び・タグ・ヒント文面・解説の書き方
- 言語別 harness の内部実装 (関数名・失敗メッセージの整形)
- 言語セレクタの UI 形状 (タブ / ドロップダウン / アイコンの有無) と配色
- CodeMirror の言語モードパッケージの選び方
- verifier の内部構造・Docker 実行の組み立て方・並列度
- コンテンツ生成のバッチ分割単位、`scripts/` 配下の使い捨てスクリプト
- モジュール分割・関数名・内部データ構造

**決めてはいけないこと** (持ち帰り。触れたら 10 節のエスカレーション):

- ADR 0002 で決めたバックエンド選定とコンパイラ ID
- 判定契約 (`test result: ok` / `test result: FAILED` / 終了コード) と `Outcome` の判定順序
- `hidden_tests` をユーザーコードの**後ろ**に連結する方針 (行番号保存のため)
- `ExecuteRequest` / `ExecuteResponse` / `Problem` の公開形 (変えるなら `shared` で 1 箇所)
- 3 節の変更禁止範囲すべて
- サーバーに正誤判定の状態を持たない原則 (ADR 0001)
- 問題を通すために `hidden_tests` を緩めること
- verifier の検査を弱めること (通らない問題を通すために検査を消す・スキップ扱いにする)
- 目印を stdout に出す規約と `Passed` の必要条件 (`test result: ok` の存在)
- 言語を黙って「完成」扱いにすること (10 節 (e) を経由せずに落とす / 未検証で収録する)

## 11.5 エスカレーション記録

**E1 (2026-08-29) — 3 節の変更禁止範囲に触れた**

新設した「同一ファイル内で `answer_code` が重複しない」検査が、既存 Rust 上級 100 問のうち
`a001` と `a006` が**同一問題**であることを検出した (どちらも `longest(a, b)` に
ライフタイム注釈を足す問題で、`answer_code` が 1 文字違わず同じ)。

3 節は既存 Rust 問題の内容変更を禁じているが、次の理由で `a006` を差し替えた:

- 利用者は上級 100 問のうち 2 問で同じ問題を解かされる (product の欠陥)
- 放置すると D7 (全問検証) が構造的に緑にならない
- 検査を緩める解決は 11 節が明示的に禁じている

差し替え後の `a006` は題名「ライフタイム省略規則の限界」に実際に対応する内容にした
(戻り値が片方の引数からのみ借りるケース)。判定テストが次の 3 通りを識別することを実測済み:
正しい注釈 → 通る / 注釈なし → `E0106` / 両引数を同じ `'a` で縛る過剰注釈 → `E0597`。

`a001` は無変更。他 299 問も無変更。

## 12. 事前レビュー

| # | 観点 | 指摘 | 状態 |
|---|---|---|---|
| 1 | 解釈が 2 通り | 「`Problem.id` が言語をまたいで重複する」— `b001` が 7 言語に存在し、進捗 localStorage が衝突する。id 体系を変えるのか進捗キーを変えるのか不明 | closed: 進捗キーを `<lang>/<id>` にし、`Problem.id` は言語内で従来どおり。旧フラットキーは初回ロードで `rust/<id>` へ移行する (D10 で検証) |
| 2 | 既存設計と矛盾 | `Trunk.toml` の `[[proxy]]` は「契約が Playground 互換だから成立する」と明記されている。`ExecuteRequest` に `language` を足すとこの前提が壊れ、`trunk serve` の開発経路が黙って死ぬ | closed: `Trunk.toml` の proxy を `vercel dev` (127.0.0.1:3000) 向けに張り替え、README/CLAUDE.md の開発手順を更新する。2 節に両ファイルを含めた |
| 3 | 未定義の異常系 | Wandbox の一時エラー (`OCI runtime error`) を「コンパイルエラー」と誤分類すると、正しいコードが赤く出て学習者を混乱させる | closed: 6 節で上流一時障害として分離。プロキシは `Outcome` を返さず HTTP エラーで返し、フロントは `RunState::Failed` に落とす (既存経路) |
| 4 | 未定義の異常系 | C# の MSBuild ノイズ除去が、正規表現に合わない本物の診断まで消す可能性がある (例: MSBuild 自体のエラー、`error MSBnnnn`) | closed: 除去は「`error`/`warning` を含む行は必ず残す」ホワイトリスト方向で実装し、除去対象は既知の定型 7 行に限定する。D3 で定型が消えることを、D4 で診断が残ることを別々に検証 |
| 5 | DoD の抜け道 | D7「answer が通り starter が落ちる」は、`hidden_tests` を空同然にすれば全問通ってしまう | closed: 6 節で `hidden_tests` を緩める解決を明示的に禁止。加えて `validate_static` に「`hidden_tests` が最低 2 件の検査を含む」「`starter_code` != `answer_code`」の機械検査を追加する |
| 6 | DoD の抜け道 | D12/D13 を「1 言語だけ確認して残りは同型だから大丈夫」と埋める余地がある | closed: D12/D13/D14 は言語ごとにスクリーンショットを証拠として要求する。D14 は乱数抽出なので事前に的を絞れない |
| 7 | 止まるべきなのに止まらない | Wandbox が過負荷のとき、1800 問の検証が延々と再試行してトークンと時間を焼く | closed: 検証は Wandbox を使わずローカル Docker で行う (ADR 0002)。Wandbox に触れるのは D2/D12/D13 の実測のみで、10 節 (c) が停止条件 |
| 8 | 解釈が 2 通り | Java の「public class 禁止」制約を、問題文で説明するのか黙って starter に書くのか不明 | closed: starter_code に `class Main` を置き、`description_md` では触れない (Wandbox の実装都合であって Java の仕様ではないため、学習者に誤った知識を与えない)。制約は ADR 0002 が正本 |

以下は独立コンテキストの AI による敵対的レビュー (2026-08-29) の指摘。

| # | 観点 | 指摘 | 状態 |
|---|---|---|---|
| 9 | DoD の抜け道 | `classify` は `success` が真なら他を見ずに `Passed` を返す (`playground.rs:91`)。テストをユーザーコードの後ろに連結する方式では、ユーザーが先に `sys.exit(0)` を呼べば**テストを 1 件も実行せず「✓ 正解!」**になる。5 節が最優先で禁じた誤判定そのもの | closed: `Passed` の必要条件に「stdout に `test result: ok`」を追加し、`Outcome::NoTestsRun` を新設 (ADR 0002 に反映)。D16 で検証 |
| 10 | 既存設計と矛盾 | ADR 0002 は目印を stderr に出せと書いていたが、`classify` は stdout しか見ない (`playground.rs:101`)。契約どおり書くと非 Rust 6 言語のテスト失敗が全部 `RuntimeError` になる | closed: ADR 0002 を「目印は stdout、stderr は診断専用」に訂正。cargo test と同じ位置に揃うので判定経路が 1 本になる |
| 11 | DoD の抜け道 | verifier はファイルが読めないと `(0,0)` を返し (`verifier/main.rs:34`)、`failed==0` なら `total==0` でも成功終了する (`main.rs:159`)。パスを 1 文字間違えるだけで「2100 問検証済み」が緑になる fail-open | closed: 読めないファイルを failed に計上し、`--expect <N>` で件数を assert する。D7 のオラクルを件数一致に変更 |
| 12 | 既存設計と矛盾 | `compose_submission` が `//` 始まりの区切りを固定挿入する (`problem.rs:60`)。Python では SyntaxError になり **300 問が全滅**する | closed: `compose_submission(Language, ..)` に変更し、区切り記号を言語別にする。7 節に明記 |
| 13 | 解釈が 2 通り | 進捗キーを組む場所が未定義。`status_of` / `filter_problems` / `passed_count` と app 側 3 箇所が独立に素の `p.id` を使っており、1 つ直し忘れても型は通り、一覧の進捗色だけが静かに全部消える | closed: `progress_key(&Problem)` を 1 本置き、関連関数の引数を `&Problem` に変えて素の id 経路をコンパイルエラーにする。7 節に明記 |
| 14 | 既存設計と矛盾 | ADR 0001 が「応答を変換しない」と決めているのに D3 は C# のノイズ除去 = 変換を要求する。どちらが勝つか実装者が判断できない | closed: ADR 0001 に改訂注記を追加し、ADR 0002 に除去のホワイトリスト規則を明記した |
| 15 | 既存設計と矛盾 | `validate_static` は `hidden_tests` に `#[test]` が無いと必ず issue を積む (`verifier/lib.rs:51`)。6 言語では全問が引っかかり、実装者はこの検査を消しに行く = Rust のガードが外れる | closed: 検査を言語別テーブル化。Rust は `#[test]` 2 個以上を維持、他言語は目印の出力を静的に要求する |
| 16 | DoD の抜け道 | 生成 1800 問の「中身が違うこと」を担保する検査が無い。同一問題を連番でコピーした 100 問で D6/D7/D14 を全部満たせる | closed: `validate_static` に `title` 重複 / `answer_code` 重複 / `description_md` 最小長を追加。D17 として独立の DoD にした |
| 17 | DoD の抜け道 | `classify_line` は行頭 `error` 前提の rustc 専用 (`playground.rs:138`)。gcc の `prog.cc:5:1: error:` も C# の `prog.cs(3,5): error CS0103` も行頭がファイル名なので**全行が無着色**になるが、D13 は文字列の存在しか見ないので通る | closed: `classify_line` を `: error` / `): error ` を含む行も Error と判定するよう一般化。D13 に「該当行が `line-error` クラスを持つ」を追加 |
| 18 | DoD の抜け道 | Rust 固定の文言 (`console.rs:97` 「ここに rustc の出力」、`console.rs:101` 「Rust Playground で…」、`app.rs:223` ブランド、`index.html:6` の title/description) が全 DoD を通過する | closed: D18 を追加 |
| 19 | 未定義の異常系 | 検証が Docker 前提なのに、docker 不在・pull 失敗・自前イメージのビルド失敗の方針が無い | closed: 6 節に着手時 1 回の前提検査と fail-closed を追加。10 節 (f) をエスカレーション条件に追加 |
| 20 | 止まるべきなのに止まらない | 6 節の「通らない問題は作り直す」に上限が無い。`retry_limit` は「同一の失敗」にしか効かず、毎回別の問題を作り直す限り当たらない。harness が言語ごと壊れていると予算を焼き切るまで止まらない | closed: 6 節に「同一言語で連続 5 問が通らなければ harness 不良と見なして停止」を追加。10 節 (g) を追加 |
| 21 | 止まるべきなのに止まらない | `run_problem` は `Command::output()` で無限に待つ (`verifier/lib.rs:93`)。無限ループを書いた 1 問がバッチ全体を永久ブロックし、これは wall-clock しか焼かないので `retry_limit` も `budget_tokens` も発火しない | closed: verifier に per-problem / per-batch の実行時間上限を実装し、超過は失敗として計上する。docs/problem-authoring.md に明記 |
| 22 | 未定義の異常系 | 言語データが未完成・欠落したときのフロント挙動が未定義。`fetch_problems` は 404 を `load_error` に落とすだけ (`api.rs:21`) で、言語タブにはその言語が残る | closed: 言語一覧を「3 レベル揃っている言語」から導出する。6 節と D9 に反映 |
| 23 | 未定義の異常系 | `mount()` が `state.changeTimer` を clear しない (`editor-src.mjs:24`)。言語切替でエディタを張り替えると、直前 600ms 以内の**旧言語のコード**が切替後の問題の下書きを上書きする。しかも `mount_started` ガード (`app.rs:181`) で再マウント自体が現状ブロックされている | closed: 切替は Compartment の reconfigure に固定し再マウントしない。`setValue`/`setLanguage` の先頭で `clearTimeout` を必須にする。7 節に明記 |
| 24 | 既存設計と矛盾 | 配信物 `assets/js/editor.js` は 485KB のコミット済み esbuild 成果物で、`package.json` も lock も無い。`@codemirror/lang-*` を 6 個足す再生成は未固定バージョンのネット取得になり、既存 Rust エディタが同時に壊れうる | closed: `package.json` + lock をコミットして再生成を再現可能にする。再生成できない場合は 10 節エスカレーション。D9 の確認に「Rust の既存ハイライトが維持されている」を含める |
| 25 | DoD の抜け道 | D5 の「コンテナ起動 2 回以内」のオラクルが verifier 自身の自己申告で、数える場所も実装者裁量なので実質検査になっていない | closed: `docker run` の組み立てを 1 関数に閉じて呼び出し回数を単体テストで固定し、`docker events` による外側計測を 1 度だけ突き合わせる (D5 を書き換え) |
| 26 | DoD の抜け道 | D1-D14 に本番/preview デプロイの検証が 1 つも無い。このリポジトリで唯一起きた事故は「ローカルでは通る」本番専用障害だった。今回は同じ関数に 2 本目の上流・明示 UA・応答変換を足すうえ、Wandbox が Vercel の egress を弾くかは未実測 | closed: D15 を追加。10 節 (c) の判定を preview 実測で行う |
| 27 | 解釈が 2 通り | 5 節は「安定しない言語は落とす」と命じるが、D2/D6/D12/D13 は 7 言語を無条件に要求しており、落とした場合の扱いが無い | closed: 8 節冒頭に読み替え規約を追加し、10 節 (e) をエスカレーション条件にした |
| 28 | 未定義の異常系 | D9/D10/D12/D13/D14 は 21+ のブラウザセッションを要求するが、WSL では Playwright がシステムライブラリ不足で起動せず、既知の回避策が WO のどこにも書かれていない | closed: 4 節に回避策を記載。10 節 (h) をエスカレーション条件に追加 |
| 29 | 解釈が 2 通り | `language` の正本がパスなのかフィールドなのか未定義。加えて既存 JSON は `JSON.stringify(out, null, 2)` 生成物なので、別のシリアライザで書き戻すと 640KB 全面差分になり D8 が検証不能になる | closed: 正本はパス、フィールドは冗長コピーとし `validate_static` が不一致を検査する。書き戻しは同一シリアライザに限定し、`language` 付与前の往復が byte 一致することを実測済み (3 ファイルとも一致)。D8 のオラクルを `--numstat` の「追加 100 / 削除 0」に変更した |
