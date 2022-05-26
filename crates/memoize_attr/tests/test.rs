#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/test_pass.rs");
    t.compile_fail("tests/ui/test_fail.rs");
}
