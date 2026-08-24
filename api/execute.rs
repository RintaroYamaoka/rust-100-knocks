//! POST /api/execute — Rust Playground へのバリデーション付きプロキシ。
//! 契約・許可リスト検証は shared::playground に集約し、ここは HTTP glue のみ。
//! Vercel 公式 Rust ランタイム (vercel_runtime 2.x) 上で動く。

use std::time::Duration;

use http_body_util::BodyExt;
use hyper::StatusCode;
use shared::playground::{validate, ExecuteRequest, ExecuteResponse};
use vercel_runtime::{run, service_fn, Error, Request, Response};

const UPSTREAM: &str = "https://play.rust-lang.org/execute";
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(40);

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}

fn json_response(status: StatusCode, body: String) -> Result<Response<String>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json; charset=utf-8")
        .header("cache-control", "no-store")
        .body(body)?)
}

fn json_error(status: StatusCode, message: &str) -> Result<Response<String>, Error> {
    json_response(status, serde_json::json!({ "error": message }).to_string())
}

pub async fn handler(req: Request) -> Result<Response<String>, Error> {
    if req.method() != hyper::Method::POST {
        return json_error(StatusCode::METHOD_NOT_ALLOWED, "POST のみ受け付けます");
    }

    let body = match req.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return json_error(StatusCode::BAD_REQUEST, "リクエストボディを読めませんでした"),
    };
    let exec_req: ExecuteRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return json_error(StatusCode::BAD_REQUEST, "リクエストボディが不正です"),
    };
    if let Err(msg) = validate(&exec_req) {
        return json_error(StatusCode::BAD_REQUEST, &msg);
    }

    let client = reqwest::Client::builder().timeout(UPSTREAM_TIMEOUT).build()?;

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

    if upstream.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
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

    json_response(StatusCode::OK, serde_json::to_string(&exec_resp)?)
}
