use macro_harness::run;

#[test]
fn test_node() {
    run("tests/node.rs");
}

#[test]
fn test_attribute() {
    run("tests/attribute.rs");
}
