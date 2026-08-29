# 0001: コード実行は Rust Playground API プロキシで行う

- 日付: 2026-08-25
- 状態: 採用 (一部を ADR 0002 が改訂)

> **ADR 0002 (2026-08-29) による改訂**: 多言語対応にあたり、次の 2 点が変わった。
> 変更されなかった決定 (サーバーに正誤判定の状態を持たない / 実行サービスに一括負荷を
> かけない / 問題検証はローカルで行う) はそのまま有効である。
>
> - **受信契約の形**: 「Playground と同形」ではなくなり、`{language, code}` の自前契約になった。
>   これに伴い `trunk serve` の dev プロキシは Playground 直結では成立しなくなり、
>   `vercel dev` 向けに張り替えた
> - **無変換の原則**: C# のビルドノイズ除去に限って変換を行う (理由と除去範囲は ADR 0002)

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
