//! POST /api/execute — Rust Playground へのバリデーション付きプロキシ。
//! 契約・許可リスト検証は shared::playground に集約し、ここは HTTP glue のみ。

use std::time::Duration;

use shared::playground::{validate, ExecuteRequest, ExecuteResponse};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

const UPSTREAM: &str = "https://play.rust-lang.org/execute";
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(40);

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

fn json_error(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json; charset=utf-8")
        .body(serde_json::json!({ "error": message }).to_string().into())?)
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    if req.method() != "POST" {
        return json_error(StatusCode::METHOD_NOT_ALLOWED, "POST のみ受け付けます");
    }

    let exec_req: ExecuteRequest = match serde_json::from_slice(req.body()) {
        Ok(r) => r,
        Err(_) => return json_error(StatusCode::BAD_REQUEST, "リクエストボディが不正です"),
    };
    if let Err(msg) = validate(&exec_req) {
        return json_error(StatusCode::BAD_REQUEST, &msg);
    }

    let client = reqwest::Client::builder()
        .timeout(UPSTREAM_TIMEOUT)
        .build()
        .map_err(Error::from)?;

    let upstream = match client.post(UPSTREAM).json(&exec_req).send().await {
        Ok(resp) => resp,
        Err(e) if e.is_timeout() => {
            return json_error(
                StatusCode::GATEWAY_TIMEOUT,
                "実行がタイムアウトしました。無限ループがないか確認してください",
            )
        }
        Err(_) => {
            return json_error(
                StatusCode::BAD_GATEWAY,
                "実行サービス (Rust Playground) に接続できませんでした。しばらくして再試行してください",
            )
        }
    };

    if upstream.status() == StatusCode::TOO_MANY_REQUESTS {
        return json_error(
            StatusCode::TOO_MANY_REQUESTS,
            "実行サービスが混雑しています。少し待ってから再試行してください",
        );
    }
    if !upstream.status().is_success() {
        return json_error(StatusCode::BAD_GATEWAY, "実行サービスがエラーを返しました");
    }

    let exec_resp: ExecuteResponse = match upstream.json().await {
        Ok(r) => r,
        Err(_) => return json_error(StatusCode::BAD_GATEWAY, "実行サービスの応答を解釈できませんでした"),
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json; charset=utf-8")
        .header("cache-control", "no-store")
        .body(serde_json::to_string(&exec_resp)?.into())?)
}
