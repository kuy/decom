use macro_harness::run;

#[test]
pub fn test_basic_layout() {
    run("tests/basic_layout.rs");
}

#[test]
pub fn test_nested_layout() {
    run("tests/nested_layout.rs");
}
