use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

pub struct Props(pub Vec<Prop>);

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

pub struct Prop {
    pub key: Ident,
    pub value: Expr,
}

impl Parse for Prop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}
