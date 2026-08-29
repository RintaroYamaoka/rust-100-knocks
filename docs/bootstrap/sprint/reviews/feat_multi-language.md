# 統合前レビュー: feat/multi-language

日付: 2026-08-30 (再判定)
対象: origin/main..feat/multi-language (28 コミット、HEAD = `78d396b`)

verdict: approve

理由: blocker だった R1 (切り戻しで進捗が全滅する) は、進捗キーを v2 に上げ、v1 を
**読むだけで書かない**構成に変わったことで実際に塞がっている (書き込み経路は
`crates/app/src/storage.rs:80` の 1 本だけで、そこは `PROGRESS_KEY` = v2 固定。
`LEGACY_PROGRESS_KEY` は `storage.rs:43` の読み出しにしか現れない)。R2 の衝突解決も
両方向で成立する — main 側も `updated_at_ms` を毎回打刻している
(`git show origin/main:crates/app/src/app.rs:87,148`) ので、比較材料が実在する。
残りは軽微 4 件だけで、本番デプロイを壊す/データを壊す指摘は残っていない。

なおレビュー中に `78d396b` (v2 に旧フラットキーを持ち込まない) が追加された。
これも読んで検証済み (下記 N1)。作業ツリーは clean。

## 前回指摘の解消状況

| # | 指摘 | 状態 | 根拠 (file:line) |
|---|---|---|---|
| R1 | 切り戻すと全利用者の進捗が消える (blocker) | **解消** | `crates/app/src/storage.rs:14` が `rust100knocks.progress.v2`、`:18` が v1 を「読むだけ」と定義。書き込みは `storage.rs:78-83` の `save_progress` だけで、宛先は `PROGRESS_KEY` (v2)。`grep PROGRESS_KEY crates/` で v1 への `raw_set` は 0 件。`crates/shared/src/progress.rs:64-83` の `migrate_legacy_keys` も `remove` をやめて `insert` (複製) のみ。机上シナリオは下記「R1 の 4 ステップ追跡」参照 |
| R2 | 戻して再前進すると、戻していた間の進捗が捨てられる | **解消** | `crates/shared/src/progress.rs:74-80` が `existing.updated_at_ms >= entry.updated_at_ms` のときだけ据え置き、それ以外は上書き。v1→v2 の取り込み側 (`crates/app/src/storage.rs:43-50`) も同じ比較。旧挙動を固定していたテストは書き換え済み (`crates/shared/tests/progress.rs:73-108`)、新規に双方向のテスト 2 件 (`同 :183-215`)。main 側が打刻していることも確認 (`origin/main:crates/app/src/app.rs:87,148`) |
| R3 | 2 つ目の `[[bin]]` を含む構成の Vercel ビルド未検証 | **解消 (既出扱い)** | `0951e76` の preview ビルド成功で確認済み。指摘としては再掲しない (ただし N5 参照) |
| R4 | `.vercelignore` に `_batches` が無い | **解消** | `.vercelignore:11` に `data/problems/*/_batches`。`git ls-files 'data/problems/**/_batches/*'` = 0 も維持 |
| R5 | `nav()` が素の id で現在位置を探す | **解消** | `crates/app/src/app.rs:258,261` が `progress_key` 比較。app 配下に残る `.id` 参照は表示用の 2 箇所のみ (`crates/app/src/list.rs:90`、`crates/app/src/problem_view.rs:140`)。開示状態も `progress_key` (`problem_view.rs:89`)、一覧の選択判定も同様 (`app.rs:304`) |
| R6 | 読み込み中の状態が無く、切替直後の UI が嘘をつく | **ほぼ解消 (残り N2)** | `crates/app/src/app.rs:63,112,122` に `loading`、一覧の文言分岐が `crates/app/src/list.rs:108-115`、実行ボタンの無効化が `app.rs:405-411`。ただし Ctrl+Enter 経路は無効化されていない (N2) |
| R7 | HEAD の存在確認が瞬断で「無い」に倒れ再試行しない | **解消** | `crates/app/src/api.rs:45-53`。`send()` が `Err` のときだけ 1 回だけ作り直して再送。ループ無し・多重送信無し (HEAD なので副作用も無い)。逐次のままだが、前回の対処案が「並列化**または**再試行」だったので契約は満たしている |
| R8 | `load_error` をフェッチ開始時にクリアしていない | **解消 (残り N3)** | `crates/app/src/app.rs:111` で `spawn_local` 直前に `set(None)`。ただし破棄されたフェッチの遅れて来るエラーは残る (N3) |
| R9 | `editor.js` に内容ハッシュが無い | **解消** | `index.html:27` が `?v=2d2ae2d3`。`md5sum assets/js/editor.js` = `2d2ae2d300a2…` で一致、`dist/assets/js/editor.js` も同一ハッシュ、`dist/index.html:38` にクエリが残ることを実物で確認。`package.json:7` の `build:editor` が再生成時に md5 を表示する。ハッシュの自動照合は無い (N4) |
| R10 | 移行時の保存失敗を握り潰している | **解消** | `crates/app/src/storage.rs:40,53-55` が `(ProgressMap, bool)` を返し、`crates/app/src/app.rs:56,67` が `storage_failed` の初期値に載せる。ヘッダの警告表示 (`app.rs:369-379`) に繋がっている |
| R11 | ADR 0002 の自己矛盾 | **解消** | `docs/decisions/0002-multi-language-execution-backends.md:84-86` がホワイトリスト方向に統一され、`:119-123` を正本と明示。実装 (`crates/shared/src/playground.rs`) と一致 |

### R1 の 4 ステップ追跡 (実コードで机上検証)

1. **前の版で b001 を解く** — main は v1 (`rust100knocks.progress.v1`) に
   `{"b001": {status:passed, updated_at_ms:T1}}` を書く (`origin/main:crates/app/src/storage.rs:6,14-18`)。
2. **新しい版を開く** — `load_progress_migrated` (`crates/app/src/storage.rs:40-56`) が
   v2 を読み (空)、v1 を読んで取り込み、`migrate_legacy_keys` で `rust/b001` を**複製**し、
   `save_progress` で **v2 にだけ**書く。**v1 は無傷**。
   以後の下書き/判定も v2 のみ (`app.rs:155,235` → `storage.rs:80`)。
3. **main に切り戻す** — main は v1 を読む (`origin/main:crates/app/src/storage.rs:8-12`)。
   `b001` がそのまま残っているので ✓ マークも下書きも見える。**進捗 0% にはならない** (= R1 解消)。
   なお「新しい版で解いた分」は v1 に無いので main では見えないが、v2 に保存されたままで
   失われてはいない (前進すれば戻る)。破壊ではなく可視性の後退。
4. **また新しい版に戻す** — v2 の `rust/b001` (T1) と、切り戻し中に main が更新した
   v1 の `b001` (T2 > T1) を突き合わせ、`storage.rs:44-49` → `progress.rs:74-80` の
   二段の `updated_at_ms` 比較で新しい T2 を採る。**切り戻し中の進捗は残る**。
   同値なら移行済みを優先するので、再保存が毎回走る (書き込みループ) こともない。

## 新規指摘

| # | 観点 | 指摘 (根拠 file:line) | 深刻度 | 対処 |
|---|---|---|---|---|
| N1 | 3 データ | **`78d396b` (`map.retain` によるフラットキー除去) は妥当。指摘なし。** 検証内容は下記「N1' 」。レビュー中の追加コミットなので、記録として表に残す。 | 情報 | 対処不要 |
| N2 | 5 正しさ | **Ctrl+Enter は `loading` を見ない。** 実行ボタンは `app.rs:410` で無効化されるが、キーボード経路は `app.rs:286` の `editor::on_run(run)` から `run` (`app.rs:208-215`) を直に呼び、ガードは `run_state` と `editor_ready` だけ。取得中に Ctrl+Enter を押すと、R6 が指摘したとおり「新しい言語を表示しながら前の言語の問題を実行する」状態が残る。進捗は `p.language` 由来のキーに正しく入るのでデータは壊れない。 | minor | `run` の先頭で `loading.get_untracked()` を見て早期 return する |
| N3 | 5 正しさ | **破棄されたフェッチのエラーが、別の言語の画面に出る。** `app.rs:103-124` の Effect は世代 (key) を持たない。Java の取得中に Rust (キャッシュ済み) へ切り替えると Effect は `:105-108` で早期 return し、その後 Java の `spawn_local` が失敗すると `:120` が `load_error` を立てる。`list.rs:76-79` はそれを現在の一覧の上に出すので、Rust を正常表示しながら Java のエラー文が出る。 | minor | `spawn_local` の中で `key == (language.get_untracked(), level.get_untracked())` を確かめてから `load_error`/`loading` を触る |
| N4 | 2 本番差 | **`?v=` の更新は人手頼み。** `index.html:27` のハッシュとバンドルの実 md5 を照合する自動検査が無い (`npm test` = `tests/editor-src.test.mjs` + `scripts/merge-batches.test.mjs` の 25 件に該当なし)。今は一致しているので実害は無いが、次にバンドルを作り直して `?v=` を忘れると R9 がそのまま再発する (症状は「7 言語すべてが Rust のハイライト」)。 | minor | `npm test` に「`index.html` の `?v=` が `assets/js/editor.js` の md5 先頭 8 桁と一致する」1 件を足す |
| N5 | 契約文書 | **検証記録の preview ビルドの根拠が古い。** `docs/bootstrap/verification/feat-multi-language.md:24` と `:69` は今も `5b3dd13f` を挙げており、R3 の対処案「現 HEAD で preview を押し、ビルド成功を D15 の証拠に追記する」の追記部分が入っていない。ビルド自体は `0951e76` で成功しているので事実関係は解決済み、記録だけが遅れている。 | minor | D15 の行に `0951e76` の preview 成功を追記する |

### N1' `78d396b` (`map.retain`) の検証

レビュー中に追加されたコミットなので、こちらも読んで確かめた。**問題なし**:

- 目的は、レビュー依頼にあった「v1 と v2 の二重書き込みでクォータが倍にならないか」。
  `8994369` のままだと、v1 由来のエントリが v2 の中に
  フラット (`b001`) と名前空間つき (`rust/b001`) の 2 通りで残り続ける。
  ただし**これは倍加ではない**: 新しい版は v1 に一切書かないので (`storage.rs:80`)、
  2100 問分の下書きは v2 に 1 部だけ。重複するのは多言語化前の Rust ≤300 問分に限られる。
  したがって `8994369` のままでも blocker ではなく、`retain` は正しい節約。
- `retain` 版の 4 ステップも追跡した。`retain` 後は v2 にフラットキーが無いので、
  起動のたびに v1 のエントリが `storage.rs:43-49` で必ず一度 map に入り、その直後に
  `migrate_legacy_keys` が `updated_at_ms` で `rust/…` と突き合わせる。
  勝敗は `8994369` 版と同一 (新しい方が勝つ)、同値なら `migrated = 0` で保存もしないので
  毎起動の書き込みループにもならない。切り戻し可否も変わらない (v1 は依然として無傷)。
- テストも追随している (`crates/app/tests/storage.rs` の `v2_holds_only_namespaced_keys`)。

## 確認した点

- **テストは緑**: `cargo test --workspace --exclude app` (全スイート 0 failed)、
  `cargo test -p app` (storage 7 件を含め 0 failed)、`npm test` (0 failed) を実測。
  実行時点のツリーは `78d396b` と同一内容 (= N1 の差分込み)。
- **進捗の書き込み経路は 1 本**: `crates/app/src/storage.rs` 以外に localStorage の
  進捗キーを触るコードは無い (`grep -rn "PROGRESS_KEY\|progress.v1" crates/`)。
  v1 に `raw_set` する経路は 0 件で、切り戻し用のデータが上書きされることはない。
- **`load_progress_migrated` の戻り値変更の波及**: 呼び出し側は
  `crates/app/src/app.rs:56` の 1 箇所だけで、第 2 要素は `storage_failed:67` に接続済み。
  取りこぼしは無い (テスト側 `crates/app/tests/storage.rs` は明示的に第 2 要素を検査)。
- **`loading` シグナルの波及**: 参照は `app.rs:63,112,122,395,410` と
  `list.rs:33,110` のみ。一覧は「読み込み中」と「該当なし」を出し分け (`list.rs:108-115`)、
  実行ボタンは取得中だけ無効化される。押せなくなるのは取得中のみで、
  失敗時 (`app.rs:120`) も `:122` で必ず `false` に戻るので固まらない。
  残る穴はキーボード経路 (N2) と、別言語のフェッチ中にボタンが数秒無効になる程度。
- **HEAD の再試行は有限**: `api.rs:45-53` は `match` の `Err` 分岐で 1 回だけ作り直して
  送る形で、再帰もループも無い。最大 2 回 / (言語, レベル) で、HEAD なので二重送信の
  副作用も無い。全滅時の起動プローブ時間が最悪 2 倍になるのが唯一の代償。
- **`updated_at_ms` の供給元**: main も新版も、下書き保存と判定の両方で `now_ms()` を
  打刻する (`origin/main:crates/app/src/app.rs:87,148` / `app.rs:153,233`)。
  R2 の比較が「片側が常に 0.0」で無意味になる、という失敗はしていない。
  (クライアント時計が巻き戻る環境では判定が狂うが、これは方式に内在するもの)
- **`editor.js` は同期している**: `assets/js/editor.js` と `dist/assets/js/editor.js` の
  md5 が一致 (`2d2ae2d3…`)、`index.html` / `dist/index.html` の `?v=` も同値。
  `setLanguage` を含むことも確認。
- **既出として扱った未解決事項**: D15 (実 Function の疎通は本番でしか確認できない)、
  Vercel → wandbox.org の egress 未確認、ローカル Docker と Wandbox のパッチ版差。
  これらは `docs/bootstrap/verification/feat-multi-language.md:24,31-42,67-88` に記録済み。
- **前回の「確認した点」で挙げた事項 (データ 2100 問の健全性・データ形式の切り戻し安全性・
  デプロイ設定の無変更) は、この差分では触られていない** — `8994369` の 12 ファイルと
  `78d396b` の 2 ファイルはすべてフロント/契約/文書で、`data/`・`api/`・`vercel.json`・
  `Cargo.toml`・`scripts/build-frontend.sh` は無変更。再検証は不要と判断した。
