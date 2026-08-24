use shared::playground::{
    classify, classify_line, extract_error_codes, validate, ExecuteRequest, ExecuteResponse,
    LineKind, Outcome, MAX_CODE_BYTES,
};

fn resp(success: bool, stdout: &str, stderr: &str) -> ExecuteResponse {
    ExecuteResponse {
        success,
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    }
}

#[test]
fn judge_request_targets_tests_mode() {
    let r = ExecuteRequest::judge("fn f() {}");
    assert!(r.tests);
    assert_eq!(r.crate_type, "lib");
    assert_eq!(r.channel, "stable");
    assert_eq!(r.edition, "2024");
}

#[test]
fn run_request_targets_bin_mode() {
    let r = ExecuteRequest::run("fn main() {}");
    assert!(!r.tests);
    assert_eq!(r.crate_type, "bin");
}

#[test]
fn request_serializes_crate_type_as_camel_case() {
    let s = serde_json::to_string(&ExecuteRequest::judge("x")).unwrap();
    assert!(s.contains("\"crateType\":\"lib\""));
}

#[test]
fn validate_rejects_oversized_code() {
    let mut r = ExecuteRequest::judge("");
    r.code = "a".repeat(MAX_CODE_BYTES + 1);
    assert!(validate(&r).is_err());
}

#[test]
fn validate_rejects_unknown_channel_or_edition() {
    let mut r = ExecuteRequest::judge("fn f() {}");
    r.channel = "custom".into();
    assert!(validate(&r).is_err());
    let mut r = ExecuteRequest::judge("fn f() {}");
    r.edition = "1999".into();
    assert!(validate(&r).is_err());
}

#[test]
fn validate_accepts_default_judge_request() {
    assert!(validate(&ExecuteRequest::judge("fn f() {}")).is_ok());
}

#[test]
fn classify_success_is_passed() {
    assert_eq!(classify(&resp(true, "test result: ok", "")), Outcome::Passed);
}

#[test]
fn classify_rustc_error_is_compile_error() {
    let stderr = "   Compiling playground v0.0.1\nerror[E0308]: mismatched types\n --> src/lib.rs:2:5";
    assert_eq!(classify(&resp(false, "", stderr)), Outcome::CompileError);
}

#[test]
fn classify_test_failure_is_tests_failed() {
    let stdout = "running 1 test\ntest passes ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed";
    let stderr = "   Compiling playground v0.0.1\n    Finished dev profile";
    assert_eq!(classify(&resp(false, stdout, stderr)), Outcome::TestsFailed);
}

#[test]
fn classify_panic_without_tests_is_runtime_error() {
    let stderr = "thread 'main' panicked at src/main.rs:2:5:\nboom";
    assert_eq!(classify(&resp(false, "", stderr)), Outcome::RuntimeError);
}

#[test]
fn extract_error_codes_dedups_in_order() {
    let stderr = "error[E0308]: a\nerror[E0502]: b\nerror[E0308]: c\nerror: aborting";
    assert_eq!(extract_error_codes(stderr), vec!["E0308", "E0502"]);
}

#[test]
fn classify_line_kinds() {
    assert_eq!(classify_line("error[E0308]: mismatched types"), LineKind::Error);
    assert_eq!(classify_line("error: aborting due to 1 previous error"), LineKind::Error);
    assert_eq!(classify_line("warning: unused variable: `x`"), LineKind::Warning);
    assert_eq!(classify_line(" --> src/lib.rs:2:5"), LineKind::Note);
    assert_eq!(classify_line("note: expected type `i32`"), LineKind::Note);
    assert_eq!(classify_line("help: try removing `&`"), LineKind::Note);
    assert_eq!(classify_line("   Compiling playground v0.0.1"), LineKind::Plain);
}
