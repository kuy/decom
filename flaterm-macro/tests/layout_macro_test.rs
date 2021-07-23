use flaterm::Block;
use flaterm_macro::layout;

#[test]
fn test_layout_macro() {
    layout! {
        <Block></Block>
    };
    layout! {
        <Block />
    };
    layout! {
        <Block>
            <Block></Block>
            <Block />
        </Block>
    };
}
