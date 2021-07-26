use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use syn::{parse::ParseStream, Token};

pub struct TagTokens {
    _lt: Token![<],
    pub div: Option<Token![/]>,
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
