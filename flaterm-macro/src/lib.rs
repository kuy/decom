use boolinator::Boolinator;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    buffer::Cursor,
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Colon2,
    Path, PathArguments, PathSegment, Token, Type, TypePath,
};

/// ref. https://github.com/yewstack/yew/tree/master/packages/yew-macro

#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as LayoutNode);
    TokenStream::from(root.into_token_stream())
}

trait PeekValue<T> {
    fn peek(cursor: Cursor) -> Option<T>;
}

enum NodeType {
    Item,
    Empty,
}

enum LayoutNode {
    Item(Box<LayoutItem>),
    Empty,
}

impl LayoutNode {
    fn peek_node_type(input: ParseStream) -> Option<NodeType> {
        let input = input.fork();
        if input.is_empty() {
            Some(NodeType::Empty)
        } else if input.peek(Token![<]) {
            Some(NodeType::Item)
        } else {
            None
        }
    }
}

impl Parse for LayoutNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let layout_node = match Self::peek_node_type(input) {
            Some(NodeType::Item) => LayoutNode::Item(Box::new(input.parse()?)),
            Some(NodeType::Empty) => LayoutNode::Empty,
            None => {
                return Err(input.error("unexpected node"));
            }
        };
        Ok(layout_node)
    }
}

impl ToTokens for LayoutNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            LayoutNode::Item(item) => item.to_tokens(tokens),
            _ => (),
        }
    }
}

struct LayoutItem {
    name: Ident,
    children: LayoutChildren,
}

impl Parse for LayoutItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open = input.parse::<LayoutItemOpen>()?;
        if open.is_self_closing() {
            return Ok(Self {
                name: open.name,
                children: LayoutChildren(vec![]),
            });
        }

        let mut children: Vec<LayoutNode> = vec![];
        loop {
            if let Some(ty) = LayoutItemClose::peek(input.cursor()) {
                if open.name == ty {
                    break;
                }
            }

            children.push(input.parse()?);
        }

        input.parse::<LayoutItemClose>()?;

        Ok(Self {
            name: open.name,
            children: LayoutChildren(children),
        })
    }
}

impl ToTokens for LayoutItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { name, children } = self;
        let name_str = name.to_string();
        let item_ident = Ident::new("__flaterm_item", Span::call_site());
        let children_token_stream = children.to_token_stream();
        let assign_statement = if children_token_stream.is_empty() {
            TokenStream2::new()
        } else {
            quote! {
                #item_ident.children = __flaterm_vec;
            }
        };
        tokens.extend(quote! {
            {
                let mut #item_ident = ::flaterm::Node::new(::std::string::String::from(#name_str));
                #children_token_stream;
                #assign_statement
                #item_ident
            }
        });
    }
}

impl LayoutItem {
    fn peek_name(mut cursor: Cursor) -> (Ident, Cursor) {
        cursor.ident().unwrap()
    }
}

struct LayoutItemOpen {
    tag: TagTokens,
    name: Ident,
}

impl LayoutItemOpen {
    fn is_self_closing(&self) -> bool {
        self.tag.div.is_some()
    }
}

impl Parse for LayoutItemOpen {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (tag, content) = TagTokens::parse_start_tag(input)?;
        let content_parser = |input: ParseStream| {
            let name = input.parse()?;
            Ok((name,))
        };
        let (name,) = content_parser.parse2(content)?;
        Ok(Self { tag, name })
    }
}

struct LayoutItemClose {
    tag: TagTokens,
    _name: Ident,
}

impl Parse for LayoutItemClose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (tag, content) = TagTokens::parse_end_tag(input)?;
        let content_parser = |input: ParseStream| {
            let name = input.parse()?;
            Ok((name,))
        };
        let (name,) = content_parser.parse2(content)?;
        Ok(Self { tag, _name: name })
    }
}

impl PeekValue<Ident> for LayoutItemClose {
    fn peek(cursor: Cursor) -> Option<Ident> {
        let (punct, cursor) = cursor.punct()?;
        (punct.as_char() == '<').as_option()?;

        let (punct, cursor) = cursor.punct()?;
        (punct.as_char() == '/').as_option()?;

        let (ty, cursor) = LayoutItem::peek_name(cursor);

        let (punct, _) = cursor.punct()?;
        (punct.as_char() == '>').as_option()?;

        Some(ty)
    }
}

struct LayoutChildren(Vec<LayoutNode>);

impl ToTokens for LayoutChildren {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self(children) = self;
        if children.is_empty() {
            return; // nothing to generate
        }

        let vec_ident = Ident::new("__flaterm_vec", Span::call_site());
        let push_children_streams = children.iter().map(|child| {
            quote_spanned! {
                child.span()=> #vec_ident.push(::std::convert::Into::into(#child));
            }
        });
        tokens.extend(quote! {
            let mut #vec_ident: ::std::vec::Vec<::flaterm::Node> = ::std::vec::Vec::new();
            #(#push_children_streams)*
        });
    }
}

struct TagTokens {
    lt: Token![<],
    div: Option<Token![/]>,
    gt: Token![>],
}

impl TagTokens {
    pub fn parse_start_tag(input: ParseStream) -> syn::Result<(Self, TokenStream2)> {
        let lt = input.parse()?;
        let (content, div, gt) = Self::parse_until_tag_end(input)?;

        Ok((Self { lt, div, gt }, content))
    }

    pub fn parse_end_tag(input: ParseStream) -> syn::Result<(Self, TokenStream2)> {
        let lt = input.parse()?;
        let div = Some(input.parse()?);
        let (content, _end_div, gt) = Self::parse_until_tag_end(input)?;

        Ok((Self { lt, div, gt }, content))
    }

    pub fn parse_until_tag_end(
        input: ParseStream,
    ) -> syn::Result<(TokenStream2, Option<Token![/]>, Token![>])> {
        let mut content = vec![];
        let mut div: Option<Token![/]> = None;
        let gt: Token![>];

        loop {
            let next = input.parse()?;
            if let TokenTree::Punct(punct) = &next {
                match punct.as_char() {
                    '/' => {
                        div = Some(syn::token::Div {
                            spans: [punct.span()],
                        });
                        gt = input.parse()?;
                        break;
                    }
                    '>' => {
                        gt = syn::token::Gt {
                            spans: [punct.span()],
                        };
                        break;
                    }
                    _ => (),
                }
            }

            content.push(next);
        }

        Ok((content.into_iter().collect(), div, gt))
    }
}
