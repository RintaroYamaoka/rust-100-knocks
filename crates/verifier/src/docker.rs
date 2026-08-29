//! 検証の実行層。**`docker run` を組み立てる唯一の場所**。
//!
//! ここに閉じてあるのは、「バッチ 1 ファイルにつきコンテナ 1 回」を数えられる形で
//! 保証するため (1 問ごとに起こすと、1800 問 × 2 の起動オーバーヘッドだけで
//! 数時間かかる)。`plan_batch` は実行せず計画だけ返すので、回数はテストで固定できる。

use std::path::{Path, PathBuf};
use std::process::Command;

use shared::language::Language;

/// 1 問につき 2 通り実行する: 模範解答は通り、初期コードは落ちること。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseKind {
    Answer,
    Starter,
}

impl CaseKind {
    pub fn suffix(self) -> &'static str {
        match self {
            CaseKind::Answer => "answer",
            CaseKind::Starter => "starter",
        }
    }
}

/// 実行 1 件 (= 1 問の answer か starter)。
#[derive(Clone, Debug)]
pub struct RunCase {
    pub problem_id: String,
    pub kind: CaseKind,
    /// 合成済みの提出コード (ユーザーコード + hidden_tests)
    pub code: String,
}

impl RunCase {
    /// 作業ディレクトリ内でこのケースが使うフォルダ名。
    pub fn dir_name(&self) -> String {
        format!("{}-{}", self.problem_id, self.kind.suffix())
    }
}

/// コンテナ 1 回分の実行計画。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockerRun {
    pub image: String,
    /// ホスト側の作業ディレクトリ (コンテナの /w にマウントする)
    pub workdir: PathBuf,
    /// コンテナ内で実行する sh スクリプト
    pub script: String,
    pub network_disabled: bool,
    /// コンテナ全体の上限秒数 (ホスト側のハングに対する保険)
    pub overall_timeout_secs: u64,
}

/// 1 ケースあたりの実行上限。無限ループを書いた問題でバッチが止まらないようにする。
pub const CASE_TIMEOUT_SECS: u64 = 20;
/// コンテナ全体の上限。ケース数に比例させる。
pub const OVERALL_TIMEOUT_BASE_SECS: u64 = 120;

/// バッチをコンテナ何回で回すかの計画を返す。**実行はしない。**
///
/// Rust はローカル cargo の方が速いので Docker に載せない (空の計画を返す)。
pub fn plan_batch(language: Language, cases: &[RunCase], workdir: &Path) -> Vec<DockerRun> {
    if cases.is_empty() {
        return Vec::new();
    }
    let Some(image) = language.verify_image() else {
        return Vec::new(); // Rust
    };
    vec![DockerRun {
        image: image.to_string(),
        workdir: workdir.to_path_buf(),
        script: container_script(language, cases),
        network_disabled: true,
        overall_timeout_secs: OVERALL_TIMEOUT_BASE_SECS + CASE_TIMEOUT_SECS * cases.len() as u64,
    }]
}

/// コンテナ内で走らせる sh スクリプトを組む。
///
/// 各ケースのディレクトリで「コンパイル → 実行」を行い、`_stdout` / `_stderr` / `_exit`
/// に結果を残す。stdout と stderr を混ぜないのは、正解の目印 (`test result: ok`) が
/// stdout にあることを判定条件にしているため。
pub fn container_script(language: Language, cases: &[RunCase]) -> String {
    let src = language.source_file_name();
    let t = CASE_TIMEOUT_SECS;

    // 1 ケース分の「コンパイルして実行する」コマンド。
    // 出力は呼び出し側でリダイレクトする。
    let run_one = match language {
        Language::Cpp => format!("g++ -std=c++17 -w -o _prog {src} && ./_prog"),
        Language::Java => format!("javac -nowarn {src} && java Main"),
        Language::Python => format!("python3 {src}"),
        Language::Javascript => format!("node {src}"),
        Language::Typescript => format!("tsc --target es2020 {src} && node prog.js"),
        // C# はプロジェクトが要る。プロジェクト作成はループの外で 1 回だけ行う
        Language::Csharp => format!(
            "cp {src} /proj/Program.cs && (cd /proj && dotnet build -v q --nologo -o /proj/_out >/dev/null) && dotnet /proj/_out/proj.dll"
        ),
        Language::Rust => String::new(), // Docker では走らせない
    };

    let mut s = String::from("#!/bin/sh\n# 自動生成 (verifier)\n");
    if language == Language::Csharp {
        s.push_str(
            "export DOTNET_CLI_TELEMETRY_OPTOUT=1\nexport DOTNET_NOLOGO=1\nexport DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1\n\
             dotnet new console -o /proj >/dev/null 2>&1\n",
        );
    }

    for case in cases {
        let dir = case.dir_name();
        s.push_str(&format!(
            "cd /w/cases/{dir} 2>/dev/null && {{ timeout {t} sh -c '{run_one}' >_stdout 2>_stderr; echo $? >_exit; }}\n"
        ));
    }
    s.push_str("exit 0\n");
    s
}

/// 計画を実際に走らせる。呼ぶ側は `plan_batch` の結果をそのまま渡す。
pub fn execute(run: &DockerRun) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("timeout");
    cmd.arg(run.overall_timeout_secs.to_string())
        .arg("docker")
        .arg("run")
        .arg("--rm");
    if run.network_disabled {
        cmd.arg("--network=none");
    }
    cmd.arg("-v")
        .arg(format!("{}:/w", run.workdir.display()))
        .arg("-w")
        .arg("/w")
        .arg(&run.image)
        .arg("sh")
        .arg("/w/run.sh");
    cmd.output()
}

/// Docker と必要なイメージが揃っているかを着手時に 1 回だけ検査する。
/// 揃っていないまま進むと「検証したつもりの未検証データ」が積み上がる。
pub fn preflight(languages: &[Language]) -> Result<(), String> {
    let ok = Command::new("docker")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        return Err("docker が使えません (デーモンが起動しているか確認してください)".into());
    }
    let mut missing = Vec::new();
    for l in languages {
        let Some(image) = l.verify_image() else { continue };
        let found = Command::new("docker")
            .args(["image", "inspect", image])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !found {
            missing.push(format!("{} ({})", image, l.slug()));
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "検証イメージがありません: {}\n  docker pull で取得してください (knocks-ts:5.6.2 は node:20 + typescript@5.6.2 の自前ビルド)",
            missing.join(", ")
        ))
    }
}
