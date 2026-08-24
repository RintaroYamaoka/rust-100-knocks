//! 使い方:
//!   cargo run -p verifier                 # data/problems/ の全レベルを検証
//!   cargo run -p verifier -- beginner     # 特定レベルのみ
//!   cargo run -p verifier -- --answers-only   # starter が落ちる検査を省略 (高速化)

use std::path::PathBuf;
use std::process::ExitCode;

use shared::problem::Level;
use verifier::{load_problems_str, run_problem, validate_static};

fn level_from_arg(arg: &str) -> Option<Level> {
    match arg {
        "beginner" => Some(Level::Beginner),
        "intermediate" => Some(Level::Intermediate),
        "advanced" => Some(Level::Advanced),
        _ => None,
    }
}

fn main() -> ExitCode {
    let mut levels: Vec<Level> = Vec::new();
    let mut answers_only = false;
    for arg in std::env::args().skip(1) {
        if arg == "--answers-only" {
            answers_only = true;
        } else if let Some(l) = level_from_arg(&arg) {
            levels.push(l);
        } else {
            eprintln!("不明な引数: {arg}");
            return ExitCode::FAILURE;
        }
    }
    if levels.is_empty() {
        levels = Level::ALL.to_vec();
    }

    let scratch = PathBuf::from("target/verifier-scratch");
    let mut total = 0usize;
    let mut failed = 0usize;

    for level in levels {
        let path = PathBuf::from("data/problems").join(level.file_name());
        let json = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("⚠ {} を読めません ({e}) — スキップ", path.display());
                continue;
            }
        };
        let problems = match load_problems_str(&json) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("✗ {} のパースに失敗: {e}", path.display());
                failed += 1;
                continue;
            }
        };
        println!("== {} ({} 問) ==", level.label_ja(), problems.len());

        let issues = validate_static(&problems, level);
        for i in &issues {
            eprintln!("✗ [{}] {}", i.id, i.message);
        }
        failed += issues.len();

        for p in &problems {
            total += 1;
            match run_problem(&scratch, &p.answer_code, &p.hidden_tests) {
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
                match run_problem(&scratch, &p.starter_code, &p.hidden_tests) {
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
    }

    println!("---\n検証 {total} 問 / 問題あり {failed} 件");
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
