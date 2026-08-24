use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

/// localStorage に保存する 1 問分の進捗。キーは problem id。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProgressEntry {
    pub status: ProblemStatus,
    /// 編集途中のコード (下書き)。None なら starter_code を表示する。
    #[serde(default)]
    pub saved_code: Option<String>,
    #[serde(default)]
    pub updated_at_ms: f64,
}

pub type ProgressMap = HashMap<String, ProgressEntry>;

pub fn status_of(map: &ProgressMap, id: &str) -> ProblemStatus {
    map.get(id).map_or(ProblemStatus::Unanswered, |e| e.status)
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
        .filter(|p| matches_filter(status_of(map, &p.id), filter))
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
        .filter(|p| status_of(map, &p.id) == ProblemStatus::Passed)
        .count()
}
