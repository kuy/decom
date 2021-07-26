use crate::node::Node;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

pub struct LayoutChildren(pub Vec<Node>);

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
