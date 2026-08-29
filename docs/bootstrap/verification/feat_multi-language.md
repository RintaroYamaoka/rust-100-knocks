# 動作検証: feat/multi-language (WO-0001)
#
# Rust 専用アプリを 7 言語 2100 問に拡張する変更の、統合前の動作テスト計画。
# DoD の定義は WO-0001 の 8/9 節が正本。実測の詳細・スクリーンショット・経緯は
# docs/bootstrap/verification/feat-multi-language.md に記録してある。
#
# 行の形式: STATUS | kind | behaviour | oracle | by | evidence
# 最終更新: 2026-08-30 (本番デプロイ・疎通確認まで完了)
#
# ---- 継ぎ目 (どこを跨いだか) ----
#
# この変更が跨いだ境界は 4 つ。テストはここから導いた (実装からではない)。
#
#   1. フロント ↔ 実行プロキシ
#      契約が {channel, mode, edition, ...} から {language, code} に変わった。
#      判定の分類ロジックもこの境界にまたがる
#   2. プロキシ ↔ 上流 2 系統
#      Rust は Playground、他 6 言語は Wandbox。応答の形が違うものを
#      1 つの ExecuteResponse に均している
#   3. 問題データ ↔ 実行環境
#      2100 問の hidden_tests が、7 種類のコンパイラ/ランタイムで
#      期待どおりに通る/落ちる必要がある
#   4. 旧版 ↔ 新版 (時間をまたぐ継ぎ目)
#      既存利用者の localStorage。前進と切り戻しの両方向で壊れないこと
#
# ---- 計画 ----

PASS | contract | 7 言語の Language / 実行契約 / 問題スキーマ / 進捗キーが型として揃う | cargo test --workspace --exclude app | auto | 129 passed / 0 failed
PASS | contract | 配信される実物のプロキシが 7 言語で実コンパイラ診断を返す (継ぎ目 2) | cargo test -p rust-100-knocks-api -- --ignored (実 Playground / 実 Wandbox に接続) | auto | 4 テスト通過。rust=error[E0308] / cpp=prog.cc:1:33: error: / csharp=error CS0029 / java=prog.java:1: error: / python=SyntaxError / typescript=error TS2322 / javascript=SyntaxError
PASS | contract | C# のビルドノイズが除去され error/warning 行は 1 行も失われない (継ぎ目 2) | shared のテスト 3 件 + 実 Wandbox で成功時 stderr が空であること | auto | csharp_noise_is_removed_entirely_on_success ほか
PASS | unit | 5 種の Outcome が 7 言語で正しく分類される (継ぎ目 1) | shared のテスト。固定値は実測した本物のコンパイラ出力 | auto | 41 件
PASS | gameable | テストを 1 件も実行せず exit 0 したものを正解にしない | shared 2 件 + verifier に実コンテナで sys.exit(0) を投げて検出されること | auto | NoTestsRun に落ちることを実測。ユーザーコードが判定テストより先に終了する経路
PASS | unit | cargo test の要約行 (error: test failed) をコンパイルエラーと誤認しない | 実 Playground でテスト失敗を起こし TestsFailed になること | auto | 実測で誤判定を再現 → 修正 → 両方向を再実測
PASS | e2e | 全 2100 問で answer が通り starter が落ちる。件数も一致する (継ぎ目 3) | cargo run -p verifier -- --expect 2100 (実コンパイラを Docker で実行) | auto | 検証 2100 問 / 問題あり 0 件 / コンテナ起動 18 回 (26分30秒)
PASS | gameable | 問題の使い回しが無い (難易度をまたぐものも含む) | validate_static + validate_across_levels を全 2100 問に適用 | auto | 題名重複 0 / 模範解答重複 0。既存 Rust の 2 組を検出して差し替え済み
PASS | unit | verifier の docker run がバッチあたり 2 回以内 | plan_batch の戻り値を数えるテスト + 実行時の報告値 | auto | 7 言語で 1 回。実行時も 18 回 (21 ファイル中 Rust 3 はローカル cargo)
PASS | e2e | 21 ファイル × 各 100 問が配信される | ls dist/data/problems/*/*.json と各ファイルの件数 | auto | 21 個 / 各 100 問
PASS | contract | 既存 Rust 300 問の内容が変わっていない (継ぎ目 4) | git diff --numstat が移動と language 付与のみであること | auto | 3 ファイルとも 追加100/削除0。例外は a006 (a001 との重複を検出したため差し替え、WO 11.5 に記録)
PASS | e2e | 旧版の進捗が新版で引き継がれる (継ぎ目 4) | app のテストで旧キーを注入し rust/<id> として読めること | auto | migrate 経路のテスト 5 件
PASS | e2e | 新版を開いた後で旧版に切り戻しても進捗が見える (継ぎ目 4) | main のコードが読むキー (v1) が無傷であることを実際のキー名で検査 | auto | v1 は読むだけ・書かない。app のテストで v1 の中身が残ることを確認
PASS | metamorphic | 切り戻していた間の進捗が、再前進時に捨てられない (継ぎ目 4) | 新旧どちらが新しい場合も updated_at_ms の新しい方を採ることを双方向で検査 | auto | shared 3 件 + app 2 件。片側だけのテストだと逆方向で壊れる
PASS | e2e | 7 言語すべてで無作為抽出した問題が starter で不正解・answer で正解 | 実ブラウザ (Playwright) で seed 固定の 3 問ずつ実行 | auto | 21/21。スクリーンショット knocks-shots/D12-<lang>.png
PASS | e2e | 7 言語すべてで実物の診断が表示され、エラー行が着色される | 実ブラウザで壊したコードを実行し DOM の line-error を確認 | auto | 7/7。スクリーンショット knocks-shots/D13-<lang>.png
PASS | e2e | 言語切替で一覧とエディタが追従し、未収録言語はセレクタに出ない | 実ブラウザでセレクタの option を確認 | auto | 収録済み言語のみ表示。データを退避した言語が消えることも確認
PASS | e2e | trunk build --release が成功し、中間生成物が混入しない | ビルド後の dist を確認 | auto | 成功。_batches の混入なし
PASS | e2e | 2 つ目の [[bin]] を含む構成で Vercel のビルドが通る | preview デプロイのビルド結果 | auto | 0951e76 の preview がビルド成功
PASS | manual | Rust 固定の文言が UI から消えている | grep + スクリーンショット目視 | auto | index.html 0 件。ブランドが「100本ノック」、stderr ラベルが「診断出力 (stderr)」
PASS | e2e | Vercel の Function が実際に動く / Vercel から wandbox.org へ通信できる | 本番 (rust-100-knocks.vercel.app) への 7 言語 curl + 実ブラウザ | auto | 2026-08-30 実測。7 言語すべてで 正解→Passed / 壊れたコード→CompileError + 言語固有の診断。実ブラウザでも 14/14・7/7。静的配信も 7 言語すべて 200
DROP | manual | 2100 問の内容が学習教材として妥当か (説明の質・難易度の並び) | 人間の通読 | human | 理由: 2100 問の人力通読は非現実的。機械で取れる範囲 (重複・件数・説明の最小長・検査件数・実行結果) はすべて検査済みで、各言語の抜き取り確認も実施した

# ---- 残っている既知のリスク ----
#
# - 本番での疎通は 2026-08-30 に実施し、7 言語すべて通過した (上の PASS 行)。
#   このリポジトリで唯一起きた事故が「ローカルでは通る」本番専用障害だったので、
#   ここを本番で確かめるまでは完了としなかった。切り戻し先は 32bf310 (マージ前の main)
# - ローカル Docker と Wandbox のパッチ版差: gcc 13.4.0/13.2.0、python 3.13.15/3.13.8、
#   node 20.20.2/20.17.0、dotnet 6.0.428/6.0.425。判定契約のレベルでは 7 言語すべて
#   一致することを実測したが、個々の問題が版差で挙動を変える可能性は残る。
#   ブラウザ検証の無作為抽出がこれを拾う網になっている
# - N3 (再レビューの指摘、残置): 読み込み中に別言語へ切り替え、かつ元の取得が失敗すると、
#   破棄されたフェッチのエラーが別言語の画面に出る。症状はエラー文が 1 つ余分に出るだけ
