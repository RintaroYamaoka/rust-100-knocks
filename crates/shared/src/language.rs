//! 対応言語と、その言語がどの実行バックエンドで動くか。
//! バックエンド選定とコンパイラ ID の正本は ADR 0002。

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Cpp,
    Csharp,
    Java,
    Python,
    Typescript,
    Javascript,
}

/// 実行を委譲する上流サービス。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// play.rust-lang.org/execute (Rust のみ)
    Playground,
    /// wandbox.org/api/compile.json
    Wandbox {
        compiler: &'static str,
        /// Wandbox の `options` (コンパイラフラグ)。不要なら None。
        options: Option<&'static str>,
    },
}

impl Language {
    pub const ALL: [Language; 7] = [
        Language::Rust,
        Language::Cpp,
        Language::Csharp,
        Language::Java,
        Language::Python,
        Language::Typescript,
        Language::Javascript,
    ];

    /// URL / ディレクトリ名 / localStorage キーに使う識別子。JSON 表現と一致する。
    pub fn slug(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Cpp => "cpp",
            Language::Csharp => "csharp",
            Language::Java => "java",
            Language::Python => "python",
            Language::Typescript => "typescript",
            Language::Javascript => "javascript",
        }
    }

    /// UI 表示名。
    pub fn label(self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Cpp => "C++",
            Language::Csharp => "C#",
            Language::Java => "Java",
            Language::Python => "Python",
            Language::Typescript => "TypeScript",
            Language::Javascript => "JavaScript",
        }
    }

    pub fn from_slug(s: &str) -> Option<Language> {
        Language::ALL.into_iter().find(|l| l.slug() == s)
    }

    /// CodeMirror の言語モード識別子 (assets/js/editor-src.mjs が解釈する)。
    pub fn editor_mode(self) -> &'static str {
        self.slug()
    }

    /// 行コメントの開始記号。判定テストの区切りコメントに使う。
    /// Python に `//` を入れると SyntaxError になるので、ここを言語別にするのは必須。
    pub fn line_comment(self) -> &'static str {
        match self {
            Language::Python => "#",
            _ => "//",
        }
    }

    /// 提出コードをどこで実行するか (ADR 0002)。
    pub fn backend(self) -> Backend {
        match self {
            Language::Rust => Backend::Playground,
            Language::Cpp => Backend::Wandbox {
                compiler: "gcc-13.2.0",
                options: Some("warning,c++17"),
            },
            Language::Csharp => Backend::Wandbox {
                compiler: "dotnetcore-6.0.425",
                options: None,
            },
            Language::Java => Backend::Wandbox {
                compiler: "openjdk-jdk-22+36",
                options: None,
            },
            Language::Python => Backend::Wandbox {
                compiler: "cpython-3.13.8",
                options: None,
            },
            Language::Typescript => Backend::Wandbox {
                compiler: "typescript-5.6.2",
                options: None,
            },
            Language::Javascript => Backend::Wandbox {
                compiler: "nodejs-20.17.0",
                options: None,
            },
        }
    }

    /// 検証 (verifier) で使う Docker イメージ。Rust はローカル cargo なので None。
    /// 版は backend() の上流コンパイラに合わせてある (ADR 0002)。
    pub fn verify_image(self) -> Option<&'static str> {
        match self {
            Language::Rust => None,
            Language::Cpp => Some("gcc:13"),
            Language::Csharp => Some("mcr.microsoft.com/dotnet/sdk:6.0"),
            Language::Java => Some("eclipse-temurin:22-jdk"),
            Language::Python => Some("python:3.13"),
            Language::Typescript => Some("knocks-ts:5.6.2"),
            Language::Javascript => Some("node:20"),
        }
    }

    /// 提出コードのファイル名。上流・検証コンテナの双方で同じ名前を使う。
    ///
    /// Java が `prog.java` なのは Wandbox の制約で、これが「問題中のクラスを
    /// public にできない」理由になっている (ADR 0002)。
    pub fn source_file_name(self) -> &'static str {
        match self {
            Language::Rust => "lib.rs",
            Language::Cpp => "prog.cc",
            Language::Csharp => "prog.cs",
            Language::Java => "prog.java",
            Language::Python => "prog.py",
            Language::Typescript => "prog.ts",
            Language::Javascript => "prog.js",
        }
    }
}
