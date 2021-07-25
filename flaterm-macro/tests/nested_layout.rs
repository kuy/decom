use flaterm::Block;
use flaterm_macro::layout;

pub fn test_nested_layout() {
    layout! {
        <Block>
            <Block></Block>
            <Block />
        </Block>
    };
}
