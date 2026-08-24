//! バックエンド /api/execute と問題データ (静的 JSON) への通信層。

use shared::playground::{ExecuteRequest, ExecuteResponse};
use shared::problem::{Level, Problem};

/// エラー応答ボディ ({"error": "..."}) から利用者向けメッセージを取り出す。
pub fn error_message_from_body(status: u16, body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
        .unwrap_or_else(|| format!("実行サービスへの接続に失敗しました (HTTP {status})"))
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_problems(level: Level) -> Result<Vec<Problem>, String> {
    let url = format!("/data/problems/{}", level.file_name());
    let resp = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("問題データを取得できませんでした: {e}"))?;
    if !resp.ok() {
        return Err(format!("問題データを取得できませんでした (HTTP {})", resp.status()));
    }
    resp.json()
        .await
        .map_err(|e| format!("問題データを解釈できませんでした: {e}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn execute(req: &ExecuteRequest) -> Result<ExecuteResponse, String> {
    let resp = gloo_net::http::Request::post("/api/execute")
        .json(req)
        .map_err(|e| format!("リクエストを構築できませんでした: {e}"))?
        .send()
        .await
        .map_err(|_| "実行サービスに接続できませんでした。ネットワークを確認してください".to_string())?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(error_message_from_body(status, &text));
    }
    serde_json::from_str(&text).map_err(|_| "実行サービスの応答を解釈できませんでした".to_string())
}

// ---- host スタブ (テストビルド用。実行は wasm でのみ成立する) ----

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_problems(_level: Level) -> Result<Vec<Problem>, String> {
    Err("wasm32 専用".into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn execute(_req: &ExecuteRequest) -> Result<ExecuteResponse, String> {
    Err("wasm32 専用".into())
}
