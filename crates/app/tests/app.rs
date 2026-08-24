//! app.rs のうち host で検証できる純ロジック (進捗遷移規則) のテスト。
//! UI 配線は Playwright スクリーンショットで確認する。

use app::next_status;
use shared::playground::Outcome;
use shared::progress::ProblemStatus;

#[test]
fn passing_always_marks_passed() {
    assert_eq!(next_status(ProblemStatus::Unanswered, Outcome::Passed), ProblemStatus::Passed);
    assert_eq!(next_status(ProblemStatus::Attempted, Outcome::Passed), ProblemStatus::Passed);
}

#[test]
fn failure_marks_attempted_but_never_downgrades_passed() {
    assert_eq!(
        next_status(ProblemStatus::Unanswered, Outcome::CompileError),
        ProblemStatus::Attempted
    );
    assert_eq!(
        next_status(ProblemStatus::Unanswered, Outcome::TestsFailed),
        ProblemStatus::Attempted
    );
    // 一度正解した問題はやり直して失敗しても「正解済み」のまま
    assert_eq!(next_status(ProblemStatus::Passed, Outcome::TestsFailed), ProblemStatus::Passed);
}
