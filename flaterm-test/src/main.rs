use flaterm::Block;
use flaterm_macro::layout;

fn main() {
    layout! {
        <Block>
            <Block></Block>
            <Block />
        </Block>
    };
}
