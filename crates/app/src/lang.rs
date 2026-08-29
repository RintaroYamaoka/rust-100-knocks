//! 言語選択まわりの純ロジック。
//!
//! 「どの言語をセレクタに出すか」「起動時に何を選ぶか」「どの文言を出すか」を
//! UI 配線とネットワークから切り離し、host でテストできる形に置く。

use std::collections::HashSet;

use shared::language::{Backend, Language};
use shared::problem::Level;

/// 選択中の言語を保存する localStorage キー。進捗キーと同じ名前空間に置く。
pub const LANGUAGE_STORAGE_KEY: &str = "rust100knocks.language.v1";

/// 3 レベル分のデータが**実際に取得できた**言語だけを `Language::ALL` の順で返す。
///
/// 未完成の言語 (`data/problems/<lang>/` が無い・レベルが欠けている) をセレクタに
/// 出さないための唯一の判定。1 レベルでも欠けたら出さない。
pub fn languages_with_full_data(found: &HashSet<(Language, Level)>) -> Vec<Language> {
    Language::ALL
        .into_iter()
        .filter(|lang| Level::ALL.iter().all(|lv| found.contains(&(*lang, *lv))))
        .collect()
}

/// 起動時に選ぶ言語。まだ存在確認が済んでいない段階で呼ばれるので、
/// 保存値が読めればそれ、駄目なら Rust (従来からの既定) に落とす。
pub fn initial_language(stored: Option<&str>) -> Language {
    stored
        .and_then(Language::from_slug)
        .unwrap_or(Language::Rust)
}

/// 存在確認が出揃ったあと、選択を有効な言語へ寄せる。
///
/// 一覧が空のときは動かさない。空になるのは「確認そのものが機能しなかった」場合
/// (HEAD が通らない配信環境など) で、そこで選択を触ると今まで動いていた Rust まで
/// 巻き添えで壊れる。
pub fn resolve_selection(current: Language, available: &[Language]) -> Language {
    if available.is_empty() || available.contains(&current) {
        current
    } else {
        available[0]
    }
}

/// 言語セレクタに出す一覧。
///
/// 確認が効かなかった環境 (`available` が空) では、実際にデータが読めている
/// 選択中の言語だけを出す。空の一覧を出して「言語が 1 つも無い」ように
/// 見せるより、確実に動く 1 つを出すほうが実態に近い。
pub fn selector_languages(
    available: &[Language],
    current: Language,
    current_loaded: bool,
) -> Vec<Language> {
    if !available.is_empty() {
        return available.to_vec();
    }
    if current_loaded {
        return vec![current];
    }
    // ここに来るのは「どの言語の存在も確認できず、現在の言語も読めていない」とき。
    // 空の Vec を返すと option が 0 個になり、**別の言語に戻す手段が UI から消える**
    // (保存された言語のデータが無くなった利用者が詰む)。必ず既定の Rust を残す。
    if current == Language::Rust {
        vec![Language::Rust]
    } else {
        vec![Language::Rust, current]
    }
}

/// 実行を委譲する上流サービスの表示名 (コンソールの文言用)。
pub fn backend_label(language: Language) -> &'static str {
    match language.backend() {
        Backend::Playground => "Rust Playground",
        Backend::Wandbox { .. } => "Wandbox",
    }
}

/// 実行前のコンソール placeholder。言語に依らない文言にする。
pub fn console_idle_hint() -> &'static str {
    "コードを書いて「実行して判定」(Ctrl+Enter) を押すと、ここに実物の診断出力と判定結果が表示されます。"
}

/// 実行中のコンソール placeholder。どこで動いているかを言語ごとに正しく出す。
pub fn console_running_hint(language: Language) -> String {
    format!(
        "{} で {} のコードを実行しています…",
        backend_label(language),
        language.label()
    )
}

/// エラーコードを公式ドキュメントへリンクしてよい言語か。
///
/// rustc の `doc.rust-lang.org/error_codes/` に相当する「コード別の公式ページ」は
/// 他の 6 言語には無い。リンクにすると 404 に飛ばすことになるので素のテキストで出す。
pub fn links_error_codes(language: Language) -> bool {
    language == Language::Rust
}
