#[test]
fn ui() {
    trybuild::TestCases::new().compile_fail("tests/fails/*.rs");
}