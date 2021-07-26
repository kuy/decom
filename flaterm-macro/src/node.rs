use crate::layout::Layout;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

pub enum NodeType {
    Layout,
    Empty,
}

pub enum Node {
    Layout(Box<Layout>),
    Empty,
}

impl Node {
    pub fn peek_node_type(input: ParseStream) -> Option<NodeType> {
        let input = input.fork();
        if input.is_empty() {
            Some(NodeType::Empty)
        } else if input.peek(Token![<]) {
            Some(NodeType::Layout)
        } else {
            None
        }
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let layout_node = match Self::peek_node_type(input) {
            Some(NodeType::Layout) => Node::Layout(Box::new(input.parse()?)),
            Some(NodeType::Empty) => Node::Empty,
            None => {
                return Err(input.error("unexpected node"));
            }
        };
        Ok(layout_node)
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Node::Layout(item) => item.to_tokens(tokens),
            _ => (),
        }
    }
}
