use app::console::{split_error_codes, ConsoleSegment};

#[test]
fn splits_line_around_error_codes() {
    let segs = split_error_codes("error[E0308]: mismatched types");
    assert_eq!(
        segs,
        vec![
            ConsoleSegment::Text("error[".into()),
            ConsoleSegment::ErrorCode("E0308".into()),
            ConsoleSegment::Text("]: mismatched types".into()),
        ]
    );
}

#[test]
fn plain_line_is_single_text_segment() {
    let segs = split_error_codes("   Compiling playground v0.0.1");
    assert_eq!(segs, vec![ConsoleSegment::Text("   Compiling playground v0.0.1".into())]);
}

#[test]
fn multiple_codes_in_one_line() {
    let segs = split_error_codes("see error[E0308] and error[E0502]");
    let codes: Vec<_> = segs
        .iter()
        .filter_map(|s| match s {
            ConsoleSegment::ErrorCode(c) => Some(c.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(codes, vec!["E0308", "E0502"]);
}

#[test]
fn bracket_without_valid_code_stays_text() {
    let segs = split_error_codes("error[X123]: not a code");
    assert!(segs.iter().all(|s| matches!(s, ConsoleSegment::Text(_))));
}
