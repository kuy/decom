use flaterm::Block;
use flaterm_macro::layout;

pub fn test_basic_layout() {
    layout! {
        <Block></Block>
    };

    layout! {
        <Block />
    };
}
