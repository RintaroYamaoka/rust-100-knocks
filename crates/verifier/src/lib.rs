//! 問題コンテンツの品質検証ハーネス。
//! Playground に負荷をかけず、ローカルの cargo で answer_code + hidden_tests を実際に
//! コンパイル・実行して「収録して良い問題か」を機械判定する。

use std::fs;
use std::path::Path;
use std::process::Command;

use shared::problem::{compose_submission, Level, Problem};

#[derive(Debug, Clone)]
pub struct ProblemIssue {
    pub id: String,
    pub message: String,
}

impl ProblemIssue {
    fn new(id: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            message: message.into(),
        }
    }
}

pub fn load_problems_str(json: &str) -> Result<Vec<Problem>, serde_json::Error> {
    serde_json::from_str(json)
}

/// 実行なしで判定できる整合性チェック。
pub fn validate_static(problems: &[Problem], expected_level: Level) -> Vec<ProblemIssue> {
    let mut issues = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for p in problems {
        if !seen.insert(p.id.as_str()) {
            issues.push(ProblemIssue::new(&p.id, "id が重複しています"));
        }
        let prefix_ok = p.id.starts_with(expected_level.id_prefix())
            && p.id.len() == 4
            && p.id[1..].chars().all(|c| c.is_ascii_digit());
        if !prefix_ok {
            issues.push(ProblemIssue::new(
                &p.id,
                format!("id の形式が不正です (期待: {}NNN)", expected_level.id_prefix()),
            ));
        }
        if p.level != expected_level {
            issues.push(ProblemIssue::new(&p.id, "level がファイルの難易度と一致しません"));
        }
        if !p.hidden_tests.contains("#[test]") {
            issues.push(ProblemIssue::new(&p.id, "hidden_tests に #[test] がありません"));
        }
        for (name, value) in [
            ("title", &p.title),
            ("description_md", &p.description_md),
            ("starter_code", &p.starter_code),
            ("answer_code", &p.answer_code),
            ("explanation_md", &p.explanation_md),
        ] {
            if value.trim().is_empty() {
                issues.push(ProblemIssue::new(&p.id, format!("{name} が空です")));
            }
        }
        if p.starter_code == p.answer_code {
            issues.push(ProblemIssue::new(&p.id, "starter_code と answer_code が同一です"));
        }
    }
    issues
}

#[derive(Debug)]
pub struct RunResult {
    pub passed: bool,
    pub output: String,
}

/// コード + 判定テストをスクラッチ crate で `cargo test` し、通ったかを返す。
/// Playground の判定 (edition 2024 / lib / tests) と同条件に揃える。
pub fn run_problem(scratch_dir: &Path, code: &str, hidden_tests: &str) -> std::io::Result<RunResult> {
    let src_dir = scratch_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    let manifest = scratch_dir.join("Cargo.toml");
    if !manifest.exists() {
        fs::write(
            &manifest,
            "[package]\nname = \"knock-scratch\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )?;
    }
    fs::write(src_dir.join("lib.rs"), compose_submission(code, hidden_tests))?;

    let out = Command::new("cargo")
        .args(["test", "--quiet"])
        .current_dir(scratch_dir)
        .env("CARGO_TERM_COLOR", "never")
        .output()?;
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(RunResult {
        passed: out.status.success(),
        output,
    })
}
