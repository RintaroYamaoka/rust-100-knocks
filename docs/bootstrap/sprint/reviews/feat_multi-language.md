# 統合前レビュー: feat/multi-language

日付: 2026-08-30
対象: origin/main..feat/multi-language (26 コミット)

verdict: reject

理由: R1 が「本番に出したあと main に戻すと、既存利用者の進捗が全部消える」という
データ破壊の経路を作っている。D15 (実 Function の疎通) が OPEN のまま本番で初検証する
以上、切り戻しは唯一の復旧手段であり、その復旧手段が壊れている状態では出せない。
R1 は 1 行 (`storage.rs:9` のキーを v2 に上げて v1 を残す) で塞げる。

## 指摘

| # | 観点 | 指摘 (根拠 file:line) | 深刻度 | 対処 |
|---|---|---|---|---|
| R1 | 4 切り戻し | **切り戻すと全利用者の進捗が消える。** localStorage のキー名は main と同一 (`crates/app/src/storage.rs:9` = `"rust100knocks.progress.v1"`、`git show origin/main:crates/app/src/storage.rs:6` も同一文字列) なのに、中身のキー空間が `b001` → `rust/b001` に変わる。`crates/shared/src/progress.rs:64` の `map.remove(&old)` が旧キーを**削除**し、`crates/app/src/storage.rs:25` が即座に同じキーへ書き戻す。新ビルドを一度でも開いた利用者を main に戻すと、main の `status_of(map, id)` (`origin/main:crates/shared/src/progress.rs:31-32`) は `b001` を引くのでヒット 0 件 = ✓ マークと下書きが全滅する (panic はしない。`.ok().unwrap_or_default()` なので静かに空になる)。入力: 本番の既存利用者 (`b001` 等を保存済み) → 新ビルドを開く → main に revert → 進捗 0%。 | **blocker** | `PROGRESS_KEY` を `...progress.v2` に上げ、v1 は読むだけで消さない |
| R2 | 4 切り戻し | **戻して再前進すると、戻していた間の進捗が黙って捨てられる。** `crates/shared/src/progress.rs:66` は `map.entry(new).or_insert(entry)` で、`updated_at_ms` を一切見ない。R1 の切り戻し中に main が `b001` へ書いた新しいエントリは、再マージ時に既存の `rust/b001` (古い) に負けて破棄される。この挙動はテストで固定までされている (`crates/shared/tests/progress.rs:88-95`)。 | major | R1 を v2 化したうえで、衝突時は `updated_at_ms` の新しい方を採る |
| R3 | 1 デプロイ | **Vercel のビルドが通ったことのある最後のコミットに、2 つ目の `[[bin]]` が入っていない。** 検証記録 `docs/bootstrap/verification/feat-multi-language.md:24` が preview ビルド成功の根拠として挙げるのは `5b3dd13f`。`tools/local_server.rs` と `Cargo.toml:14-16` の `[[bin]] local-server` を足したのは 3 コミット後の `0951e76` (`git log --oneline origin/main..feat/multi-language -- Cargo.toml` はこの 1 件のみ)。CLAUDE.md が「関数は `Cargo.toml` の `[[bin]]` で自動検出」と書いている、まさにその機構を触っている。ローカルでは `cargo check --bins` が両方通ることは確認済みで、Vercel 公式 docs も「`api/**/*.rs` が関数になる」としているので `tools/` 配下は関数にはならない見込みだが、**この構成でのビルドは Vercel 上で一度も走っていない**。このリポジトリの唯一の事故が Vercel 限定のビルド/実行失敗 (`docs/bootstrap/incidents/2026-08-25-vercel-rust-deploy-failures.md`) である以上、推定で通してはいけない。 | major | マージ前に現 HEAD で preview を 1 回押し、ビルド成功を D15 の証拠に追記する |
| R4 | 3 データ | **`.vercelignore` に `_batches` が無い。** `9305bc6` は `.gitignore:22-24` に `data/problems/*/_batches/` を足しただけ。git push 経由のデプロイは実際に無害 (`git ls-files 'data/problems/**/_batches/*'` = 0、`dist/` にも無し) だが、`.vercelignore` (`target/**` / `dist` / `docs` / `.bootstrap` / `node_modules` の 5 行) には無いので、`vercel deploy` を手元から打つ経路では 90 ファイル・約 9MB がアップロードされ、`index.html:21` の `rel="copy-dir" href="data"` が `dist/` へ丸ごと写す。現に作業ツリーには 6 言語分の `_batches` が残っている (`data/` 実測 19MB / tracked 9.84MB)。 | minor | `.vercelignore` に `data/problems/*/_batches/` を 1 行足す |
| R5 | 5 正しさ | `nav()` が言語を見ずに素の id で現在位置を探す — `crates/app/src/app.rs:247,250` (`p.id == cur`)。今は自動選択 Effect (`app.rs:174-186`) が `selected` と `problems` の言語を揃えるので実害は出ていないが、CLAUDE.md が名指しで警告している「素の `p.id` で引く」経路がここだけ残っている。`problems` が Python 一覧・`selected` が rust/b050 の状態で `nav(1)` を押すと `python/b051` に飛ぶ。 | minor | `progress_key` 比較か `(language, id)` 比較にする |
| R6 | 2 本番差 | **読み込み中の状態が無く、切替直後の UI が嘘をつく。** `crates/app/src/app.rs:181-185` は `ps.first()` が `Some` のときしか選択を移さない。キャッシュミス時は 300〜680KB の JSON を落とし切るまで一覧が空で、`selected`・問題ペイン・エディタは**前の言語のまま**。その間 `list.rs:104-107` は「条件に合う問題がありません」と出し、ヘッダは「0 / 0 問クリア」、実行ボタンは押せる (`app.rs:393`)。ローカルでは一瞬なので気づかないが、本番の回線では数秒見える。入力: Rust b050 を開く → 遅い回線でセレクタを Python に → 数秒間「Python と表示しながら Rust b050 を実行する」。 | minor | フェッチ中フラグを持ち、一覧の空表示と実行ボタンを分岐させる |
| R7 | 2 本番差 | 起動時の存在確認が 21 回の **逐次** HEAD (`crates/app/src/app.rs:77-84` の二重ループが `.await` を直列に回す)。完了までセレクタは Rust 1 件のまま (`lang.rs:59-69`) で言語を変えられない。RTT 150ms で約 3 秒。失敗はすべて「無い」に倒れ再試行しない (`crates/app/src/api.rs:43-46`) ので、21 本のうち 1 本が瞬断しただけでその言語がセッション中ずっとセレクタから消える。 | minor | 並列化するか、失敗時に 1 回だけ再試行する |
| R8 | 5 正しさ | `load_error` をフェッチ開始時にクリアしていない — `crates/app/src/app.rs:98-115` はキャッシュヒット (101 行) と成功時 (106 行) にしか `None` にしない。Java の取得に失敗 → Python に切り替える、で Java のエラー文が Python の読み込み中ずっと残る。 | minor | `spawn_local` の直前で `load_error.set(None)` |
| R9 | 2 本番差 | `editor.js` だけ内容ハッシュが付かない (`index.html:22-23`。wasm と CSS は `dist/app-739d7344916a988e.js` / `main-f28e05e64bb59964.css` とハッシュ付き)。旧バンドル (485,837 バイト、`setLanguage` を 1 件も含まない) を HTTP キャッシュに持つ再訪者は、新 wasm × 旧 glue になる。`crates/app/src/editor.rs:73-77` の `has_fn` ガードで panic はしないが、**7 言語すべてが Rust のハイライトで描かれる**。Vercel の既定はハッシュなし静的ファイルを都度再検証するので発生確率は低い。 | minor | `?v=` かハッシュ付きコピーにする |
| R10 | 5 正しさ | 移行時の保存失敗を握り潰している — `crates/app/src/storage.rs:25` の `let _ = save_progress(&map);`。同じファイルの 39-49 行が「戻り値を捨ててはいけない」と書いている当の関数で、`app.rs:146,226` は `storage_failed` に繋いでいるのに、移行だけ繋がっていない。クォータ超過中の利用者は、移行が保存されないまま毎回やり直す。 | minor | 戻り値を `storage_failed` に反映する |
| R11 | 契約文書 | ADR 0002 が自己矛盾している。84-85 行は「`.cs(行,列): error\|warning CSnnnn` にマッチする行**だけ**残す」、120-123 行は「`error`/`warning` を含む行は必ず残すホワイトリスト方向」。実装 (`crates/shared/src/playground.rs:163-200`) は後者。正本が 2 通りに読めると次の変更で前者に寄せられる。 | minor | 84-85 行を 120 行の記述に合わせる |

## 確認した点

- **データは健全**: 実 serde (`crates/shared/src/problem.rs:65-81`) と実 verifier で 21 ファイル 2100 問をパース — パース失敗 0 / `validate_static_with_expected` 0 / `validate_across_levels` 0。キー集合は全 2100 件で完全一致、`language`/`level` はディレクトリ・ファイル名と全件整合、BOM/CRLF/不正 UTF-8 なし、配信 9.84MiB。id は設計どおり言語間で衝突するが、進捗は全経路が `progress_key`/`&Problem` を通る (R5 の 1 箇所を除く)。中間生成物・秘密情報の混入なし (削除は `scripts/_merge-batches.mjs` の 1 件のみ)。
- **データ形式自体の切り戻しは安全**: main の `Problem` (`origin/main:crates/shared/src/problem.rs:40-55`) は `deny_unknown_fields` を持たないので、追加された `"language"` を無視して新 JSON を読める (main の構造体をそのままコンパイルして実測)。パス移動は git の rename として記録されており、revert でコード・データが一緒に戻る。壊れるのは進捗キーだけ (R1)。
- **エディタバンドルは同期している**: `assets/js/editor.js` を `esbuild 0.28.2` で再生成して `cmp` — バイト一致。7 言語モードと Compartment 版 `setLanguage` を含む。
- **デプロイ設定の変更点は把握済み**: `vercel.json` は無変更 (`functions` を書かない前提を維持)、`scripts/build-frontend.sh` も無変更、リポジトリ直下に `build.sh` は無い、`index.html` の `data-trunk` 指定も無変更 (title/description のみ)。`package.json` / `package-lock.json` はこのブランチで新規だが lockfile は同期済み (v3、差分 0) で、`5b3dd13f` の preview ビルドで既に通っている。`maxDuration` 未指定はプロキシの最悪 ~122 秒に対し Hobby 既定 300 秒なので問題なし。未検証で残るのは R3 の 1 点。
- **テストは緑**: `cargo test --workspace --exclude app` / `cargo test -p app` / `npm test` すべて成功。`cargo check --bins` で `execute` / `local-server` 両バイナリが型検査を通ることも確認。
