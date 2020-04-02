extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

mod args;
mod attrs;
mod error;
mod fields;
mod generate;
mod merge;

use args::Args;
use generate::generate;

#[proc_macro_attribute]
pub fn optfield(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: ItemStruct = parse_macro_input!(item);
    let args: Args = parse_macro_input!(attr);

    let opt_item = generate(args, &item);

    let out = quote! {
        #item

        #opt_item
    };

    out.into()
}

#[cfg(test)]
mod test_util {
    use proc_macro2::TokenStream;
    use syn::{parse::Parser, parse2, Attribute, Field, ItemStruct};

    use crate::args::Args;
    use crate::attrs::generator::is_doc_attr;

    pub fn parse_item_and_args(
        item_tokens: TokenStream,
        args_tokens: TokenStream,
    ) -> (ItemStruct, Args) {
        (parse_item(item_tokens), parse_args(args_tokens))
    }

    pub fn parse_field_and_args(
        field_tokens: TokenStream,
        args_tokens: TokenStream,
    ) -> (Field, Args) {
        (parse_field(field_tokens), parse_args(args_tokens))
    }

    pub fn parse_item(tokens: TokenStream) -> ItemStruct {
        parse2(tokens).unwrap()
    }

    pub fn parse_field(tokens: TokenStream) -> Field {
        Field::parse_named.parse2(tokens).unwrap()
    }

    pub fn parse_args(tokens: TokenStream) -> Args {
        parse2(tokens).unwrap()
    }

    pub fn parse_attr(tokens: TokenStream) -> Attribute {
        parse_attrs(tokens).get(0).unwrap().clone()
    }

    pub fn parse_attrs(tokens: TokenStream) -> Vec<Attribute> {
        let parser = Attribute::parse_outer;
        parser.parse2(tokens).unwrap()
    }

    pub fn attrs_contain_all(attrs: &[Attribute], other_attrs: &[Attribute]) -> bool {
        for attr in other_attrs {
            if !attrs.contains(attr) {
                return false;
            }
        }

        true
    }

    pub fn attrs_contain_any(attrs: &[Attribute], any_attrs: &[Attribute]) -> bool {
        for attr in any_attrs {
            if attrs.contains(attr) {
                return true;
            }
        }

        false
    }

    pub fn doc_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
        attrs
            .iter()
            .filter(|a| is_doc_attr(a))
            .map(|a| a.clone())
            .collect()
    }
}
