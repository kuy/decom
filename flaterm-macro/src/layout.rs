use crate::props::Props;
use crate::tag::TagTokens;
use crate::PeekValue;
use crate::{children::LayoutChildren, node::Node};
use boolinator::Boolinator;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream, Parser};

pub struct Layout {
    pub name: Ident,
    pub children: LayoutChildren,
    pub props: Props,
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

pub struct LayoutOpenTag {
    pub tag: TagTokens,
    pub name: Ident,
    pub props: Props,
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

pub struct LayoutCloseTag {
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
