use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{quote_spanned, ToTokens};
use syn::parse_macro_input;
use syn::{
    parse::{Parse, ParseStream, Parser},
    spanned::Spanned,
    Token, Type,
};

#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as LayoutComponent);
    TokenStream::from(root.into_token_stream())
}

struct LayoutComponent {
    ty: Type,
}

impl Parse for LayoutComponent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open = input.parse::<LayoutComponentOpen>()?;
        if open.is_self_closing() {
            return Ok(Self { ty: open.ty });
        }

        input.parse::<LayoutComponentClose>()?;

        Ok(Self { ty: open.ty })
    }
}

impl ToTokens for LayoutComponent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ty } = self;
        tokens.extend(quote_spanned! { ty.span() => {
            #ty::new()
        } });
    }
}

struct LayoutComponentOpen {
    tag: TagTokens,
    ty: Type,
}

impl LayoutComponentOpen {
    fn is_self_closing(&self) -> bool {
        self.tag.div.is_some()
    }
}

impl Parse for LayoutComponentOpen {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (tag, content) = TagTokens::parse_start_tag(input)?;
        let content_parser = |input: ParseStream| {
            let ty = input.parse()?;
            Ok((ty,))
        };
        let (ty,) = content_parser.parse2(content)?;
        Ok(Self { tag, ty })
    }
}

struct LayoutComponentClose {
    tag: TagTokens,
    _ty: Type,
}

impl Parse for LayoutComponentClose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (tag, content) = TagTokens::parse_end_tag(input)?;
        let content_parser = |input: ParseStream| {
            let ty = input.parse()?;
            Ok((ty,))
        };
        let (ty,) = content_parser.parse2(content)?;
        Ok(Self { tag, _ty: ty })
    }
}

/// ref. yew-macro
/// https://github.com/yewstack/yew/blob/b3ed684f0b859cf826a398304856574739825666/packages/yew-macro/src/html_tree/tag.rs#L32-L36
struct TagTokens {
    pub lt: Token![<],
    pub div: Option<Token![/]>,
    pub gt: Token![>],
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
