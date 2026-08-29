use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::language::Language;
use crate::problem::Problem;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProblemStatus {
    /// 一度も判定実行していない
    Unanswered,
    /// 挑戦したがまだ正解していない
    Attempted,
    /// 正解済み
    Passed,
}

/// localStorage に保存する 1 問分の進捗。キーは `progress_key` が組む。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProgressEntry {
    pub status: ProblemStatus,
    /// 編集途中のコード (下書き)。None なら starter_code を表示する。
    #[serde(default)]
    pub saved_code: Option<String>,
    #[serde(default)]
    pub updated_at_ms: f64,
}

impl ProgressEntry {
    pub fn empty() -> Self {
        Self {
            status: ProblemStatus::Unanswered,
            saved_code: None,
            updated_at_ms: 0.0,
        }
    }
}

pub type ProgressMap = HashMap<String, ProgressEntry>;

/// 進捗キーを組む唯一の場所。
///
/// 言語をまたぐと `id` は重複する (`b001` が 7 言語に存在する) ので、言語で名前空間を
/// 切る。キーの組み立てをここ 1 箇所に閉じ、他の関数が `&Problem` を受け取るように
/// してあるのは、素の `id` で進捗を引く経路を型で塞ぐため。素の `id` を使っても
/// コンパイルは通ってしまい、症状は「一覧の進捗色が静かに全部消える」だけになる。
pub fn progress_key(problem: &Problem) -> String {
    format!("{}/{}", problem.language.slug(), problem.id)
}

/// 旧形式 (Rust 専用時代のフラットな `b001`) のキーを `rust/b001` に移行する。
///
/// 移行済みキーが既にある場合は既存を優先する (二重移行で上書きしない)。
pub fn migrate_legacy_keys(map: &mut ProgressMap) -> usize {
    let legacy: Vec<String> = map
        .keys()
        .filter(|k| !k.contains('/'))
        .cloned()
        .collect();
    let mut moved = 0;
    for old in legacy {
        let new = format!("{}/{}", Language::Rust.slug(), old);
        if let Some(entry) = map.remove(&old) {
            map.entry(new).or_insert(entry);
            moved += 1;
        }
    }
    moved
}

pub fn status_of(map: &ProgressMap, problem: &Problem) -> ProblemStatus {
    map.get(&progress_key(problem))
        .map_or(ProblemStatus::Unanswered, |e| e.status)
}

pub fn saved_code_of<'a>(map: &'a ProgressMap, problem: &Problem) -> Option<&'a str> {
    map.get(&progress_key(problem))
        .and_then(|e| e.saved_code.as_deref())
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusFilter {
    All,
    OnlyUnanswered,
    OnlyAttempted,
    OnlyPassed,
}

pub fn matches_filter(status: ProblemStatus, filter: StatusFilter) -> bool {
    match filter {
        StatusFilter::All => true,
        StatusFilter::OnlyUnanswered => status == ProblemStatus::Unanswered,
        StatusFilter::OnlyAttempted => status == ProblemStatus::Attempted,
        StatusFilter::OnlyPassed => status == ProblemStatus::Passed,
    }
}

/// 状態フィルタ + 検索クエリ (id / title / tags、大文字小文字無視) で問題一覧を絞り込む。
pub fn filter_problems<'a>(
    problems: &'a [Problem],
    map: &ProgressMap,
    filter: StatusFilter,
    query: &str,
) -> Vec<&'a Problem> {
    let q = query.trim().to_lowercase();
    problems
        .iter()
        .filter(|p| matches_filter(status_of(map, p), filter))
        .filter(|p| {
            q.is_empty()
                || p.id.to_lowercase().contains(&q)
                || p.title.to_lowercase().contains(&q)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}

/// 一覧に含まれる問題のうち正解済みの数 (達成率表示用)。
pub fn passed_count(problems: &[Problem], map: &ProgressMap) -> usize {
    problems
        .iter()
        .filter(|p| status_of(map, p) == ProblemStatus::Passed)
        .count()
}
