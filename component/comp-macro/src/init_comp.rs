// SPDX-License-Identifier: MPL-2.0

use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{Attribute, parse::Parse};

/// The content inside typeflag macro
pub struct ComponentInitFunction {
    _attrs: Vec<Attribute>,
    function: Vec<TokenTree>,
    pub function_name: TokenTree,
}

impl Parse for ComponentInitFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _attrs = input.call(Attribute::parse_inner)?;
        let mut vec: Vec<TokenTree> = Vec::new();
        vec.push(input.parse().unwrap());
        vec.push(input.parse().unwrap());
        let function_name: TokenTree = input.parse().unwrap();
        vec.push(function_name.clone());
        while !input.is_empty() {
            vec.push(input.parse().unwrap());
        }
        Ok(ComponentInitFunction {
            _attrs,
            function: vec,
            function_name,
        })
    }
}

impl ToTokens for ComponentInitFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for token in &self.function {
            tokens.append(token.clone());
        }
    }
}
