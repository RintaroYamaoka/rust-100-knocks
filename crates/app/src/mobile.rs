//! スマホ幅のときだけ効く「1 画面 1 ペイン」の状態。
//!
//! 3 ペイン (一覧 / 問題 / ワークベンチ) を 375px 幅に縦積みすると、エディタに
//! 辿り着くまで数画面ぶんスクロールすることになり、問題文とコードを行き来する
//! たびに位置を見失う。狭い画面では 1 ペインだけを全画面で見せ、下部タブで
//! 切り替える (= ネイティブアプリと同じ操作) 形にする。
//!
//! どのペインを見せるかの**表示**は CSS (`.main[data-pane=...]`) が持ち、
//! ここは「いまどれか」と「移動したらどれになるか」だけを持つ。
//! デスクトップ幅ではこの状態は画面に影響しない (CSS 側が参照しない)。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobilePane {
    List,
    Problem,
    Code,
}

impl MobilePane {
    /// 下部タブの並び順。CSS の `.main[data-pane=...]` もこの slug で引く。
    pub const ALL: [MobilePane; 3] = [MobilePane::List, MobilePane::Problem, MobilePane::Code];

    pub fn slug(self) -> &'static str {
        match self {
            MobilePane::List => "list",
            MobilePane::Problem => "problem",
            MobilePane::Code => "code",
        }
    }

    pub fn label_ja(self) -> &'static str {
        match self {
            MobilePane::List => "一覧",
            MobilePane::Problem => "問題",
            MobilePane::Code => "コード",
        }
    }
}

/// 問題を移動したあとに見せるペイン。
///
/// 一覧で選んだときも、前後移動 (← / → / 「次の問題へ」) のときも、まず問題文を
/// 見せる。移動直後にコードだけが差し替わると、何の問題を解いているのか分からない
/// まま書き始めることになるため。
pub fn pane_after_problem_change() -> MobilePane {
    MobilePane::Problem
}
