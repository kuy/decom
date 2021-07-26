use flaterm_macro::layout;

fn test() {
    // single attr
    layout! {
        <Block title="flaterm-macro Manual"></Block>
    };

    layout! {
        <Block title="flaterm-macro Manual" />
    };

    // multiple attrs
    layout! {
        <Block title="flaterm-macro Manual" height=3></Block>
    };
}
