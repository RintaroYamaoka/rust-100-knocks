//! Zed 風のドラッグ可能なペイン分割。サイズは CSS 変数として root に流し込み、
//! ドラッグ計算 (クランプ) はここで純ロジックとして持つ (host でテスト可能)。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitTarget {
    /// サイドバー | 問題ペイン
    Sidebar,
    /// 問題ペイン | ワークベンチ
    Problem,
    /// エディタ / コンソール (縦)
    Console,
}

impl SplitTarget {
    pub fn axis(self) -> Axis {
        match self {
            SplitTarget::Sidebar | SplitTarget::Problem => Axis::Horizontal,
            SplitTarget::Console => Axis::Vertical,
        }
    }

    /// (min, max) px
    pub fn bounds(self) -> (f64, f64) {
        match self {
            SplitTarget::Sidebar => (200.0, 600.0),
            SplitTarget::Problem => (280.0, 1200.0),
            SplitTarget::Console => (120.0, 800.0),
        }
    }
}

pub fn clamp_size(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LayoutSizes {
    pub sidebar_w: f64,
    pub problem_w: f64,
    pub console_h: f64,
}

impl Default for LayoutSizes {
    fn default() -> Self {
        Self {
            sidebar_w: 300.0,
            problem_w: 560.0,
            console_h: 260.0,
        }
    }
}

impl LayoutSizes {
    /// ドラッグ開始時のサイズ `start` に対して、ポインタ移動量 `delta` (px) を適用する。
    /// コンソールは画面下端に張り付いているため、上へ動かす (負の delta) と大きくなる。
    pub fn apply_drag(&mut self, target: SplitTarget, start: f64, delta: f64) {
        let (min, max) = target.bounds();
        match target {
            SplitTarget::Sidebar => self.sidebar_w = clamp_size(start + delta, min, max),
            SplitTarget::Problem => self.problem_w = clamp_size(start + delta, min, max),
            SplitTarget::Console => self.console_h = clamp_size(start - delta, min, max),
        }
    }

    pub fn get(&self, target: SplitTarget) -> f64 {
        match target {
            SplitTarget::Sidebar => self.sidebar_w,
            SplitTarget::Problem => self.problem_w,
            SplitTarget::Console => self.console_h,
        }
    }

    /// `.main` 要素の style 属性に流し込む CSS 変数文字列
    pub fn css_vars(&self) -> String {
        format!(
            "--sidebar-w:{}px;--problem-w:{}px;--console-h:{}px",
            self.sidebar_w, self.problem_w, self.console_h
        )
    }
}

const STORAGE_KEY: &str = "rust100knocks.layout.v1";

pub fn load_layout() -> LayoutSizes {
    crate::storage::raw_get(STORAGE_KEY)
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_layout(sizes: &LayoutSizes) {
    if let Ok(s) = serde_json::to_string(sizes) {
        let _ = crate::storage::raw_set(STORAGE_KEY, &s);
    }
}
