//! CodeMirror 6 (assets/js/editor.js の window.RustKnocksEditor) への interop 層。
//! host ビルドでは no-op スタブになり、実挙動は wasm + 実ブラウザでのみ成立する。

use shared::language::Language;

#[cfg(target_arch = "wasm32")]
mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_namespace = ["window", "RustKnocksEditor"])]
    extern "C" {
        pub fn mount(parent_id: &str, initial_code: &str, language_slug: &str) -> bool;
        #[wasm_bindgen(js_name = setValue)]
        pub fn set_value(code: &str);
        #[wasm_bindgen(js_name = setLanguage)]
        pub fn set_language(mode: &str);
        #[wasm_bindgen(js_name = getValue)]
        pub fn get_value() -> String;
        pub fn focus();
        #[wasm_bindgen(js_name = setOnRun)]
        pub fn set_on_run(cb: &js_sys::Function);
        #[wasm_bindgen(js_name = setOnSave)]
        pub fn set_on_save(cb: &js_sys::Function);
        #[wasm_bindgen(js_name = setOnChange)]
        pub fn set_on_change(cb: &js_sys::Function);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn ready() -> bool {
    web_sys::window()
        .map(|w| js_sys::Reflect::has(&w, &wasm_bindgen::JsValue::from_str("RustKnocksEditor")).unwrap_or(false))
        .unwrap_or(false)
}

/// glue script のロード完了を待ってエディタをマウントする (80ms 間隔でリトライ)。
#[cfg(target_arch = "wasm32")]
pub fn mount_retrying(
    parent_id: &'static str,
    initial_code: String,
    language: Language,
    on_mounted: std::rc::Rc<dyn Fn()>,
) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    // 最初から正しい言語で立ち上げる。マウント後に setLanguage で直すと、
    // 一瞬だけ別の言語のハイライトで描かれる
    if ready() && ffi::mount(parent_id, &initial_code, language.slug()) {
        on_mounted();
        return;
    }
    let cb =
        Closure::once_into_js(move || mount_retrying(parent_id, initial_code, language, on_mounted));
    if let Some(w) = web_sys::window() {
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 80);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn set_value(code: &str) {
    if ready() {
        ffi::set_value(code);
    }
}

/// エディタのシンタックスハイライトを言語に追従させる。
///
/// **再マウントはしない**。JS 側は CodeMirror の Compartment を reconfigure する
/// 契約で、再マウントすると編集中の下書きが失われる。JS 側が未実装 (旧 glue) でも
/// ここが panic しないよう、関数の存在を確認してから呼ぶ。
#[cfg(target_arch = "wasm32")]
pub fn set_language(language: Language) {
    if has_fn("setLanguage") {
        ffi::set_language(language.editor_mode());
    }
}

/// `window.RustKnocksEditor` に指定名の関数があるか。
#[cfg(target_arch = "wasm32")]
fn has_fn(name: &str) -> bool {
    use wasm_bindgen::JsValue;
    let Some(w) = web_sys::window() else {
        return false;
    };
    let Ok(ed) = js_sys::Reflect::get(&w, &JsValue::from_str("RustKnocksEditor")) else {
        return false;
    };
    js_sys::Reflect::get(&ed, &JsValue::from_str(name))
        .map(|v| v.is_function())
        .unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
pub fn get_value() -> String {
    if ready() {
        ffi::get_value()
    } else {
        String::new()
    }
}

#[cfg(target_arch = "wasm32")]
pub fn focus() {
    if ready() {
        ffi::focus();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn on_run(f: impl Fn() + 'static) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    let closure = Closure::wrap(Box::new(f) as Box<dyn Fn()>);
    ffi::set_on_run(closure.as_ref().unchecked_ref());
    closure.forget();
}

#[cfg(target_arch = "wasm32")]
pub fn on_save(f: impl Fn() + 'static) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    let closure = Closure::wrap(Box::new(f) as Box<dyn Fn()>);
    ffi::set_on_save(closure.as_ref().unchecked_ref());
    closure.forget();
}

#[cfg(target_arch = "wasm32")]
pub fn on_change(f: impl Fn(String) + 'static) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    let closure = Closure::wrap(Box::new(f) as Box<dyn Fn(String)>);
    ffi::set_on_change(closure.as_ref().unchecked_ref());
    closure.forget();
}

// ---- host スタブ (テスト用 no-op) ----

#[cfg(not(target_arch = "wasm32"))]
pub fn ready() -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount_retrying(
    _parent_id: &'static str,
    _initial_code: String,
    _language: Language,
    _on_mounted: std::rc::Rc<dyn Fn()>,
) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_value(_code: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_language(_language: Language) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_value() -> String {
    String::new()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn focus() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_run(_f: impl Fn() + 'static) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_save(_f: impl Fn() + 'static) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_change(_f: impl Fn(String) + 'static) {}
