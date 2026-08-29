use std::path::Path;

use shared::language::Language;
use verifier::docker::{container_script, plan_batch, CaseKind, RunCase};

fn cases(n: usize) -> Vec<RunCase> {
    (1..=n)
        .flat_map(|i| {
            [CaseKind::Answer, CaseKind::Starter].into_iter().map(move |k| RunCase {
                problem_id: format!("b{i:03}"),
                kind: k,
                code: "// code".into(),
            })
        })
        .collect()
}

#[test]
fn a_whole_batch_needs_exactly_one_container() {
    // 1 問ごとにコンテナを起こすと 1800 問 × 2 で起動オーバーヘッドだけで数時間かかる。
    // バッチ 1 ファイル = コンテナ 1 回であることをここで固定する。
    for lang in Language::ALL.into_iter().filter(|l| *l != Language::Rust) {
        let plan = plan_batch(lang, &cases(20), Path::new("/tmp/w"));
        assert_eq!(plan.len(), 1, "{} が {} 回コンテナを起こしている", lang.slug(), plan.len());
    }
}

#[test]
fn container_count_does_not_grow_with_problem_count() {
    let small = plan_batch(Language::Cpp, &cases(1), Path::new("/tmp/w")).len();
    let large = plan_batch(Language::Cpp, &cases(100), Path::new("/tmp/w")).len();
    assert_eq!(small, large);
}

#[test]
fn rust_is_planned_locally_not_in_docker() {
    // Rust はローカル cargo が速いので Docker に載せない (ADR 0002)
    assert!(plan_batch(Language::Rust, &cases(20), Path::new("/tmp/w")).is_empty());
}

#[test]
fn empty_batch_plans_no_container() {
    assert!(plan_batch(Language::Cpp, &[], Path::new("/tmp/w")).is_empty());
}

#[test]
fn plan_uses_the_pinned_image_for_the_language() {
    let plan = plan_batch(Language::Java, &cases(3), Path::new("/tmp/w"));
    assert_eq!(plan[0].image, "eclipse-temurin:22-jdk");
    let plan = plan_batch(Language::Csharp, &cases(3), Path::new("/tmp/w"));
    assert_eq!(plan[0].image, "mcr.microsoft.com/dotnet/sdk:6.0");
}

#[test]
fn plan_is_network_isolated() {
    // 提出コードは信頼できない。ネットワークを与えない
    let plan = plan_batch(Language::Python, &cases(3), Path::new("/tmp/w"));
    assert!(plan[0].network_disabled, "コンテナにネットワークが残っている");
}

#[test]
fn every_case_gets_a_per_case_timeout() {
    // 無限ループを書いた 1 問がバッチ全体を永久にブロックしないこと。
    // これは wall-clock しか焼かないので、再試行上限もトークン上限も発火しない
    for lang in Language::ALL.into_iter().filter(|l| *l != Language::Rust) {
        let script = container_script(lang, &cases(2));
        assert!(script.contains("timeout "), "{} のスクリプトに timeout が無い", lang.slug());
    }
}

#[test]
fn script_visits_every_case_directory() {
    let script = container_script(Language::Cpp, &cases(2));
    for id in ["b001", "b002"] {
        for suffix in ["answer", "starter"] {
            assert!(script.contains(&format!("{id}-{suffix}")), "{id}-{suffix} が抜けている:\n{script}");
        }
    }
}

#[test]
fn script_records_exit_code_and_streams_separately() {
    let script = container_script(Language::Python, &cases(1));
    // stdout と stderr を混ぜると「目印が stdout にあるか」を判定できなくなる
    assert!(script.contains("_stdout"));
    assert!(script.contains("_stderr"));
    assert!(script.contains("_exit"));
}

#[test]
fn csharp_script_creates_the_project_once_outside_the_loop() {
    // dotnet new をケースごとに走らせると 40 回 × 3 秒でバッチが崩壊する
    let script = container_script(Language::Csharp, &cases(20));
    assert_eq!(script.matches("dotnet new console").count(), 1, "{script}");
}

#[test]
fn scripts_use_the_language_source_file_name() {
    for lang in Language::ALL.into_iter().filter(|l| *l != Language::Rust) {
        let script = container_script(lang, &cases(1));
        assert!(
            script.contains(lang.source_file_name()),
            "{} のスクリプトが {} を使っていない",
            lang.slug(),
            lang.source_file_name()
        );
    }
}
