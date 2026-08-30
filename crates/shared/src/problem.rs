use serde::{Deserialize, Serialize};

use crate::language::Language;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}

impl Level {
    pub const ALL: [Level; 3] = [Level::Beginner, Level::Intermediate, Level::Advanced];

    pub fn label_ja(&self) -> &'static str {
        match self {
            Level::Beginner => "初級",
            Level::Intermediate => "中級",
            Level::Advanced => "上級",
        }
    }

    /// 問題 id の先頭 1 文字 (b001 / i001 / a001)
    pub fn id_prefix(&self) -> char {
        match self {
            Level::Beginner => 'b',
            Level::Intermediate => 'i',
            Level::Advanced => 'a',
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Level::Beginner => "beginner",
            Level::Intermediate => "intermediate",
            Level::Advanced => "advanced",
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            Level::Beginner => "beginner.json",
            Level::Intermediate => "intermediate.json",
            Level::Advanced => "advanced.json",
        }
    }

    pub fn from_slug(s: &str) -> Option<Level> {
        Level::ALL.into_iter().find(|l| l.slug() == s)
    }
}

/// 問題データの配置。パスが `language` / `level` の正本で、`Problem` のフィールドは
/// その冗長コピー (verifier が両者の一致を検査する)。
pub fn problems_rel_path(language: Language, level: Level) -> String {
    format!("data/problems/{}/{}", language.slug(), level.file_name())
}

/// 収録済み言語のマニフェスト。ビルド時に scripts/gen-manifest.mjs が生成する。
pub const MANIFEST_URL: &str = "/data/problems/index.json";

/// フロントが取得する URL。
pub fn problems_url(language: Language, level: Level) -> String {
    format!("/data/problems/{}/{}", language.slug(), level.file_name())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Problem {
    pub id: String,
    pub language: Language,
    pub level: Level,
    pub title: String,
    pub description_md: String,
    pub starter_code: String,
    /// ユーザーコードに結合して正誤判定に使うコード。フロントには配信されるが UI には出さない。
    pub hidden_tests: String,
    pub answer_code: String,
    pub explanation_md: String,
    #[serde(default)]
    pub hints: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// ユーザーコードと判定用テストを 1 つの提出コードに合成する。
///
/// ユーザーコードを**先**に置くのは、コンパイラ診断の行番号をユーザーが書いた行と
/// 一致させるため。区切りコメントの記号は言語別 — Python に `//` を入れると
/// SyntaxError になり、その言語の問題が全滅する。
pub fn compose_submission(language: Language, user_code: &str, hidden_tests: &str) -> String {
    let c = language.line_comment();
    format!("{user_code}\n\n{c} ===== 判定用テスト (自動付加) =====\n{hidden_tests}\n")
}
