//! 進捗の永続化。wasm では localStorage、host (テスト) ではメモリ内フォールバック。
//! どちらも同じシリアライズ形式 (ProgressMap の JSON) を通る。

use shared::progress::ProgressMap;

const STORAGE_KEY: &str = "rust100knocks.progress.v1";

pub fn load_progress() -> ProgressMap {
    raw_load()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_progress(map: &ProgressMap) {
    if let Ok(s) = serde_json::to_string(map) {
        raw_save(&s);
    }
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
fn raw_load() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item(STORAGE_KEY).ok()?
}

#[cfg(target_arch = "wasm32")]
fn raw_save(s: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_KEY, s);
    }
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static MEMORY: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[cfg(not(target_arch = "wasm32"))]
fn raw_load() -> Option<String> {
    MEMORY.with(|m| m.borrow().clone())
}

#[cfg(not(target_arch = "wasm32"))]
fn raw_save(s: &str) {
    MEMORY.with(|m| *m.borrow_mut() = Some(s.to_string()));
}
