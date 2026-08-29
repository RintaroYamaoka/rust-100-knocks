//! 永続化。wasm では localStorage、host (テスト) ではメモリ内フォールバック。
//! どちらも同じシリアライズ形式を通る。

use shared::language::Language;
use shared::progress::{migrate_legacy_keys, ProgressMap};

use crate::lang::LANGUAGE_STORAGE_KEY;

const PROGRESS_KEY: &str = "rust100knocks.progress.v1";

pub fn load_progress() -> ProgressMap {
    raw_get(PROGRESS_KEY)
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 起動時の読み込み口。旧フラットキー (`b001`) を `rust/b001` に移行し、
/// 移行が起きたらその場で保存し直す。
///
/// 多言語化前からの利用者の進捗はこの 1 回で救われる。移行し忘れると症状は
/// 「一覧の正解マークが静かに全部消える」なので、読み込み経路を 1 本に絞る。
pub fn load_progress_migrated() -> ProgressMap {
    let mut map = load_progress();
    if migrate_legacy_keys(&mut map) > 0 {
        let _ = save_progress(&map);
    }
    map
}

/// 前回選択していた言語 (無ければ None)。
pub fn load_language() -> Option<String> {
    raw_get(LANGUAGE_STORAGE_KEY)
}

pub fn save_language(language: Language) {
    let _ = raw_set(LANGUAGE_STORAGE_KEY, language.slug());
}

/// 進捗を保存する。**保存できたかを返す。**
///
/// 問題数が 300 → 2100 になったので、下書きを溜めた利用者が localStorage の
/// 5MB クォータに届きうる。到達後は書き込みが例外で落ちるが、握り潰すと
/// 画面は成功時と同じままで、リロードして初めて全部消えていたと気づくことになる。
pub fn save_progress(map: &ProgressMap) -> bool {
    if let Ok(s) = serde_json::to_string(map) {
        return raw_set(PROGRESS_KEY, &s);
    }
    false
}

/// 現在時刻 (epoch ms)。進捗の updated_at_ms に使う。
pub fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn raw_get(key: &str) -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item(key).ok()?
}

/// localStorage への書き込み。成功したかを返す。
///
/// 戻り値を捨ててはいけない。問題数が 300 → 2100 に増えたことで、下書きを溜めた
/// 利用者が 5MB のクォータに到達しうる。到達後は `QuotaExceededError` が投げられて
/// **無言で捨てられ**、正解マークも下書きも保存されなくなるが、画面は成功時と
/// 何も変わらないので、リロードして初めて気づくことになる。
#[cfg(target_arch = "wasm32")]
pub fn raw_set(key: &str, s: &str) -> bool {
    match web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        Some(storage) => storage.set_item(key, s).is_ok(),
        None => false,
    }
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static MEMORY: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn raw_get(key: &str) -> Option<String> {
    MEMORY.with(|m| m.borrow().get(key).cloned())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn raw_set(key: &str, s: &str) -> bool {
    MEMORY.with(|m| {
        m.borrow_mut().insert(key.to_string(), s.to_string());
    });
    true
}
