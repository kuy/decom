use flaterm_macro::layout;

struct Block;

impl Block {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn test_layout_macro() {
    layout! {
        <Block></Block>
    };
    layout! {
        <Block />
    };
}
