use app::layout::{clamp_size, Axis, LayoutSizes, SplitTarget};

#[test]
fn clamp_respects_min_and_max() {
    assert_eq!(clamp_size(100.0, 200.0, 600.0), 200.0);
    assert_eq!(clamp_size(700.0, 200.0, 600.0), 600.0);
    assert_eq!(clamp_size(350.0, 200.0, 600.0), 350.0);
}

#[test]
fn defaults_are_sane() {
    let d = LayoutSizes::default();
    assert!(d.sidebar_w >= 200.0 && d.sidebar_w <= 600.0);
    assert!(d.problem_w >= 280.0);
    assert!(d.console_h >= 120.0);
}

#[test]
fn split_target_axis_and_bounds() {
    assert_eq!(SplitTarget::Sidebar.axis(), Axis::Horizontal);
    assert_eq!(SplitTarget::Problem.axis(), Axis::Horizontal);
    assert_eq!(SplitTarget::Console.axis(), Axis::Vertical);
    let (min, max) = SplitTarget::Console.bounds();
    assert!(min < max);
}

#[test]
fn apply_drag_updates_only_target() {
    let mut s = LayoutSizes::default();
    let before = s.clone();
    // サイドバー境界を右へ 50px ドラッグ
    s.apply_drag(SplitTarget::Sidebar, before.sidebar_w, 50.0);
    assert_eq!(s.sidebar_w, before.sidebar_w + 50.0);
    assert_eq!(s.problem_w, before.problem_w);
    assert_eq!(s.console_h, before.console_h);

    // コンソール境界は上へドラッグ (delta 負) すると高くなる
    let mut s2 = LayoutSizes::default();
    s2.apply_drag(SplitTarget::Console, before.console_h, -40.0);
    assert_eq!(s2.console_h, before.console_h + 40.0);
}

#[test]
fn apply_drag_clamps_to_bounds() {
    let mut s = LayoutSizes::default();
    let (min, max) = SplitTarget::Sidebar.bounds();
    s.apply_drag(SplitTarget::Sidebar, s.sidebar_w, -10_000.0);
    assert_eq!(s.sidebar_w, min);
    s.apply_drag(SplitTarget::Sidebar, s.sidebar_w, 10_000.0);
    assert_eq!(s.sidebar_w, max);
}

#[test]
fn sizes_roundtrip_json() {
    let s = LayoutSizes { sidebar_w: 333.0, problem_w: 444.0, console_h: 222.0 };
    let json = serde_json::to_string(&s).unwrap();
    let back: LayoutSizes = serde_json::from_str(&json).unwrap();
    assert_eq!(back, s);
}
