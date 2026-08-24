use serde::{Deserialize, Serialize};

/// 提出コードの上限。Playground へのプロキシ時に DoS 的な巨大ペイロードを弾く。
pub const MAX_CODE_BYTES: usize = 64 * 1024;

/// `/api/execute` の受信契約 = Rust Playground /execute の送信契約。
/// フロントが組み立て、バックエンドは検証してそのまま上流へ転送する
/// (形を揃えることで trunk serve の dev プロキシでも同じ経路が成立する)。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExecuteRequest {
    pub channel: String,
    pub mode: String,
    pub edition: String,
    #[serde(rename = "crateType")]
    pub crate_type: String,
    pub tests: bool,
    pub code: String,
    #[serde(default)]
    pub backtrace: bool,
}

impl ExecuteRequest {
    /// 正誤判定: 提出コード (ユーザーコード + hidden_tests) をテストモードで実行する。
    pub fn judge(code: &str) -> Self {
        Self {
            channel: "stable".into(),
            mode: "debug".into(),
            edition: "2024".into(),
            crate_type: "lib".into(),
            tests: true,
            code: code.into(),
            backtrace: false,
        }
    }

    /// 素実行: fn main を持つコードをそのまま実行する (println! デバッグ用)。
    pub fn run(code: &str) -> Self {
        Self {
            crate_type: "bin".into(),
            tests: false,
            ..Self::judge(code)
        }
    }
}

/// Playground が返す実行結果。フロントはこれを `classify` で解釈する。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExecuteResponse {
    pub success: bool,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
}

/// プロキシが上流へ転送してよいリクエストか (許可リスト方式)。
pub fn validate(req: &ExecuteRequest) -> Result<(), String> {
    if req.code.len() > MAX_CODE_BYTES {
        return Err(format!("コードが大きすぎます (上限 {} bytes)", MAX_CODE_BYTES));
    }
    if !["stable", "beta", "nightly"].contains(&req.channel.as_str()) {
        return Err(format!("不正な channel: {}", req.channel));
    }
    if !["debug", "release"].contains(&req.mode.as_str()) {
        return Err(format!("不正な mode: {}", req.mode));
    }
    if !["2015", "2018", "2021", "2024"].contains(&req.edition.as_str()) {
        return Err(format!("不正な edition: {}", req.edition));
    }
    if !["bin", "lib"].contains(&req.crate_type.as_str()) {
        return Err(format!("不正な crateType: {}", req.crate_type));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// コンパイルが通り全テスト成功 (= 正解) / 素実行なら exit 0
    Passed,
    /// コンパイルは通ったがテストが失敗 (= 未正解)
    TestsFailed,
    /// rustc のコンパイルエラー
    CompileError,
    /// 実行時パニック等
    RuntimeError,
}

/// Playground の応答を結果種別に分類する。
/// テスト失敗は cargo test の "test result: FAILED" が stdout に出ることを信号にする。
pub fn classify(resp: &ExecuteResponse) -> Outcome {
    if resp.success {
        return Outcome::Passed;
    }
    let has_compile_error = resp.stderr.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("error[") || t.starts_with("error:")
    });
    if has_compile_error {
        Outcome::CompileError
    } else if resp.stdout.contains("test result: FAILED") {
        Outcome::TestsFailed
    } else {
        Outcome::RuntimeError
    }
}

/// stderr から rustc エラーコード (E0308 等) を出現順・重複なしで抜き出す。
/// UI が公式 error_codes ドキュメントへのリンクを張るのに使う。
pub fn extract_error_codes(stderr: &str) -> Vec<String> {
    let mut codes: Vec<String> = Vec::new();
    let mut rest = stderr;
    while let Some(pos) = rest.find("error[") {
        rest = &rest[pos + "error[".len()..];
        if let Some(end) = rest.find(']') {
            let code = &rest[..end];
            if code.len() == 5
                && code.starts_with('E')
                && code[1..].chars().all(|c| c.is_ascii_digit())
                && !codes.iter().any(|c| c == code)
            {
                codes.push(code.to_string());
            }
        }
    }
    codes
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Error,
    Warning,
    Note,
    Plain,
}

/// コンソール表示用: rustc 出力の 1 行を色分け種別に分類する。
pub fn classify_line(line: &str) -> LineKind {
    let t = line.trim_start();
    if t.starts_with("error") {
        LineKind::Error
    } else if t.starts_with("warning") {
        LineKind::Warning
    } else if t.starts_with("note:") || t.starts_with("help:") || t.starts_with("-->") {
        LineKind::Note
    } else {
        LineKind::Plain
    }
}
