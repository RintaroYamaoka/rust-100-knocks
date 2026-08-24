use serde::{Deserialize, Serialize};

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

    pub fn file_name(&self) -> &'static str {
        match self {
            Level::Beginner => "beginner.json",
            Level::Intermediate => "intermediate.json",
            Level::Advanced => "advanced.json",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Problem {
    pub id: String,
    pub level: Level,
    pub title: String,
    pub description_md: String,
    pub starter_code: String,
    /// ユーザーコードに結合して正誤判定に使う #[test] 群。フロントには配信されるが UI には出さない。
    pub hidden_tests: String,
    pub answer_code: String,
    pub explanation_md: String,
    #[serde(default)]
    pub hints: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// ユーザーコードと判定用テストを 1 つの提出コードに合成する。
/// 区切りコメントはエラー行番号のずれをユーザーが把握できるよう目印になる。
pub fn compose_submission(user_code: &str, hidden_tests: &str) -> String {
    format!("{user_code}\n\n// ===== 判定用テスト (自動付加) =====\n{hidden_tests}\n")
}
