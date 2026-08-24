//! 使い方:
//!   cargo run -p verifier                 # data/problems/ の全レベルを検証
//!   cargo run -p verifier -- beginner     # 特定レベルのみ
//!   cargo run -p verifier -- --answers-only   # starter が落ちる検査を省略 (高速化)
//!   cargo run -p verifier -- --file batch.json --level beginner --scratch target/s1
//!                                         # 単一ファイルを独立スクラッチで検証 (並列生成用)

use std::path::PathBuf;
use std::process::ExitCode;

use shared::problem::Level;
use verifier::{load_problems_str, run_problem, validate_static, ProblemIssue};

fn level_from_arg(arg: &str) -> Option<Level> {
    match arg {
        "beginner" => Some(Level::Beginner),
        "intermediate" => Some(Level::Intermediate),
        "advanced" => Some(Level::Advanced),
        _ => None,
    }
}

struct FileJob {
    path: PathBuf,
    level: Level,
}

fn verify_file(job: &FileJob, scratch: &PathBuf, answers_only: bool) -> (usize, usize) {
    let mut total = 0usize;
    let mut failed = 0usize;

    let json = match std::fs::read_to_string(&job.path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⚠ {} を読めません ({e}) — スキップ", job.path.display());
            return (0, 0);
        }
    };
    let problems = match load_problems_str(&json) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ {} のパースに失敗: {e}", job.path.display());
            return (0, 1);
        }
    };
    println!("== {} : {} ({} 問) ==", job.path.display(), job.level.label_ja(), problems.len());

    let issues: Vec<ProblemIssue> = validate_static(&problems, job.level);
    for i in &issues {
        eprintln!("✗ [{}] {}", i.id, i.message);
    }
    failed += issues.len();

    for p in &problems {
        total += 1;
        match run_problem(scratch, &p.answer_code, &p.hidden_tests) {
            Ok(r) if r.passed => {}
            Ok(r) => {
                eprintln!("✗ [{}] answer_code がテストを通りません:\n{}", p.id, r.output);
                failed += 1;
                continue;
            }
            Err(e) => {
                eprintln!("✗ [{}] 実行失敗: {e}", p.id);
                failed += 1;
                continue;
            }
        }
        if !answers_only {
            match run_problem(scratch, &p.starter_code, &p.hidden_tests) {
                Ok(r) if !r.passed => {}
                Ok(_) => {
                    eprintln!("✗ [{}] starter_code のままテストが通ってしまいます", p.id);
                    failed += 1;
                    continue;
                }
                Err(e) => {
                    eprintln!("✗ [{}] 実行失敗: {e}", p.id);
                    failed += 1;
                    continue;
                }
            }
        }
        println!("✓ {}", p.id);
    }
    (total, failed)
}

fn main() -> ExitCode {
    let mut levels: Vec<Level> = Vec::new();
    let mut answers_only = false;
    let mut file: Option<PathBuf> = None;
    let mut file_level: Option<Level> = None;
    let mut scratch = PathBuf::from("target/verifier-scratch");

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--answers-only" => answers_only = true,
            "--file" => match args.next() {
                Some(p) => file = Some(PathBuf::from(p)),
                None => {
                    eprintln!("--file にはパスが必要です");
                    return ExitCode::FAILURE;
                }
            },
            "--level" => match args.next().as_deref().and_then(level_from_arg) {
                Some(l) => file_level = Some(l),
                None => {
                    eprintln!("--level には beginner/intermediate/advanced を指定します");
                    return ExitCode::FAILURE;
                }
            },
            "--scratch" => match args.next() {
                Some(p) => scratch = PathBuf::from(p),
                None => {
                    eprintln!("--scratch にはパスが必要です");
                    return ExitCode::FAILURE;
                }
            },
            other => match level_from_arg(other) {
                Some(l) => levels.push(l),
                None => {
                    eprintln!("不明な引数: {other}");
                    return ExitCode::FAILURE;
                }
            },
        }
    }

    let jobs: Vec<FileJob> = if let Some(path) = file {
        let Some(level) = file_level else {
            eprintln!("--file を使うときは --level も指定してください");
            return ExitCode::FAILURE;
        };
        vec![FileJob { path, level }]
    } else {
        if levels.is_empty() {
            levels = Level::ALL.to_vec();
        }
        levels
            .into_iter()
            .map(|level| FileJob {
                path: PathBuf::from("data/problems").join(level.file_name()),
                level,
            })
            .collect()
    };

    let mut total = 0usize;
    let mut failed = 0usize;
    for job in &jobs {
        let (t, f) = verify_file(job, &scratch, answers_only);
        total += t;
        failed += f;
    }

    println!("---\n検証 {total} 問 / 問題あり {failed} 件");
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
