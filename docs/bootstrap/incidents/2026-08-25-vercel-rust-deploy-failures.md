# 2026-08-25 Vercel デプロイが 4 段階で失敗 (build.sh 衝突 / 展開先不在 / glibc / 非公式ランタイム)

## 何が起きたか

初回デプロイから本番 API が動くまでに 4 回失敗した。いずれも「ローカルでは通る」が Vercel のビルド/実行環境の前提が違うことによる。

| # | 症状 | 原因 | 修正 |
|---|---|---|---|
| 1 | 関数ビルド中に rustup/trunk が走り失敗 | vercel-rust ランタイムは **repo 直下の `build.sh` を関数ビルド前フックとして自動実行**する。フロント用ビルドスクリプトを `build.sh` と命名していた | `scripts/build-frontend.sh` に改名 (`16961e7`) |
| 2 | `tar: /vercel/.cargo/bin: Cannot open` | build image は cargo が別パスにあり `$HOME/.cargo/bin` が存在しない | `mkdir -p` (`099f90d`) |
| 3 | `trunk: GLIBC_2.35 not found` | gnu 版 prebuilt trunk が build image の glibc より新しい glibc を要求 | musl (静的リンク) 版に変更 (`bae6398`) |
| 4 | 本番 `/api/execute` が `FUNCTION_INVOCATION_FAILED` (バリデーション拒否ケースでも 500) | `vercel-rust@4.0.11` + `vercel_runtime 1.x` (vercel-community/rust) は **2026-01 にアーカイブ済み**。現在は Vercel 公式 Rust ランタイム (`vercel_runtime = "2"`, `vercel.json` の `functions` 指定不要) が正 | 公式ランタイムへ移行 (`5d95a99`) |

## 教訓

- **外部ランタイムは採用前に「まだ生きているか」を 1 回確認する** (repo の archived 表示 / crates.io の latest と docs のズレ)。`cargo info vercel_runtime` が `1.1.6 (latest 2.4.0)` と出ていた時点で気づけた
- **Vercel の build image の前提を推測しない**: cargo の位置・glibc のバージョン・自動実行されるファイル名 (`build.sh`) は手元と違う。prebuilt バイナリは musl 版を選ぶ
- ローカルで Lambda Runtime API を模した起動テストは「バイナリが起動できるか」しか証明しない (#4 はローカルで再現しない)。本番で失敗したら **ランタイムログをまず見る** (今回は CLI スコープの都合で見られず、公式 docs の照合で特定した)

## 再発防止

- `CLAUDE.md` 既知の地雷に #1 / #4 を記載済み
- `scripts/build-frontend.sh` にコメントで #2 / #3 の理由を残した
