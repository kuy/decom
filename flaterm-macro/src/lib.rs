/// ref. https://github.com/yewstack/yew/tree/master/packages/yew-macro
use boolinator::Boolinator;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    buffer::Cursor,
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    spanned::Spanned,
    Expr, Token,
};

#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as Node);
    TokenStream::from(root.into_token_stream())
}

trait PeekValue<T> {
    fn peek(cursor: Cursor) -> Option<T>;
}

enum NodeType {
    Layout,
    Empty,
}

enum Node {
    Layout(Box<Layout>),
    Empty,
}

impl Node {
    fn peek_node_type(input: ParseStream) -> Option<NodeType> {
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

struct Layout {
    name: Ident,
    children: LayoutChildren,
    props: Props,
}

impl Parse for Layout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open = input.parse::<LayoutOpenTag>()?;
        if open.is_self_closing() {
            return Ok(Self {
                name: open.name,
                children: LayoutChildren(vec![]),
                props: open.props,
            });
        }

        let mut children: Vec<Node> = vec![];
        loop {
            if let Some(ty) = LayoutCloseTag::peek(input.cursor()) {
                if open.name == ty {
                    break;
                }
            }

            children.push(input.parse()?);
        }

        input.parse::<LayoutCloseTag>()?;

        Ok(Self {
            name: open.name,
            children: LayoutChildren(children),
            props: open.props,
        })
    }
}

impl ToTokens for Layout {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            children,
            props,
        } = self;
        let name_str = name.to_string();
        let layout_ident = Ident::new("__flaterm_l", Span::call_site());
        let props_token_stream = props.to_token_stream();
        let props_assignment = if props_token_stream.is_empty() {
            quote!()
        } else {
            quote!(#layout_ident.props = __flaterm_p;)
        };
        let children_token_stream = children.to_token_stream();
        let children_assignment = if children_token_stream.is_empty() {
            quote!()
        } else {
            quote!(#layout_ident.children = __flaterm_c;)
        };
        tokens.extend(quote! {
            {
                let mut #layout_ident = ::flaterm::Node::new(::std::string::String::from(#name_str));
                #props_token_stream
                #props_assignment
                #children_token_stream
                #children_assignment
                #layout_ident
            }
        });
    }
}

impl Layout {
    fn peek_name(cursor: Cursor) -> (Ident, Cursor) {
        cursor.ident().unwrap()
    }
}

struct LayoutOpenTag {
    tag: TagTokens,
    name: Ident,
    props: Props,
}

impl LayoutOpenTag {
    fn is_self_closing(&self) -> bool {
        self.tag.div.is_some()
    }
}

impl Parse for LayoutOpenTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (tag, inner_brackets) = TagTokens::parse_start_tag(input)?;
        let inner_parser = |input: ParseStream| {
            let name = input.parse()?;
            let props = input.parse()?;
            Ok((name, props))
        };
        let (name, props) = inner_parser.parse2(inner_brackets)?;
        Ok(Self { tag, name, props })
    }
}

struct LayoutCloseTag {
    _tag: TagTokens,
    _name: Ident,
}

impl Parse for LayoutCloseTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (tag, content) = TagTokens::parse_end_tag(input)?;
        let content_parser = |input: ParseStream| {
            let name = input.parse()?;
            Ok((name,))
        };
        let (name,) = content_parser.parse2(content)?;
        Ok(Self {
            _tag: tag,
            _name: name,
        })
    }
}

impl PeekValue<Ident> for LayoutCloseTag {
    fn peek(cursor: Cursor) -> Option<Ident> {
        let (punct, cursor) = cursor.punct()?;
        (punct.as_char() == '<').as_option()?;

        let (punct, cursor) = cursor.punct()?;
        (punct.as_char() == '/').as_option()?;

        let (ty, cursor) = Layout::peek_name(cursor);

        let (punct, _) = cursor.punct()?;
        (punct.as_char() == '>').as_option()?;

        Some(ty)
    }
}

struct LayoutChildren(Vec<Node>);

impl ToTokens for LayoutChildren {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self(children) = self;
        if children.is_empty() {
            return; // nothing to generate
        }

        let vec_ident = Ident::new("__flaterm_c", Span::call_site());
        let push_children_streams = children.iter().map(|child| {
            quote_spanned! {
                child.span()=> #vec_ident.push(::std::convert::Into::into(#child));
            }
        });

        tokens.extend(quote! {
            let mut #vec_ident: ::std::vec::Vec<::flaterm::Node> = ::std::default::Default::default();
            #(#push_children_streams)*
        });
    }
}

struct TagTokens {
    _lt: Token![<],
    div: Option<Token![/]>,
    _gt: Token![>],
}

impl TagTokens {
    pub fn parse_start_tag(input: ParseStream) -> syn::Result<(Self, TokenStream2)> {
        let lt = input.parse()?;
        let (inner_brackets, div, gt) = Self::parse_until_end_bracket(input)?;

        Ok((
            Self {
                _lt: lt,
                div,
                _gt: gt,
            },
            inner_brackets,
        ))
    }

    pub fn parse_end_tag(input: ParseStream) -> syn::Result<(Self, TokenStream2)> {
        let lt = input.parse()?;
        let div = Some(input.parse()?);
        let (inner_brackets, _end_div, gt) = Self::parse_until_end_bracket(input)?;

        Ok((
            Self {
                _lt: lt,
                div,
                _gt: gt,
            },
            inner_brackets,
        ))
    }

    pub fn parse_until_end_bracket(
        input: ParseStream,
    ) -> syn::Result<(TokenStream2, Option<Token![/]>, Token![>])> {
        let mut inner_brackets = vec![];
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

            inner_brackets.push(next);
        }

        Ok((inner_brackets.into_iter().collect(), div, gt))
    }
}

struct Props(Vec<Prop>);

impl Parse for Props {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut props = vec![];
        while !input.is_empty() {
            props.push(input.parse()?);
        }
        Ok(Self(props))
    }
}

impl ToTokens for Props {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self(props) = self;
        if props.is_empty() {
            return; // nothing to generate
        }

        let map_ident = Ident::new("__flaterm_p", Span::call_site());
        let insert_props_streams = props.iter().map(|prop| {
            let Prop { key, value } = prop;
            let key_str = key.to_string();
            quote!(#map_ident.insert(::std::string::String::from(#key_str), ::std::convert::Into::into(#value));)
        });

        tokens.extend(quote! {
            let mut #map_ident: ::std::collections::btree_map::BTreeMap<::std::string::String, ::flaterm::PropValue> = ::std::default::Default::default();
            #(#insert_props_streams)*
        });
    }
}

struct Prop {
    key: Ident,
    value: Expr,
}

impl Parse for Prop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}
