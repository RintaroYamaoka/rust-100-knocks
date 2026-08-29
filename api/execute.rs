//! POST /api/execute — 言語ごとの実行サービスへのバリデーション付きプロキシ。
//!
//! Rust は play.rust-lang.org、他 6 言語は wandbox.org に振り分ける (ADR 0002)。
//! 契約・検証・応答の詰め替えは shared::playground に集約し、ここは HTTP glue のみ。
//! Vercel 公式 Rust ランタイム (vercel_runtime 2.x) 上で動く。

use std::time::Duration;

use http_body_util::BodyExt;
use hyper::StatusCode;
use shared::language::{Backend, Language};
use shared::playground::{
    normalize_playground, normalize_wandbox, validate, wandbox_request, ExecuteRequest,
    ExecuteResponse, PlaygroundRequest, PlaygroundResponse, WandboxResponse,
};
use vercel_runtime::{run, service_fn, Error, Request, Response};

const PLAYGROUND_URL: &str = "https://play.rust-lang.org/execute";
const WANDBOX_URL: &str = "https://wandbox.org/api/compile.json";
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(40);

/// Wandbox は既定の User-Agent を 403 で弾く。明示的に名乗る必要がある。
const USER_AGENT: &str = "rust-100-knocks (+https://github.com/RintaroYamaoka/rust-100-knocks)";

/// Wandbox の一時エラーに対する再試行回数。
const WANDBOX_RETRIES: usize = 2;

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

fn ok(resp: &ExecuteResponse) -> Result<Response<String>, Error> {
    json_response(StatusCode::OK, serde_json::to_string(resp)?)
}

fn client() -> Result<reqwest::Client, Error> {
    Ok(reqwest::Client::builder()
        .timeout(UPSTREAM_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()?)
}

const BUSY: &str = "実行サービスが混雑しています。少し待ってから再試行してください";
const UNREACHABLE: &str = "実行サービスに接続できませんでした。しばらくして再試行してください";
const TIMED_OUT: &str = "実行がタイムアウトしました。無限ループがないか確認してください";
const UNPARSEABLE: &str = "実行サービスの応答を解釈できませんでした";

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

    match exec_req.language.backend() {
        Backend::Playground => run_playground(&exec_req.code).await,
        Backend::Wandbox { .. } => run_wandbox(exec_req.language, &exec_req.code).await,
    }
}

async fn run_playground(code: &str) -> Result<Response<String>, Error> {
    let client = client()?;
    let upstream = match client
        .post(PLAYGROUND_URL)
        .json(&PlaygroundRequest::judge(code))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) if e.is_timeout() => return json_error(StatusCode::GATEWAY_TIMEOUT, TIMED_OUT),
        Err(_) => return json_error(StatusCode::BAD_GATEWAY, UNREACHABLE),
    };
    if upstream.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return json_error(StatusCode::TOO_MANY_REQUESTS, BUSY);
    }
    if !upstream.status().is_success() {
        return json_error(StatusCode::BAD_GATEWAY, "実行サービスがエラーを返しました");
    }
    match upstream.json::<PlaygroundResponse>().await {
        Ok(raw) => ok(&normalize_playground(&raw)),
        Err(_) => json_error(StatusCode::BAD_GATEWAY, UNPARSEABLE),
    }
}

async fn run_wandbox(language: Language, code: &str) -> Result<Response<String>, Error> {
    let Some(payload) = wandbox_request(language, code) else {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "この言語の実行先が設定されていません");
    };
    let client = client()?;

    // Wandbox は過負荷時に OCI ランタイムの一時エラーを返す。
    // これをコンパイルエラーとして見せると、正しいコードが赤く出て学習者が混乱するので、
    // 数回だけ再試行してから上流一時障害として返す。
    for attempt in 0..=WANDBOX_RETRIES {
        let upstream = match client.post(WANDBOX_URL).json(&payload).send().await {
            Ok(r) => r,
            Err(e) if e.is_timeout() => return json_error(StatusCode::GATEWAY_TIMEOUT, TIMED_OUT),
            Err(_) => return json_error(StatusCode::BAD_GATEWAY, UNREACHABLE),
        };
        if upstream.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return json_error(StatusCode::TOO_MANY_REQUESTS, BUSY);
        }
        if !upstream.status().is_success() {
            return json_error(StatusCode::BAD_GATEWAY, "実行サービスがエラーを返しました");
        }
        let raw: WandboxResponse = match upstream.json().await {
            Ok(r) => r,
            Err(_) => return json_error(StatusCode::BAD_GATEWAY, UNPARSEABLE),
        };
        if raw.is_upstream_transient_error() {
            if attempt < WANDBOX_RETRIES {
                tokio::time::sleep(Duration::from_millis(700 * (attempt as u64 + 1))).await;
                continue;
            }
            return json_error(StatusCode::SERVICE_UNAVAILABLE, BUSY);
        }
        return ok(&normalize_wandbox(language, &raw));
    }
    json_error(StatusCode::SERVICE_UNAVAILABLE, BUSY)
}
