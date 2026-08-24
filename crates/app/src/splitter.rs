//! ドラッグ可能な分割バー。pointer capture でバー外にポインタが出ても追従する。

use leptos::prelude::*;

use crate::layout::{save_layout, Axis, LayoutSizes, SplitTarget};

#[component]
pub fn Splitter(target: SplitTarget, sizes: RwSignal<LayoutSizes>) -> impl IntoView {
    let dragging = RwSignal::new(false);
    // ドラッグ開始時の (ポインタ座標, ペインサイズ)
    let origin = RwSignal::new((0.0f64, 0.0f64));
    // 親 (.main) がドラッグ中にエディタ等の pointer-events を切るための共有フラグ
    let global_dragging: Option<RwSignal<bool>> = use_context();
    let set_dragging = move |v: bool| {
        dragging.set(v);
        if let Some(g) = global_dragging {
            g.set(v);
        }
    };

    let axis_class = match target.axis() {
        Axis::Horizontal => "splitter splitter-h",
        Axis::Vertical => "splitter splitter-v",
    };

    let coord = move |ev: &leptos::ev::PointerEvent| match target.axis() {
        Axis::Horizontal => ev.client_x() as f64,
        Axis::Vertical => ev.client_y() as f64,
    };

    view! {
        <div
            class=axis_class
            class:dragging=move || dragging.get()
            on:pointerdown=move |ev| {
                ev.prevent_default();
                origin.set((coord(&ev), sizes.with_untracked(|s| s.get(target))));
                set_dragging(true);
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::JsCast;
                    if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                        let _ = el.set_pointer_capture(ev.pointer_id());
                    }
                }
            }
            on:pointermove=move |ev| {
                if !dragging.get_untracked() {
                    return;
                }
                let (start_pos, start_size) = origin.get_untracked();
                let delta = coord(&ev) - start_pos;
                sizes.update(|s| s.apply_drag(target, start_size, delta));
            }
            on:pointerup=move |_| {
                if dragging.get_untracked() {
                    set_dragging(false);
                    sizes.with_untracked(save_layout);
                }
            }
            on:pointercancel=move |_| set_dragging(false)
            on:dblclick=move |_| {
                // ダブルクリックで既定サイズに戻す
                let d = LayoutSizes::default();
                sizes.update(|s| match target {
                    SplitTarget::Sidebar => s.sidebar_w = d.sidebar_w,
                    SplitTarget::Problem => s.problem_w = d.problem_w,
                    SplitTarget::Console => s.console_h = d.console_h,
                });
                sizes.with_untracked(save_layout);
            }
            title="ドラッグでサイズ変更 / ダブルクリックで既定に戻す"
        ></div>
    }
}
