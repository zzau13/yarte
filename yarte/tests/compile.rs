#[test]
fn ui() {
    trybuild::TestCases::new().compile_fail("tests/fails/*.rs");
}

#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio2"))]
#[test]
fn proc_ui() {
    trybuild::TestCases::new().compile_fail("tests/proc-fails/*.rs");
}
