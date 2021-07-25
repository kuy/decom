use macro_harness::run;

// #[test]
fn test_basic_layout() {
    run("tests/basic_layout.rs");
}

// #[test]
fn test_nested_layout() {
    run("tests/nested_layout.rs");
}

#[test]
fn test_node() {
    run("tests/node.rs");
}
