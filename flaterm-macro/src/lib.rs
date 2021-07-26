use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{buffer::Cursor, parse_macro_input};

mod children;
mod layout;
mod node;
mod props;
mod tag;

// ref. https://github.com/yewstack/yew/tree/master/packages/yew-macro

#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as node::Node);
    TokenStream::from(root.into_token_stream())
}

trait PeekValue<T> {
    fn peek(cursor: Cursor) -> Option<T>;
}
