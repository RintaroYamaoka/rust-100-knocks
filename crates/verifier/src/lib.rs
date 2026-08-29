//! 問題コンテンツの品質検証ハーネス。
//!
//! 上流の実行サービス (Playground / Wandbox) に負荷をかけず、ローカルで
//! `answer_code` / `starter_code` を実際にコンパイル・実行して「収録して良い問題か」を
//! 機械判定する。Rust はローカル cargo、他 6 言語は版を固定した Docker イメージ
//! (ADR 0002 の表が正本)。

pub mod docker;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use shared::language::Language;
use shared::playground::{TEST_FAILED_MARKER, TEST_OK_MARKER};
use shared::problem::{compose_submission, Level, Problem};

use crate::docker::{plan_batch, CaseKind, RunCase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemIssue {
    pub id: String,
    pub message: String,
}

impl ProblemIssue {
    pub fn new(id: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            message: message.into(),
        }
    }
}

pub fn load_problems_str(json: &str) -> Result<Vec<Problem>, serde_json::Error> {
    serde_json::from_str(json)
}

/// `description_md` の最小文字数。これを下回るものは説明として成立していない。
pub const MIN_DESCRIPTION_CHARS: usize = 80;

/// 実行なしで判定できる整合性チェック。
///
/// `expected_language` / `expected_level` はファイルのパスから来る (パスが正本で、
/// `Problem` のフィールドはその冗長コピー)。
pub fn validate_static(
    problems: &[Problem],
    expected_language: Language,
    expected_level: Level,
) -> Vec<ProblemIssue> {
    let mut issues = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    let mut titles: HashMap<&str, &str> = HashMap::new();
    let mut answers: HashMap<&str, &str> = HashMap::new();

    for p in problems {
        if !seen_ids.insert(p.id.as_str()) {
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
        if p.language != expected_language {
            issues.push(ProblemIssue::new(
                &p.id,
                format!(
                    "language ({}) がファイルの言語 ({}) と一致しません",
                    p.language.slug(),
                    expected_language.slug()
                ),
            ));
        }
        if p.level != expected_level {
            issues.push(ProblemIssue::new(&p.id, "level がファイルの難易度と一致しません"));
        }

        for (name, value) in [
            ("title", &p.title),
            ("description_md", &p.description_md),
            ("starter_code", &p.starter_code),
            ("hidden_tests", &p.hidden_tests),
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

        if p.description_md.chars().count() < MIN_DESCRIPTION_CHARS {
            issues.push(ProblemIssue::new(
                &p.id,
                format!("description_md が短すぎます ({} 文字 / 最低 {MIN_DESCRIPTION_CHARS})", p.description_md.chars().count()),
            ));
        }

        issues.extend(validate_hidden_tests(p));

        // 使い回しの検出。1 問をコピーした 100 問は他の検査を全部通ってしまう
        if let Some(other) = titles.insert(p.title.as_str(), p.id.as_str()) {
            issues.push(ProblemIssue::new(
                &p.id,
                format!("title が {other} と重複しています: 「{}」", p.title),
            ));
        }
        if let Some(other) = answers.insert(p.answer_code.as_str(), p.id.as_str()) {
            issues.push(ProblemIssue::new(
                &p.id,
                format!("answer_code が {other} と完全に同一です"),
            ));
        }
    }
    issues
}

/// `hidden_tests` が判定契約を満たしているか (言語別)。
///
/// Rust は `#[test]` 形式を維持する (既存 300 問がこれに依存している)。
/// 他言語は成功/失敗の目印を自分で出す必要がある。
fn validate_hidden_tests(p: &Problem) -> Vec<ProblemIssue> {
    let mut issues = Vec::new();
    match p.language {
        Language::Rust => {
            let n = p.hidden_tests.matches("#[test]").count();
            if n < 2 {
                issues.push(ProblemIssue::new(
                    &p.id,
                    format!("hidden_tests の #[test] が {n} 個です (最低 2 個)"),
                ));
            }
        }
        _ => {
            if !p.hidden_tests.contains(TEST_OK_MARKER) {
                issues.push(ProblemIssue::new(
                    &p.id,
                    format!("hidden_tests が成功の目印 \"{TEST_OK_MARKER}\" を出力していません"),
                ));
            }
            if !p.hidden_tests.contains(TEST_FAILED_MARKER) {
                issues.push(ProblemIssue::new(
                    &p.id,
                    format!("hidden_tests が失敗の目印 \"{TEST_FAILED_MARKER}\" を出力していません"),
                ));
            }
            let checks = p.hidden_tests.matches("FAILED: ").count();
            if checks < 1 {
                issues.push(ProblemIssue::new(
                    &p.id,
                    "hidden_tests に個別検査の失敗表示 (FAILED: <名前>) がありません",
                ));
            }
        }
    }
    issues
}

// ---- 実行検証 ----

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseOutcome {
    pub problem_id: String,
    pub kind: CaseKind,
    /// exit 0 かつ stdout に成功の目印があるか
    pub passed: bool,
    pub output: String,
}

#[derive(Debug, Default)]
pub struct BatchReport {
    pub outcomes: Vec<CaseOutcome>,
    pub container_runs: usize,
    pub errors: Vec<String>,
}

/// 提出コードが「テストを最後まで走らせて全部通った」かどうか。
/// 終了コードだけでは、テストが 1 件も走らないまま exit 0 したケースと区別できない。
fn case_passed(exit_code: i32, stdout: &str) -> bool {
    exit_code == 0 && stdout.contains(TEST_OK_MARKER)
}

/// 1 バッチ (= 1 ファイル分の問題群) をコンテナ 1 回で検証する。
pub fn run_batch_docker(
    language: Language,
    problems: &[Problem],
    workdir: &Path,
) -> std::io::Result<BatchReport> {
    let mut report = BatchReport::default();
    if problems.is_empty() {
        return Ok(report);
    }

    let cases: Vec<RunCase> = problems
        .iter()
        .flat_map(|p| {
            [
                (CaseKind::Answer, &p.answer_code),
                (CaseKind::Starter, &p.starter_code),
            ]
            .into_iter()
            .map(|(kind, code)| RunCase {
                problem_id: p.id.clone(),
                kind,
                code: compose_submission(language, code, &p.hidden_tests),
            })
        })
        .collect();

    // 作業ディレクトリを組む
    let _ = fs::remove_dir_all(workdir);
    fs::create_dir_all(workdir.join("cases"))?;
    for c in &cases {
        let dir = workdir.join("cases").join(c.dir_name());
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(language.source_file_name()), &c.code)?;
    }

    let plan = plan_batch(language, &cases, workdir);
    for run in &plan {
        fs::write(workdir.join("run.sh"), &run.script)?;
        let out = docker::execute(run)?;
        report.container_runs += 1;
        if !out.status.success() {
            report.errors.push(format!(
                "コンテナが異常終了しました (image={}, status={:?}): {}",
                run.image,
                out.status.code(),
                String::from_utf8_lossy(&out.stderr).chars().take(500).collect::<String>()
            ));
        }
    }

    for c in &cases {
        let dir = workdir.join("cases").join(c.dir_name());
        let exit = fs::read_to_string(dir.join("_exit")).ok();
        let stdout = fs::read_to_string(dir.join("_stdout")).unwrap_or_default();
        let stderr = fs::read_to_string(dir.join("_stderr")).unwrap_or_default();

        let Some(code) = exit.as_deref().and_then(|s| s.trim().parse::<i32>().ok()) else {
            // 実行結果が無い = タイムアウトかコンテナ側の異常。未検証にせず失敗として数える
            report.outcomes.push(CaseOutcome {
                problem_id: c.problem_id.clone(),
                kind: c.kind,
                passed: false,
                output: "実行結果が記録されていません (タイムアウトの可能性)".into(),
            });
            continue;
        };
        report.outcomes.push(CaseOutcome {
            problem_id: c.problem_id.clone(),
            kind: c.kind,
            passed: case_passed(code, &stdout),
            output: format!("exit={code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"),
        });
    }
    Ok(report)
}

/// Rust は Docker を使わずローカル cargo で 1 問ずつ検証する (この方が速い)。
pub fn run_problem_rust(scratch_dir: &Path, code: &str, hidden_tests: &str) -> std::io::Result<CaseOutcome> {
    let src_dir = scratch_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    let manifest = scratch_dir.join("Cargo.toml");
    if !manifest.exists() {
        fs::write(
            &manifest,
            // 空の [workspace] で親 workspace への取り込みを防ぐ
            "[package]\nname = \"knock-scratch\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[workspace]\n",
        )?;
    }
    fs::write(
        src_dir.join("lib.rs"),
        compose_submission(Language::Rust, code, hidden_tests),
    )?;

    let out = Command::new("timeout")
        .arg(docker::CASE_TIMEOUT_SECS.to_string())
        .arg("cargo")
        .args(["test", "--quiet"])
        .current_dir(scratch_dir)
        .env("CARGO_TERM_COLOR", "never")
        .output()?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    Ok(CaseOutcome {
        problem_id: String::new(),
        kind: CaseKind::Answer,
        passed: case_passed(out.status.code().unwrap_or(-1), &stdout),
        output: format!("{stdout}{stderr}"),
    })
}
