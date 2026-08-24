# 0001: コード実行は Rust Playground API プロキシで行う

- 日付: 2026-08-25
- 状態: 採用

## 文脈

「実際の rustc エラーが出る」ブラウザ演習アプリを Vercel にデプロイしたい。Vercel Functions 内で rustc は実行できない (コンパイラ不在・実行時間/サイズ制約・任意コード実行のサンドボックス問題)。

## 決定

play.rust-lang.org/execute を自前の Rust 製 Vercel Function (`api/execute.rs`) 経由でプロキシする。

- `/api/execute` の受信契約を **Playground の契約と同形** にする (`shared::playground::ExecuteRequest`)。これにより `trunk serve` の dev プロキシで backend 抜きでも同一経路が成立する
- バックエンドは許可リスト検証 (channel/mode/edition/crateType/コードサイズ) のみ行い、変換しない
- 正誤判定は「ユーザーコード + hidden_tests を tests モードで実行し、success / stdout の `test result: FAILED` / stderr の `error[` で分類」(`shared::playground::classify`)

## 帰結

- 追加インフラゼロ・無料。実 rustc の出力がそのまま得られる
- 非公式 API のためレート制限・仕様変更リスクあり → 429/502 をユーザー向けメッセージに整形。問題コンテンツの検証は Playground でなくローカル cargo (`crates/verifier`) で行い、負荷をかけない
- 将来自前実行サーバーに切り替える場合も `ExecuteRequest/Response` 契約の裏側を差し替えるだけでよい

## 却下した代替案

- 自前実行サーバー (Fly.io 等): 構築・運用・サンドボックス化のコストが個人プロジェクトに過大
- ブラウザ内 WASM コンパイラ: rustc の WASM 化は現実的でない
