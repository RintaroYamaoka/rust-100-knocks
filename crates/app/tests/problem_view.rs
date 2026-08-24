use app::problem_view::answer_visible;
use shared::progress::ProblemStatus;

#[test]
fn answer_gate_semantics() {
    // 正解済み → 常に閲覧可
    assert!(answer_visible(ProblemStatus::Passed, false));
    // 未正解でも明示的に開示した場合は閲覧可 (ネタバレ防止は既定値のみ)
    assert!(answer_visible(ProblemStatus::Attempted, true));
    assert!(!answer_visible(ProblemStatus::Attempted, false));
    assert!(!answer_visible(ProblemStatus::Unanswered, false));
}
